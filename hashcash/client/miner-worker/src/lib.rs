// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

mod preludes;
use preludes::*;

use hashcash::{
	client::{
		api::MiningWorkerBackend,
		consensus::{self, randomx},
	},
	randomx::{RandomXFlags, RandomXVm},
};
use log::*;
use rand::{thread_rng, Rng};
use std::time::Duration;
use substrate::codec::Encode;

#[derive(Debug)]
enum Error {
	MetadataNotReady,
	SeedHashNotFetched,
	DatasetNotAllocated,
	VmNotCreated,
}

pub struct MiningWorker<B> {
	backend: B,
	nonce: Nonce,
}

impl<B> MiningWorker<B>
where
	B: MiningWorkerBackend<Hash, Difficulty> + Clone + Send + Sync + 'static,
{
	pub fn new(backend: B) -> Self {
		MiningWorker { backend, nonce: thread_rng().gen() }
	}

	pub fn start(&self, threads_count: usize) {
		let threads_count = threads_count.max(1);
		info!(target: LOG_TARGET, "⚒️  Starting MiningWorker with {} thread(s)", threads_count);

		for thread_index in 0..threads_count {
			let mut backend = self.backend.clone();
			let nonce = self.nonce + thread_index as Nonce;

			std::thread::spawn(move || {
				let mut version = backend.version();
				let mut seed_hash = Hash::default();
				let mut vm: Option<RandomXVm> = None;
				let mut error: Option<Error> = None;
				let mut is_new_vm = false;
				let mut is_build_changed = false;

				loop {
					if error.is_some() {
						match error.take().unwrap() {
							// on_major_syncing
							Error::MetadataNotReady => (),
							err =>
								warn!(target: LOG_TARGET, "error: mining-worker({}): {:?}", thread_index, err),
						}
						std::thread::sleep(Duration::from_secs(1));
					}

					let mut nonce = nonce;

					if !backend.bump() {
						error = Some(Error::MetadataNotReady);
						continue;
					}

					match backend.seed_hash() {
						Some(new_seed_hash) =>
							if seed_hash != new_seed_hash {
								seed_hash = new_seed_hash;
								vm = None;
							},
						None => {
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
						let new_version = backend.version();
						if version != new_version {
							version = new_version;
							is_build_changed = true;
							break;
						}

						if is_new_vm {
							is_new_vm = false;
							vm.as_mut()
								.unwrap()
								.calculate_hash_first(&(backend.pre_hash(), nonce).encode());
						} else {
							let seal = consensus::Seal { nonce };
							if !is_build_changed {
								nonce += threads_count as Nonce;
							}

							let hash = Hash::from(
								vm.as_mut()
									.unwrap()
									.calculate_hash_next(&(backend.pre_hash(), nonce).encode()),
							);

							if !is_build_changed &&
								consensus::check_hash(&hash, backend.difficulty())
							{
								if !backend.submit(hash, seal.encode()) {
									warn!(target: LOG_TARGET, "error: mining-worker({}): failed to submit seal", thread_index);
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
