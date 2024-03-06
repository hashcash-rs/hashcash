// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use std::sync::Arc;
use substrate::{
	client::{api::HeaderBackend, consensus::pow::Error},
	codec::{Decode, Encode},
};

#[derive(Debug, Decode, Encode)]
pub struct Seal {
	pub nonce: Nonce,
}

/// Checks if a hash fits the given difficulty.
pub fn check_hash(hash: &Hash, difficulty: Difficulty) -> bool {
	let hash = U256::from(&hash[..]);
	let (_, overflowed) = hash.overflowing_mul(difficulty.into());

	!overflowed
}

/// Returns a block number for retrieving the seed hash.
pub fn seed_height(height: BlockNumber) -> BlockNumber {
	const SEEDHASH_EPOCH_BLOCKS: BlockNumber = 2048;
	const SEEDHASH_EPOCH_LAG: BlockNumber = 64;

	if height <= SEEDHASH_EPOCH_BLOCKS + SEEDHASH_EPOCH_LAG {
		return 0;
	}

	(height - SEEDHASH_EPOCH_LAG - 1) & !(SEEDHASH_EPOCH_BLOCKS - 1)
}

/// Returns a seed hash for VM initialization with the given block number or hash.
pub fn seed_hash<C>(client: &Arc<C>, parent: &BlockId) -> Result<Hash, Error<Block>>
where
	C: HeaderBackend<Block>,
{
	let parent_number = client
		.block_number_from_id(parent)
		.map_err(Error::Client)?
		.ok_or(Error::Environment(format!("Block number not found: {:?}", parent)))?;

	let seed_height = seed_height(parent_number);

	client
		.hash(seed_height)
		.map_err(Error::Client)?
		.ok_or(Error::Environment(format!("Block hash not found: {:?}", seed_height)))
}
