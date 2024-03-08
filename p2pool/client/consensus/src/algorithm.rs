// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use hashcash::{
	client::consensus::{self, randomx},
	primitives::{block_template::BlockTemplate, core::AccountId},
};
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
		runtime::traits::Block as BlockT,
	},
};

pub struct P2PoolAlgorithm<C> {
	client: Arc<C>,
}

impl<C> P2PoolAlgorithm<C> {
	pub fn new(client: Arc<C>) -> Self {
		Self { client }
	}
}

impl<C> Clone for P2PoolAlgorithm<C> {
	fn clone(&self) -> Self {
		Self { client: self.client.clone() }
	}
}

impl<C> PowAlgorithm<Block> for P2PoolAlgorithm<C>
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
		_parent: &BlockId,
		_pre_hash: &Hash,
		pre_digest: Option<&[u8]>,
		seal: &Seal,
		difficulty: Difficulty,
	) -> Result<bool, Error<Block>> {
		let block_template = pre_digest
			.map(|v| <(AccountId, Option<BlockTemplate>)>::decode(&mut &v[..]))
			.ok_or(Error::Other("Unable to verify: pre-digest not set".to_string()))?
			.map_err(|e| Error::Other(e.to_string()))?
			.1
			.ok_or(Error::Other("Unable to verify: block template not set".to_string()))?;

		let seal = match consensus::Seal::decode(&mut &seal[..]) {
			Ok(seal) => seal,
			Err(_) => return Ok(false),
		};

		let work = randomx::calculate_hash(
			&block_template.seed_hash,
			(block_template.block.hash(), seal.nonce).encode().as_slice(),
		)
		.map_err(|_| Error::Environment("Failed to calculate a RandomX hash".to_string()))?;

		Ok(consensus::check_hash(&work, difficulty))
	}
}
