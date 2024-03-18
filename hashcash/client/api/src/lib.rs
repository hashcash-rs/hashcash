// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

mod preludes;
use preludes::*;

pub use hashcash::primitives::core::{opaque::Block, Difficulty, Hash};
pub use substrate::{client::consensus::pow::Version, primitives::consensus::pow::Seal};

use serde::{Deserialize, Serialize};
use substrate::{
	client::consensus::{
		pow::{self, PowAlgorithm},
		BlockImport, JustificationSyncLink,
	},
	codec::{Decode, Encode},
	primitives::runtime::traits::Block as BlockT,
};

/// A type alias for MiningMetadata with the Hashcash hash and difficulty types.
pub type MiningMetadata = pow::MiningMetadata<Hash, Difficulty>;

/// A wrapper for [`sc_consensus_pow::MiningHandle`] that provides a simplified interface.
pub trait MiningHandle {
	/// Returns MiningMetadata. Can be None if the mining handle is not ready.
	fn metadata(&self) -> Option<MiningMetadata>;
	/// Submits a new seal.
	fn submit(&self, seal: Seal) -> bool;
	/// Returns the version of mining build.
	fn version(&self) -> Version;
}

impl<B, A, L, P, I> MiningHandle for pow::MiningHandle<B, A, L, P, I>
where
	B: BlockT<Hash = Hash>,
	A: PowAlgorithm<B, Difficulty = Difficulty>,
	L: JustificationSyncLink<B>,
	I: BlockImport<B>,
{
	fn metadata(&self) -> Option<MiningMetadata> {
		pow::MiningHandle::metadata(self)
	}

	fn submit(&self, seal: Seal) -> bool {
		futures::executor::block_on(pow::MiningHandle::submit(self, seal))
	}

	fn version(&self) -> Version {
		pow::MiningHandle::version(self)
	}
}

/// MiningWorker backend that handles metadata and submits a newly mined seal.
pub trait MiningWorkerBackend<Hash, Difficulty> {
	/// Returns the seed hash.
	fn seed_hash(&self) -> Option<Hash>;
	/// Returns the pre-hash of inner block template.
	fn pre_hash(&self) -> Hash;
	/// Returns the target mining difficulty.
	fn difficulty(&self) -> Difficulty;
	/// Returns the version of inner mining metadata.
	///
	/// When the version is changed, the inner mining metadata should be updated by calling
	/// [`bump()`].
	fn version(&self) -> Version;
	/// Submits a new seal.
	fn submit(&self, work: Hash, seal: Seal) -> bool;
	/// Updates the inner mining metadata to a new version.
	fn bump(&mut self) -> bool;
}

/// Mining metadata for remote miners.
#[derive(Clone, Encode, Decode, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct MinerData {
	/// A block template to be mined.
	pub block: Block,
	/// The target mining difficulty.
	pub difficulty: Difficulty,
	/// The seed hash for the mining algorithm.
	pub seed_hash: Hash,
}

/// A struct for submitting a new block seal.
#[derive(Decode, Encode)]
pub struct BlockSubmitParams<B> {
	/// A block template.
	pub block: B,
	/// The seal to be submitted.
	pub seal: Vec<u8>,
}
