// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{common, preludes::*, randomx};

use hashcash::randomx::{RandomXFlags, RandomXVm};
use log::*;
use rand::{thread_rng, Rng};
use std::{sync::Arc, time::Duration};
pub use substrate::{
	client::{
		api::HeaderBackend,
		consensus::{
			pow::{PowAlgorithm, Version},
			BlockImport, JustificationSyncLink,
		},
	},
	codec::Encode,
	primitives::{api::ProvideRuntimeApi, consensus::pow::Seal, runtime::traits::Block as BlockT},
};

pub type MiningMetadata = sc_consensus_pow::MiningMetadata<Hash, Difficulty>;

pub trait MiningHandle {
	fn metadata(&self) -> Option<MiningMetadata>;
	fn submit(&self, seal: Seal) -> bool;
	fn version(&self) -> Version;
	fn reset(&self);
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

	fn version(&self) -> Version {
		sc_consensus_pow::MiningHandle::version(self)
	}

	fn reset(&self) {
		sc_consensus_pow::MiningHandle::on_major_syncing(self)
	}
}

#[derive(Debug)]
enum Error {
	EmptyMetadata,
	SeedHashNotFetched,
	DatasetNotAllocated,
	VmNotCreated,
}

pub struct Miner<B: BlockT, C, H> {
	client: Arc<C>,
	handle: Arc<H>,
	nonce: Nonce,
	_marker: std::marker::PhantomData<B>,
}

impl<C, H> Miner<Block, C, H>
where
	C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + 'static,
	H: MiningHandle + Send + Sync + 'static,
{
	pub fn new(params: MinerParams<C, H>) -> Self {
		let MinerParams { client, handle, .. } = params;

		Miner { client, handle, nonce: thread_rng().gen(), _marker: Default::default() }
	}

	pub fn start(&self, threads_count: usize) {
		let threads_count = threads_count.max(1);
		info!(target: LOG_TARGET, "⚒️  Starting Miner with {} thread(s)", threads_count);

		for thread_index in 0..threads_count {
			let client = self.client.clone();
			let handle = self.handle.clone();
			let nonce = self.nonce + thread_index as Nonce;

			std::thread::spawn(move || {
				let mut version = handle.version();
				let mut seed_hash = Hash::default();
				let mut vm: Option<RandomXVm> = None;
				let mut error: Option<Error> = None;
				let mut is_new_vm = false;
				let mut is_build_changed = false;

				loop {
					if error.is_some() {
						match error.take().unwrap() {
							// on_major_syncing
							Error::EmptyMetadata => (),
							err =>
								warn!(target: LOG_TARGET, "Error in miner thread-{}: {:?}", thread_index, err),
						}
						std::thread::sleep(Duration::from_secs(1));
					}

					let mut nonce = nonce;

					let metadata = match handle.metadata() {
						Some(metadata) => metadata,
						None => {
							error = Some(Error::EmptyMetadata);
							continue;
						},
					};

					match common::seed_hash(&client, &BlockId::Hash(metadata.best_hash)) {
						Ok(new_seed_hash) =>
							if seed_hash != new_seed_hash {
								seed_hash = new_seed_hash;
								vm = None;
							},
						Err(_) => {
							error = Some(Error::SeedHashNotFetched);
							continue;
						},
					}

					if vm.is_none() {
						let dataset = match randomx::get_or_init_dataset(&seed_hash) {
							Ok(dataset) => dataset,
							Err(_) => {
								error = Some(Error::DatasetNotAllocated);
								continue;
							},
						};

						vm = match RandomXVm::new(
							randomx::get_flags() | RandomXFlags::FullMem,
							None,
							Some(dataset),
						) {
							Ok(vm) => Some(vm),
							Err(_) => {
								error = Some(Error::VmNotCreated);
								continue;
							},
						};
						is_new_vm = true;
						is_build_changed = false;
					}

					loop {
						let new_version = handle.version();
						if version != new_version {
							version = new_version;
							is_build_changed = true;
							break;
						}

						if is_new_vm {
							is_new_vm = false;
							vm.as_mut()
								.unwrap()
								.calculate_hash_first(&(metadata.pre_hash, nonce).encode());
						} else {
							let seal = common::Seal { nonce };
							if !is_build_changed {
								nonce += threads_count as Nonce;
							}

							let hash = Hash::from(
								vm.as_mut()
									.unwrap()
									.calculate_hash_next(&(metadata.pre_hash, nonce).encode()),
							);

							if !is_build_changed && common::check_hash(&hash, metadata.difficulty) {
								handle.submit(seal.encode());
							}
							is_build_changed = false;
						}
					}
				}
			});
		}
	}
}

pub struct MinerParams<C, H>
where
	C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + 'static,
	H: MiningHandle + Send + Sync + 'static,
{
	/// Client handle for fetching seed hash.
	pub client: Arc<C>,
	/// Mining handle for fetching metadata and submitting seal.
	pub handle: Arc<H>,
	/// Number of threads to use for mining.
	pub threads_count: usize,
}

pub fn start_miner<C, H>(params: MinerParams<C, H>)
where
	C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + 'static,
	H: MiningHandle + Send + Sync + 'static,
{
	let threads_count = params.threads_count;
	Miner::new(params).start(threads_count);
}
