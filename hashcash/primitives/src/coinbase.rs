// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: Apache-2.0

use crate::preludes::*;

#[cfg(feature = "std")]
use hashcash::primitives::core::{AccountId, Difficulty};
#[cfg(feature = "std")]
use substrate::codec::Decode;
use substrate::{
	codec::Encode,
	primitives::{
		inherents::{self, InherentData, InherentIdentifier, IsFatalError},
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

#[cfg(feature = "std")]
pub type InherentTypeImpl = BTreeMap<AccountId, Difficulty>;

#[cfg(feature = "std")]
pub struct InherentDataProvider {
	pub author: AccountId,
}

#[cfg(feature = "std")]
impl InherentDataProvider {
	pub fn new(author: AccountId) -> Self {
		Self { author }
	}
}

#[cfg(feature = "std")]
#[async_trait::async_trait]
impl inherents::InherentDataProvider for InherentDataProvider {
	async fn provide_inherent_data(
		&self,
		inherent_data: &mut InherentData,
	) -> Result<(), inherents::Error> {
		inherent_data.put_data(
			INHERENT_IDENTIFIER,
			&InherentTypeImpl::from([(self.author.clone(), 1 as Difficulty)]),
		)
	}

	async fn try_handle_error(
		&self,
		_identifier: &InherentIdentifier,
		_error: &[u8],
	) -> Option<Result<(), inherents::Error>> {
		None
	}
}
