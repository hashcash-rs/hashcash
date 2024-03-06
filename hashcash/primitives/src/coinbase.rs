// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "std")]
use parity_scale_codec::Decode;
use parity_scale_codec::Encode;

use sp_inherents::{InherentIdentifier, IsFatalError};
use sp_runtime::sp_std::collections::btree_map::BTreeMap;

pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"coinbase";

#[derive(Encode, sp_runtime::RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Decode, thiserror::Error))]
pub enum InherentError {}

impl IsFatalError for InherentError {
	fn is_fatal_error(&self) -> bool {
		true
	}
}

pub type InherentType<AccountId, Weight> = BTreeMap<AccountId, Weight>;
