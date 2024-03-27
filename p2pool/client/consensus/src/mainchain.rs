// Copyright (c) The Hashcash Authors
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use futures::FutureExt;
use hashcash::{
	client::{
		api::consensus::{seed_height, SEEDHASH_EPOCH_BLOCKS},
		utils::rpc::{
			reconnecting_rpc_client::FibonacciBackoff, rpc_params, Error, RpcClient,
			RpcSubscription,
		},
	},
	primitives::core::{opaque::Header, BlockNumber},
};
use log::*;
use parking_lot::RwLock;
use std::{
	collections::{BTreeMap, HashMap},
	sync::Arc,
};
use substrate::primitives::{blockchain::BlockStatus, core::traits::SpawnNamed};
use tokio::time::Duration;

const BLOCK_HEADERS_REQUIRED: BlockNumber = 720;

#[derive(Debug, Default)]
pub struct Mainchain {
	by_height: BTreeMap<BlockNumber, Arc<Header>>,
	by_hash: HashMap<Hash, Arc<Header>>,
}

impl Mainchain {
	pub fn new() -> Self {
		Default::default()
	}

	pub fn header(&self, hash: Option<Hash>) -> Option<Header> {
		match hash {
			Some(hash) => self.by_hash.get(&hash).map(|h| Arc::unwrap_or_clone(h.clone())),
			None => self.by_height.last_key_value().map(|(_, h)| Arc::unwrap_or_clone(h.clone())),
		}
	}

	pub fn status(&self, hash: Hash) -> BlockStatus {
		match self.by_hash.get(&hash) {
			Some(_) => BlockStatus::InChain,
			None => BlockStatus::Unknown,
		}
	}

	pub fn number(&self, hash: Option<Hash>) -> Option<BlockNumber> {
		match hash {
			Some(hash) => self.by_hash.get(&hash).map(|h| h.number),
			None => self.by_height.last_key_value().map(|(n, _)| *n),
		}
	}

	pub fn hash(&self, number: Option<BlockNumber>) -> Option<Hash> {
		match number {
			Some(number) => self.by_height.get(&number).map(|h| h.hash()),
			None => self.by_height.last_key_value().map(|(_, h)| h.hash()),
		}
	}

	pub fn seed_hash(&self, number: BlockNumber) -> Option<Hash> {
		self.hash(Some(seed_height(number)))
	}
}

#[derive(Clone)]
pub struct MainchainReader {
	rpc_client: RpcClient,
	chain: Arc<RwLock<Mainchain>>,
}

macro_rules! retry {
	($fut:expr) => {
		{
			let mut backoff = FibonacciBackoff::from_millis(100).max_delay(Duration::from_secs(10));
			loop {
				match $fut.await {
					Ok(v) => break v,
					Err(e) => error!(target: LOG_TARGET, "Mainchain RPC error: {:?}", e),
				}
				tokio::time::sleep(backoff.next().unwrap()).await;
			}
		}
	};
}

impl MainchainReader {
	pub fn new(
		rpc_client: RpcClient,
		chain: Arc<RwLock<Mainchain>>,
		spawner: impl SpawnNamed,
	) -> Self where {
		let reader = Self { rpc_client, chain };
		spawner.spawn("mainchain-reader", None, reader.clone().init().boxed());
		reader
	}

	pub async fn run(mut self) {
		let mut subscription = retry!(self.chain_subscribe_new_heads());
		while let Some(header) = subscription.next().await {
			match header {
				Ok(header) => self.import_header(header),
				Err(e) => error!(target: LOG_TARGET, "Mainchain subscription error: {:?}", e),
			}
		}
	}

	async fn init(mut self) {
		let best_header =
			retry!(self.chain_get_header(None)).expect("Best header must be found; qed");
		let seed_height = seed_height(best_header.number);
		let prev_seed_height = seed_height.saturating_sub(SEEDHASH_EPOCH_BLOCKS);
		if let Some(header) = self.fetch_header(Some(seed_height)).await {
			self.import_header(header);
		}
		if prev_seed_height != seed_height {
			if let Some(header) = self.fetch_header(Some(prev_seed_height)).await {
				self.import_header(header);
			}
		}
		let begin_height = best_header.number.saturating_sub(BLOCK_HEADERS_REQUIRED - 1);
		for height in (begin_height..best_header.number).rev() {
			if let Some(header) = self.fetch_header(Some(height)).await {
				self.import_header(header);
			}
		}
		debug!(target: LOG_TARGET, "Mainchain sync initialized.");
	}

	async fn fetch_header(&self, number: Option<BlockNumber>) -> Option<Header> {
		let hash = retry!(self.chain_get_block_hash(number))?;
		retry!(self.chain_get_header(Some(hash)))
	}

	fn import_header(&mut self, header: Header) {
		let header = Arc::new(header);
		let (height, hash) = (header.number, header.hash());
		{
			let mut chain = self.chain.write();
			if let Some(h) = chain.by_height.insert(height, header.clone()) {
				if h.hash() != hash {
					info!(target: LOG_TARGET, "♻️  Mainchain reorg on #{},{} to #{},{}", height, h.hash(), height, hash);
				}
			}
			chain.by_hash.insert(hash, header);

			if height > BLOCK_HEADERS_REQUIRED {
				if let Some(header) = chain.by_height.remove(&(height - BLOCK_HEADERS_REQUIRED)) {
					chain.by_hash.remove(&header.hash());
				}
			}
		}
		info!(target: LOG_TARGET, "📥 Imported mainchain #{} ({})", height, hash);
	}

	async fn chain_get_block_hash(
		&self,
		number: Option<BlockNumber>,
	) -> Result<Option<Hash>, Error> {
		let hash = self.rpc_client.request("chain_getBlockHash", rpc_params![number]).await?;
		Ok(hash)
	}

	async fn chain_get_header(&self, hash: Option<Hash>) -> Result<Option<Header>, Error> {
		let header = self.rpc_client.request("chain_getHeader", rpc_params![hash]).await?;
		Ok(header)
	}

	async fn chain_subscribe_new_heads(&self) -> Result<RpcSubscription<Header>, Error> {
		let subscription = self
			.rpc_client
			.subscribe("chain_subscribeNewHeads", rpc_params![], "chain_unsubscribeNewHeads")
			.await?;
		Ok(subscription)
	}
}
