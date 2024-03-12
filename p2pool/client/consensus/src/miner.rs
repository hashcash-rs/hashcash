// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use futures::channel::mpsc;
use hashcash::{
	client::consensus::{self, randomx, MiningHandle},
	primitives::{
		block_template::{BlockSubmitParams, BlockTemplate},
		core::AccountId,
	},
	randomx::{RandomXFlags, RandomXVm},
};
use log::*;
use parity_scale_codec::Decode;
use rand::{thread_rng, Rng};
use std::{sync::Arc, time::Duration};
pub use substrate::{
	client::{api::HeaderBackend, consensus::pow::PowAlgorithm},
	codec::Encode,
	primitives::{api::ProvideRuntimeApi, core::Bytes, runtime::traits::Block as BlockT},
};

pub struct Miner<B: BlockT, C, H> {
	_client: Arc<C>,
	handle: Arc<H>,
	nonce: Nonce,
	submitter: mpsc::UnboundedSender<Bytes>,
	_marker: std::marker::PhantomData<B>,
}

#[derive(Debug)]
enum Error {
	EmptyMetadata,
	InvalidPreRuntime,
	DatasetNotAllocated,
	VmNotCreated,
}

impl<C, H> Miner<Block, C, H>
where
	C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + 'static,
	H: MiningHandle + Send + Sync + 'static,
{
	pub fn new(client: Arc<C>, handle: Arc<H>, submitter: mpsc::UnboundedSender<Bytes>) -> Self {
		Miner {
			_client: client,
			handle,
			nonce: thread_rng().gen(),
			submitter,
			_marker: Default::default(),
		}
	}

	pub fn start(&self, threads_count: usize) {
		let threads_count = threads_count.max(1);
		info!(target: LOG_TARGET, "⚒️  Starting Miner with {} thread(s)", threads_count);

		for thread_index in 0..threads_count {
			let handle = self.handle.clone();
			let nonce = self.nonce + thread_index as Nonce;

			let submitter = self.submitter.clone();
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
							err => {
								warn!(target: LOG_TARGET, "Error in miner thread-{}: {:?}", thread_index, err)
							},
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

					let block_template = match metadata
						.pre_runtime
						.map(|v| <(AccountId, Option<BlockTemplate>)>::decode(&mut &v[..]))
					{
						Some(Ok((_, Some(block_template)))) => block_template,
						_ => {
							error = Some(Error::InvalidPreRuntime);
							handle.reset();
							continue;
						},
					};

					if seed_hash != block_template.seed_hash {
						seed_hash = block_template.seed_hash;
						vm = None;
					}
					let pre_hash = block_template.block.hash();

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
							vm.as_mut().unwrap().calculate_hash_first(&(pre_hash, nonce).encode());
						} else {
							let seal = consensus::Seal { nonce };
							if !is_build_changed {
								nonce += threads_count as Nonce;
							}

							let hash = Hash::from(
								vm.as_mut()
									.unwrap()
									.calculate_hash_next(&(pre_hash, nonce).encode()),
							);

							if !is_build_changed &&
								consensus::check_hash(&hash, metadata.difficulty)
							{
								handle.submit(seal.encode());

								if consensus::check_hash(&hash, block_template.difficulty) {
									let params = BlockSubmitParams {
										block: block_template.clone().block,
										seal: seal.encode(),
									};
									if let Err(e) = submitter.unbounded_send(params.to_bytes()) {
										warn!(target: LOG_TARGET, "Failed to submit block: {:?}", e);
									}
								}
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

	pub submitter: mpsc::UnboundedSender<Bytes>,
}

pub fn start_miner<C, H>(params: MinerParams<C, H>)
where
	C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + 'static,
	H: MiningHandle + Send + Sync + 'static,
{
	Miner::new(params.client, params.handle, params.submitter).start(params.threads_count);
}
