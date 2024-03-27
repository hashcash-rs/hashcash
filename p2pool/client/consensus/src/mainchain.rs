// Copyright (c) The Hashcash Authors
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use hashcash::{
	client::utils::rpc::{
		reconnecting_rpc_client::FibonacciBackoff, rpc_params, Error, RpcClient, RpcSubscription,
	},
	primitives::core::{opaque::Header, BlockNumber},
};
use log::*;
use parking_lot::RwLock;
use std::{
	collections::{BTreeMap, HashMap},
	sync::Arc,
};
use substrate::primitives::blockchain::BlockStatus;
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
}

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
	pub fn new(rpc_client: RpcClient, chain: Arc<RwLock<Mainchain>>) -> Self {
		MainchainReader { rpc_client, chain }
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

	fn import_header(&mut self, header: Header) {
		let header = Arc::new(header);
		let (height, hash) = (header.number, header.hash());
		{
			let mut chain = self.chain.write();
			if let Some(h) = chain.by_height.insert(height, header.clone()) {
				info!(target: LOG_TARGET, "♻️  Mainchain reorg on #{},{} to #{},{}", height, h.hash(), height, hash);
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

	async fn chain_subscribe_new_heads(&self) -> Result<RpcSubscription<Header>, Error> {
		let subscription = self
			.rpc_client
			.subscribe("chain_subscribeNewHeads", rpc_params![], "chain_unsubscribeNewHeads")
			.await?;
		Ok(subscription)
	}
}
