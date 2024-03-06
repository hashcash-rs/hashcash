// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{common, preludes::*, randomx};

use std::sync::Arc;
use substrate::{
	client::{
		api::{AuxStore, HeaderBackend},
		consensus::pow::{Error, PowAlgorithm},
	},
	codec::{Decode, Encode},
	primitives::{
		api::ProvideRuntimeApi,
		consensus::pow::{DifficultyApi, Seal},
	},
};

pub struct RandomXAlgorithm<C> {
	client: Arc<C>,
}

impl<C> RandomXAlgorithm<C> {
	pub fn new(client: Arc<C>) -> Self {
		RandomXAlgorithm { client }
	}
}

impl<C> Clone for RandomXAlgorithm<C> {
	fn clone(&self) -> Self {
		RandomXAlgorithm { client: self.client.clone() }
	}
}

impl<C> PowAlgorithm<Block> for RandomXAlgorithm<C>
where
	C: HeaderBackend<Block> + AuxStore + ProvideRuntimeApi<Block>,
	C::Api: DifficultyApi<Block, Difficulty>,
{
	type Difficulty = Difficulty;

	fn difficulty(&self, parent: Hash) -> Result<Difficulty, Error<Block>> {
		self.client
			.runtime_api()
			.difficulty(parent)
			.map_err(|e| Error::Client(e.into()))
	}

	fn verify(
		&self,
		parent: &BlockId,
		pre_hash: &Hash,
		_pre_digest: Option<&[u8]>,
		seal: &Seal,
		difficulty: Difficulty,
	) -> Result<bool, Error<Block>> {
		let seal = match common::Seal::decode(&mut &seal[..]) {
			Ok(seal) => seal,
			Err(_) => return Ok(false),
		};
		let seed_hash = common::seed_hash(&self.client, parent)?;

		let work = randomx::calculate_hash(&seed_hash, (pre_hash, seal.nonce).encode().as_slice())
			.map_err(|_| Error::Environment("Failed to calculate a RandomX hash".to_string()))?;

		Ok(common::check_hash(&work, difficulty))
	}
}
