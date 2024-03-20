// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use hashcash::client::api::{self, consensus, MiningHandle, MiningMetadata, Seal, Version};
use std::sync::Arc;
use substrate::client::api::HeaderBackend;

pub struct MiningWorkerBackend<C, H> {
	client: Arc<C>,
	handle: Arc<H>,
	metadata: Option<MiningMetadata>,
}

impl<C, H> Clone for MiningWorkerBackend<C, H> {
	fn clone(&self) -> Self {
		Self {
			client: self.client.clone(),
			handle: self.handle.clone(),
			metadata: self.metadata.clone(),
		}
	}
}

impl<C, H> MiningWorkerBackend<C, H>
where
	C: HeaderBackend<Block>,
	H: MiningHandle,
{
	pub fn new(client: Arc<C>, handle: Arc<H>) -> Self {
		Self { client, handle, metadata: None }
	}
}

impl<C, H> api::MiningWorkerBackend<Hash, Difficulty> for MiningWorkerBackend<C, H>
where
	C: HeaderBackend<Block>,
	H: MiningHandle,
{
	fn seed_hash(&self) -> Option<Hash> {
		let best_hash = self.metadata.as_ref()?.best_hash;

		consensus::seed_hash(&self.client, &BlockId::Hash(best_hash)).ok()
	}

	fn pre_hash(&self) -> Hash {
		self.metadata.as_ref().unwrap().pre_hash
	}

	fn difficulty(&self) -> Difficulty {
		self.metadata.as_ref().unwrap().difficulty
	}

	fn version(&self) -> Version {
		self.handle.version()
	}

	fn submit(&self, _work: Hash, seal: Seal) -> bool {
		self.handle.submit(seal)
	}

	fn bump(&mut self) -> bool {
		self.metadata = self.handle.metadata();

		self.metadata.is_some()
	}
}
