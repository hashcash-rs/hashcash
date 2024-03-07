// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: Apache-2.0

use crate::preludes::*;

#[cfg(feature = "std")]
use substrate::codec::Decode;
use substrate::{
	codec::Encode,
	primitives::{
		inherents::{InherentIdentifier, IsFatalError},
		std::collections::btree_map::BTreeMap,
	},
};

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
