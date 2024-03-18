// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{error::*, preludes::*, LOG_TARGET};

use hashcash::{
	client::api::MinerData,
	primitives::core::{AccountId, Difficulty},
};
use jsonrpsee::{
	core::{client::ClientT, params::ArrayParams},
	http_client::{HttpClient, HttpClientBuilder},
	rpc_params,
};
use p2pool::client::consensus::P2POOL_AUX_PREFIX;
use std::{collections::BTreeMap, sync::Arc};
use substrate::{
	client::api::{backend::AuxStore, BlockchainEvents},
	codec::Decode,
	primitives::{
		blockchain::HeaderBackend,
		consensus::pow::POW_ENGINE_ID,
		runtime::{
			traits::{Block, Header, NumberFor, Saturating, Zero},
			DigestItem,
		},
	},
};

#[derive(Clone)]
pub struct MinerDataProvider<B: Block, C> {
	rpc_client: HttpClient,
	client: Arc<C>,
	author: AccountId,
	genesis_hash: B::Hash,
	window_size: NumberFor<B>,
}

impl<B, C> MinerDataProvider<B, C>
where
	B: Block,
	C: AuxStore + BlockchainEvents<B> + HeaderBackend<B> + 'static,
{
	pub fn new(
		mainchain_rpc: String,
		client: Arc<C>,
		author: AccountId,
		genesis_hash: B::Hash,
		window_size: NumberFor<B>,
	) -> Result<Self, MinerDataError> {
		Ok(Self {
			rpc_client: HttpClientBuilder::default()
				.build(mainchain_rpc)
				.map_err(MinerDataError::HttpClient)?,
			client,
			author,
			genesis_hash,
			window_size,
		})
	}

	pub async fn miner_data(&self, best_hash: &B::Hash) -> Option<MinerData> {
		match self.miner_data_inner(best_hash).await {
			Ok(miner_data) => Some(miner_data),
			Err(e) => {
				log::warn!(target: LOG_TARGET, "{:?}", e);
				None
			},
		}
	}

	async fn miner_data_inner(&self, best_hash: &B::Hash) -> Result<MinerData, MinerDataError> {
		let shares = self.get_shares(best_hash).await?;
		self.rpc_client
			.request::<MinerData, ArrayParams>(
				"miner_getMinerData",
				rpc_params!(self.author.clone(), shares),
			)
			.await
			.map_err(MinerDataError::HttpClient)
	}

	async fn get_shares(
		&self,
		best_hash: &B::Hash,
	) -> Result<Vec<(AccountId, Difficulty)>, MinerDataError> {
		let best_header = self
			.client
			.header(*best_hash)
			.map_err(MinerDataError::Blockchain)?
			.ok_or(MinerDataError::Other(format!("Header does not exist: {:?}", best_hash)))?;

		let mut shares = BTreeMap::<AccountId, Difficulty>::new();

		let mut current = best_header;
		let mut count = NumberFor::<B>::zero();
		while current.hash() != self.genesis_hash && count < self.window_size {
			let author = self
				.author_of(current.clone())?
				.ok_or(MinerDataError::Other("Author does not exist".to_string()))?;
			let difficulty = self.difficulty_of(current.clone())?;
			match shares.get_mut(&author) {
				Some(value) => {
					*value = value.saturating_add(difficulty);
				},
				None => {
					shares.insert(author, difficulty);
				},
			};

			current = self.parent_of(current)?;
			count = count.saturating_plus_one();
		}

		let mut shares = shares
			.iter()
			.map(|(k, v)| (k.clone(), *v))
			.collect::<Vec<(AccountId, Difficulty)>>();

		if shares.is_empty() {
			shares.push((self.author.clone(), 1));
		}

		log::debug!(target: LOG_TARGET, "📊 Shares: {:?}", shares);
		Ok(shares)
	}

	fn parent_of(
		&self,
		header: <B as Block>::Header,
	) -> Result<<B as Block>::Header, MinerDataError> {
		let parent_hash = header.parent_hash();
		let parent =
			self.client.header(*parent_hash).map_err(MinerDataError::Blockchain)?.ok_or(
				MinerDataError::Other(format!("Header does not exist: {:?}", parent_hash)),
			)?;
		Ok(parent)
	}

	fn author_of(&self, header: <B as Block>::Header) -> Result<Option<AccountId>, MinerDataError> {
		let mut author: Option<AccountId> = None;
		for log in header.digest().logs() {
			if let DigestItem::PreRuntime(POW_ENGINE_ID, v) = log {
				if author.is_some() {
					return Err(MinerDataError::Other("Multiple authors exist".to_string()));
				}
				author = Some(
					<(AccountId, MinerData)>::decode(&mut &v[..]).map_err(MinerDataError::Codec)?.0,
				);
			}
		}
		Ok(author)
	}

	fn difficulty_of(&self, header: <B as Block>::Header) -> Result<Difficulty, MinerDataError> {
		let key: Vec<u8> =
			P2POOL_AUX_PREFIX.iter().chain(header.hash().as_ref()).copied().collect();

		let difficulty = self
			.client
			.get_aux(&key)
			.map_err(MinerDataError::Blockchain)?
			.map(|v| Difficulty::decode(&mut &v[..]))
			.ok_or(MinerDataError::Other(format!(
				"Difficulty does not exist: {:?}",
				header.hash()
			)))?
			.map_err(MinerDataError::Codec)?;
		Ok(difficulty)
	}
}
