// Copyright (c) The Hashcash Authors
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{preludes::*, Mainchain};

use futures::{Future, FutureExt};
use hashcash::{client::api::consensus::Seal, primitives::core::opaque::Header};
use log::*;
use parking_lot::RwLock;
use std::{error::Error, pin::Pin, sync::Arc};
use substrate::{
	codec::Encode,
	primitives::{
		consensus::{
			block_validation::{BlockAnnounceValidator as BlockAnnounceValidatorT, Validation},
			pow::POW_ENGINE_ID,
		},
		runtime::DigestItem,
	},
};

const LOG_TARGET: &str = "sync";

#[derive(Debug)]
pub struct BlockAnnounceValidator {
	mainchain: Arc<RwLock<Mainchain>>,
}

impl BlockAnnounceValidator {
	pub fn new(mainchain: Arc<RwLock<Mainchain>>) -> Self {
		Self { mainchain }
	}
}

impl BlockAnnounceValidatorT<Block> for BlockAnnounceValidator {
	fn validate(
		&mut self,
		header: &Header,
		data: &[u8],
	) -> Pin<Box<dyn Future<Output = Result<Validation, Box<dyn Error + Send>>> + Send>> {
		let is_empty = data.is_empty();
		let pre_digest = header
			.digest
			.logs()
			.iter()
			.find_map(|log| log.pre_runtime_try_to::<PreDigest>(&POW_ENGINE_ID));
		let seal = header
			.digest
			.logs()
			.iter()
			.find_map(|log| log.seal_try_to::<Seal>(&POW_ENGINE_ID));
		let mainchain = self.mainchain.clone();

		async move {
			if !is_empty {
				debug!(
					target: LOG_TARGET,
					"Received unknown data alongside the block announcement.",
				);
				Ok(Validation::Failure { disconnect: true })
			} else {
				let (miner_data, seal) = match (pre_digest, seal) {
					(Some((_, miner_data)), Some(seal)) => (miner_data, seal),
					_ => {
						warn!(
							target: LOG_TARGET,
							"Received a block announcement without a pre-digest or seal.",
						);
						return Ok(Validation::Failure { disconnect: true });
					},
				};

				if let Some(best_header) = mainchain.read().header(None) {
					if miner_data.block.header.number + 2 < best_header.number {
						let block = build_block(miner_data.block, seal);
						warn!(
							target: LOG_TARGET,
							"Received a block announcement for a stale block {} (mainchain: #{}, current: #{}).",
							block.header.hash(),
							best_header.number,
							block.header.number,
						);
						return Ok(Validation::Failure { disconnect: false });
					} else if miner_data.block.header.number > best_header.number + 2 {
						let block = build_block(miner_data.block, seal);
						warn!(
							target: LOG_TARGET,
							"Received a block announcement for a block ahead on mainchain {} (mainchain: #{}, current: #{}).",
							block.header.hash(),
							best_header.number,
							block.header.number,
						);
						return Ok(Validation::Failure { disconnect: false });
					}
				}
				Ok(Validation::Success { is_new_best: false })
			}
		}
		.boxed()
	}
}

fn build_block(mut block: Block, seal: Seal) -> Block {
	block.header.digest.push(DigestItem::Seal(POW_ENGINE_ID, seal.encode()));
	block
}
