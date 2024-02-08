// Copyright (c) 2024 Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use hashcash::randomx::{
	Error as RandomXError, RandomXCache, RandomXDataset, RandomXFlags, RandomXVm,
};
use parking_lot::Mutex;
use schnellru::{ByLength, LruMap};
use std::{
	cell::RefCell,
	sync::{Arc, OnceLock},
};

static CACHES: OnceLock<Arc<Mutex<LruMap<Hash, Arc<RandomXCache>>>>> = OnceLock::new();
static DATASETS: OnceLock<Arc<Mutex<LruMap<Hash, Arc<RandomXDataset>>>>> = OnceLock::new();

pub struct CachedVm {
	pub seed_hash: Hash,
	pub vm: RandomXVm,
}

thread_local! {
	static FAST_VM: RefCell<Option<CachedVm>> = RefCell::new(None);
	static LIGHT_VM: RefCell<Option<CachedVm>> = RefCell::new(None);
}

#[derive(Debug)]
pub enum Error {
	DatasetNotFound,
	RandomXError(RandomXError),
}

impl From<RandomXError> for Error {
	fn from(e: RandomXError) -> Self {
		Error::RandomXError(e)
	}
}

pub(crate) fn get_flags() -> RandomXFlags {
	RandomXFlags::default()
}

pub(crate) fn get_or_init_cache(seed_hash: &Hash) -> Result<Arc<RandomXCache>, Error> {
	let shared_caches = CACHES.get_or_init(|| Arc::new(Mutex::new(LruMap::new(ByLength::new(3)))));
	let mut shared_caches = shared_caches.lock();

	if let Some(cache) = shared_caches.get(seed_hash) {
		Ok(cache.clone())
	} else {
		let mut cache = RandomXCache::new(get_flags())?;
		cache.init(&seed_hash[..]);

		let cache = Arc::new(cache);
		shared_caches.insert(*seed_hash, cache.clone());
		Ok(cache)
	}
}

pub(crate) fn get_dataset(seed_hash: &Hash) -> Result<Option<Arc<RandomXDataset>>, Error> {
	let shared_datasets =
		DATASETS.get_or_init(|| Arc::new(Mutex::new(LruMap::new(ByLength::new(2)))));

	Ok(shared_datasets.lock().get(seed_hash).cloned())
}

pub(crate) fn get_or_init_dataset(seed_hash: &Hash) -> Result<Arc<RandomXDataset>, Error> {
	let shared_datasets =
		DATASETS.get_or_init(|| Arc::new(Mutex::new(LruMap::new(ByLength::new(2)))));
	let mut shared_datasets = shared_datasets.lock();

	if let Some(dataset) = shared_datasets.get(seed_hash) {
		Ok(dataset.clone())
	} else {
		let cache = get_or_init_cache(seed_hash)?;

		let mut dataset = RandomXDataset::new(get_flags())?;
		dataset.init(&cache);

		let dataset = Arc::new(dataset);
		shared_datasets.insert(*seed_hash, dataset.clone());
		Ok(dataset)
	}
}

pub fn calculate_hash(seed_hash: &Hash, input: &[u8]) -> Result<Hash, Error> {
	match FAST_VM.with_borrow_mut(|cached| match cached {
		Some(cached) if &cached.seed_hash == seed_hash =>
			Ok::<_, Error>(Hash::from(cached.vm.calculate_hash(input))),
		_ => match get_dataset(seed_hash)? {
			Some(dataset) => {
				let mut vm =
					RandomXVm::new(get_flags() | RandomXFlags::FullMem, None, Some(dataset))?;
				let hash = Hash::from(vm.calculate_hash(input));
				*cached = Some(CachedVm { seed_hash: *seed_hash, vm });
				Ok(hash)
			},
			None => Err(Error::DatasetNotFound),
		},
	}) {
		Ok(hash) => Ok(hash),
		Err(_) => LIGHT_VM.with_borrow_mut(|cached| match cached {
			Some(cached) if &cached.seed_hash == seed_hash =>
				Ok(Hash::from(cached.vm.calculate_hash(input))),
			_ => {
				let cache = get_or_init_cache(seed_hash)?;
				let mut vm = RandomXVm::new(get_flags(), Some(cache), None)?;
				let hash = Hash::from(vm.calculate_hash(input));
				*cached = Some(CachedVm { seed_hash: *seed_hash, vm });
				Ok(hash)
			},
		}),
	}
}
