// Copyright (c) 2024 Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use sp_runtime::{traits::Block, ConsensusEngineId};

/// A trait that provides multiple pre-runtime digests for different consensus engines.
pub trait PreRuntimeProvider<B: Block> {
	/// Returns a set of pre-runtime digests.
	fn pre_runtime(&self, best_hash: &B::Hash) -> Vec<(ConsensusEngineId, Option<Vec<u8>>)>;
}
