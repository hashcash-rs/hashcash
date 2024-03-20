// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use hashcash::{
	client::api::{
		self, consensus, BlockSubmitParams, MinerData, MiningHandle, MiningMetadata, Seal, Version,
	},
	primitives::core::{opaque::Block, AccountId, Difficulty, Hash},
};
use std::sync::Arc;
use substrate::{
	client::{api::HeaderBackend, utils::mpsc::TracingUnboundedSender},
	codec::Decode,
	primitives::runtime::traits::Block as BlockT,
};

pub struct MiningWorkerBackend<C, H> {
	client: Arc<C>,
	handle: Arc<H>,
	submit: TracingUnboundedSender<BlockSubmitParams<Block>>,
	metadata: Option<MiningMetadata>,
	miner_data: Option<MinerData>,
}

impl<C, H> Clone for MiningWorkerBackend<C, H> {
	fn clone(&self) -> Self {
		Self {
			client: self.client.clone(),
			handle: self.handle.clone(),
			submit: self.submit.clone(),
			metadata: self.metadata.clone(),
			miner_data: self.miner_data.clone(),
		}
	}
}

impl<C, H> MiningWorkerBackend<C, H>
where
	C: HeaderBackend<Block>,
	H: MiningHandle,
{
	pub fn new(
		client: Arc<C>,
		handle: Arc<H>,
		submit: TracingUnboundedSender<BlockSubmitParams<Block>>,
	) -> Self {
		Self { client, handle, submit, metadata: None, miner_data: None }
	}

	pub fn mainchain_difficulty(&self) -> Difficulty {
		self.miner_data.as_ref().unwrap().difficulty
	}
}

impl<C, H> api::MiningWorkerBackend<Hash, Difficulty> for MiningWorkerBackend<C, H>
where
	C: HeaderBackend<Block>,
	H: MiningHandle,
{
	fn seed_hash(&self) -> Option<Hash> {
		self.miner_data.as_ref().map(|v| v.seed_hash)
	}

	fn pre_hash(&self) -> Hash {
		self.miner_data.as_ref().unwrap().block.hash()
	}

	fn difficulty(&self) -> Difficulty {
		self.metadata.as_ref().unwrap().difficulty
	}

	fn version(&self) -> Version {
		self.handle.version()
	}

	fn submit(&self, work: Hash, seal: Seal) -> bool {
		let res = self.handle.submit(seal.clone());

		if consensus::check_hash(&work, self.mainchain_difficulty()) {
			let _ = self.submit.unbounded_send(BlockSubmitParams {
				block: self.miner_data.as_ref().unwrap().block.clone(),
				seal,
			});
		}
		res
	}

	fn bump(&mut self) -> bool {
		self.metadata = self.handle.metadata();

		if let Some(ref metadata) = self.metadata {
			match metadata
				.pre_runtime
				.as_ref()
				.map(|v| <(AccountId, MinerData)>::decode(&mut &v[..]))
			{
				Some(Ok((_, miner_data))) => {
					self.miner_data = Some(miner_data);
					true
				},
				_ => {
					self.metadata = None;
					false
				},
			}
		} else {
			false
		}
	}
}
