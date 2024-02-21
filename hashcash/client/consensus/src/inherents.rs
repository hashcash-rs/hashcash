// Copyright (c) 2024 Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use hashcash::primitives::core::constants::SS58_PREFIX;
use log::*;
use substrate::primitives::{
	core::crypto::{Ss58AddressFormat, Ss58Codec},
	inherents::{self, InherentData, InherentIdentifier},
};

pub mod coinbase {
	use super::*;

	pub type InherentType = hashcash::primitives::coinbase::InherentType<AccountId, Difficulty>;

	pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"coinbase";

	pub struct InherentDataProvider {
		pub author: AccountId,
	}

	impl InherentDataProvider {
		pub fn new(author: AccountId) -> Self {
			Self { author }
		}

		pub fn print_author(&self) {
			info!(
				target: LOG_TARGET,
				"Miner address is: {}",
				self.author.to_ss58check_with_version(Ss58AddressFormat::custom(SS58_PREFIX))
			);
		}
	}

	#[async_trait::async_trait]
	impl inherents::InherentDataProvider for InherentDataProvider {
		async fn provide_inherent_data(
			&self,
			inherent_data: &mut InherentData,
		) -> Result<(), inherents::Error> {
			inherent_data.put_data(
				INHERENT_IDENTIFIER,
				&InherentType::from([(self.author.clone(), 1 as Difficulty)]),
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
}
