// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{error::*, preludes::*, LOG_TARGET, STORAGE_KEY};

use futures::StreamExt;
use hashcash::primitives::{
	block_template::BlockTemplate,
	core::{AccountId, Difficulty},
};
use jsonrpsee::{
	core::{client::ClientT, params::ArrayParams},
	http_client::{HttpClient, HttpClientBuilder},
	rpc_params,
};
use p2pool::client::consensus::P2POOL_AUX_PREFIX;
use std::{collections::BTreeMap, sync::Arc, time::Duration};
use substrate::{
	client::{
		api::{backend::AuxStore, BlockchainEvents},
		consensus::pow::UntilImportedOrTimeout,
	},
	codec::{Decode, Encode},
	primitives::{
		blockchain::HeaderBackend,
		consensus::{pow::POW_ENGINE_ID, SelectChain, SyncOracle},
		runtime::{
			traits::{Block, Header, NumberFor, Saturating, Zero},
			DigestItem,
		},
	},
};

pub struct BlockTemplateSyncWorker<B: Block, C, S, SO> {
	rpc_client: HttpClient,
	client: Arc<C>,
	select_chain: S,
	author: AccountId,
	genesis_hash: B::Hash,
	window_size: NumberFor<B>,
	sync_oracle: SO,
	timeout: Duration,
}

impl<B, C, S, SO> BlockTemplateSyncWorker<B, C, S, SO>
where
	B: Block,
	C: AuxStore + BlockchainEvents<B> + HeaderBackend<B> + 'static,
	S: SelectChain<B>,
	SO: SyncOracle,
{
	pub fn new(
		mainchain_rpc: String,
		client: Arc<C>,
		select_chain: S,
		author: AccountId,
		genesis_hash: B::Hash,
		window_size: NumberFor<B>,
		sync_oracle: SO,
		timeout: Duration,
	) -> Result<Self, BlockTemplateError> {
		Ok(Self {
			rpc_client: HttpClientBuilder::default()
				.build(mainchain_rpc)
				.map_err(BlockTemplateError::HttpClient)?,
			client,
			select_chain,
			author,
			genesis_hash,
			window_size,
			sync_oracle,
			timeout,
		})
	}

	pub async fn run(self) {
		let mut timer =
			UntilImportedOrTimeout::new(self.client.import_notification_stream(), self.timeout);
		loop {
			if timer.next().await.is_none() {
				break;
			}

			if !self.sync_oracle.is_major_syncing() {
				match self.get_shares().await {
					Ok(shares) => self.update_block_template(shares).await,
					Err(e) => log::warn!(target: LOG_TARGET, "{}", e),
				}
			}
			tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
		}
	}

	async fn get_shares(&self) -> Result<Vec<(AccountId, Difficulty)>, String> {
		let best_header = self
			.select_chain
			.best_chain()
			.await
			.map_err(|e| format!("Unable to get best chain: {:?}", e))?;

		let mut shares = BTreeMap::<AccountId, Difficulty>::new();

		let mut current = best_header;
		let mut count = NumberFor::<B>::zero();
		while current.hash() != self.genesis_hash && count < self.window_size {
			let author = match self.get_author(current.clone()) {
				Ok(Some(author)) => author,
				Ok(None) => {
					current = self.get_parent(current)?;
					continue;
				},
				Err(e) => {
					log::warn!(target: LOG_TARGET, "{}", e);
					current = self.get_parent(current)?;
					continue;
				},
			};
			let difficulty = match self.get_dfficulty(current.clone()) {
				Ok(diffculty) => diffculty,
				Err(e) => {
					log::warn!(target: LOG_TARGET, "{}", e);
					current = self.get_parent(current)?;
					continue;
				},
			};
			match shares.get_mut(&author) {
				Some(value) => *value += difficulty,
				None => {
					shares.insert(author, difficulty);
				},
			};

			current = self.get_parent(current)?;
			count = count.saturating_plus_one();
		}

		let mut shares = shares
			.iter()
			.map(|(k, v)| (k.clone(), *v))
			.collect::<Vec<(AccountId, Difficulty)>>();

		if shares.is_empty() {
			shares.push((self.author.clone(), 1));
		}

		Ok(shares)
	}

	fn get_parent(&self, header: <B as Block>::Header) -> Result<<B as Block>::Header, String> {
		let parent_hash = header.parent_hash();
		let parent = self
			.client
			.header(*parent_hash)
			.map_err(|e| format!("Unable to get best chain: {:?}", e))?
			.ok_or(format!("Header does not exist: {:?}", parent_hash))?;
		Ok(parent)
	}

	fn get_author(&self, header: <B as Block>::Header) -> Result<Option<AccountId>, String> {
		let mut author: Option<AccountId> = None;
		for log in header.digest().logs() {
			if let DigestItem::PreRuntime(POW_ENGINE_ID, v) = log {
				if author.is_some() {
					return Err("Multiple authors exist".to_string());
				}
				author = Some(
					<(AccountId, Option<BlockTemplate>)>::decode(&mut &v[..])
						.map_err(|e| format!("Unable to decode: {:?}", e))?
						.0,
				);
			}
		}
		Ok(author)
	}

	fn get_dfficulty(&self, header: <B as Block>::Header) -> Result<Difficulty, String> {
		let key: Vec<u8> =
			P2POOL_AUX_PREFIX.iter().chain(header.hash().as_ref()).copied().collect();

		let difficulty = self
			.client
			.get_aux(&key)
			.map_err(|e| format!("Unable to get difficulty: {:?}", e))?
			.map(|v| Difficulty::decode(&mut &v[..]))
			.ok_or(format!("Difficulty does not exist: {:?}", header.hash()))?
			.map_err(|e| format!("Unable to decode: {:?}", e))?;
		Ok(difficulty)
	}

	async fn update_block_template(&self, shares: Vec<(AccountId, Difficulty)>) {
		match self
			.rpc_client
			.request::<BlockTemplate, ArrayParams>(
				"miner_getBlockTemplate",
				rpc_params!(self.author.clone(), shares),
			)
			.await
		{
			Ok(res) => {
				let _ = self.client.as_ref().insert_aux(&[(STORAGE_KEY, &res.encode()[..])], &[]);
			},
			Err(e) => {
				log::warn!(target: LOG_TARGET, "Unable to get block-template: {:?}", e);
			},
		}
	}
}
