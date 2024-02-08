// Copyright (c) 2024 Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{common, preludes::*, randomx};

use hashcash::randomx::{RandomXFlags, RandomXVm};
use std::{sync::Arc, time::Duration};
pub use substrate::{
	client::{
		api::HeaderBackend,
		consensus::{pow::PowAlgorithm, BlockImport, JustificationSyncLink},
	},
	codec::Encode,
	primitives::{api::ProvideRuntimeApi, consensus::pow::Seal, runtime::traits::Block as BlockT},
};

pub type MiningMetadata = sc_consensus_pow::MiningMetadata<Hash, Difficulty>;

pub trait MiningHandle {
	fn metadata(&self) -> Option<MiningMetadata>;
	fn submit(&self, seal: Seal) -> bool;
}

impl<B, A, L, P, I> MiningHandle for sc_consensus_pow::MiningHandle<B, A, L, P, I>
where
	B: BlockT<Hash = Hash>,
	A: PowAlgorithm<B, Difficulty = Difficulty>,
	L: JustificationSyncLink<B>,
	I: BlockImport<B>,
{
	fn metadata(&self) -> Option<MiningMetadata> {
		sc_consensus_pow::MiningHandle::metadata(self)
	}

	fn submit(&self, seal: Seal) -> bool {
		futures::executor::block_on(sc_consensus_pow::MiningHandle::submit(self, seal))
	}
}

pub struct Miner<B: BlockT, C, H> {
	client: Arc<C>,
	handle: Arc<H>,
	_marker: std::marker::PhantomData<B>,
}

impl<C, H> Miner<Block, C, H>
where
	C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + 'static,
	H: MiningHandle + Send + Sync + 'static,
{
	pub fn new(client: Arc<C>, handle: Arc<H>) -> Self {
		Miner { client, handle, _marker: Default::default() }
	}

	pub fn start(&self, threads_count: usize) {
		for i in 0..threads_count {
			let client = self.client.clone();
			let handle = self.handle.clone();
			let nonce = i as Nonce;

			std::thread::spawn(move || loop {
				let mut nonce = nonce;

				loop {
					let metadata = handle.metadata();

					if let Some(metadata) = metadata {
						let seed_hash =
							common::seed_hash(&client, &BlockId::Hash(metadata.best_hash))
								.expect("");
						let dataset = randomx::get_or_init_dataset(&seed_hash).expect("");
						let mut vm = RandomXVm::new(
							randomx::get_flags() | RandomXFlags::FullMem,
							None,
							Some(dataset),
						)
						.expect("");
						let hash =
							Hash::from(vm.calculate_hash(&(metadata.pre_hash, nonce).encode()));

						if common::check_hash(&hash, metadata.difficulty) {
							handle.submit(common::Seal { nonce }.encode());
						}

						nonce += threads_count as Nonce;
					} else {
						std::thread::sleep(Duration::from_secs(1));
					}
				}
			});
		}
	}
}
