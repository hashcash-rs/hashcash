// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

mod error;
mod preludes;
mod provider;
mod worker;

pub const LOG_TARGET: &str = "block-template";
pub const STORAGE_KEY: &[u8] = b"block_template";

use error::BlockTemplateError;
use preludes::{
	hashcash::primitives::core::AccountId,
	substrate::{
		client::api::{backend::AuxStore, BlockchainEvents},
		primitives::{
			blockchain::HeaderBackend,
			consensus::{SelectChain, SyncOracle},
			runtime::traits::{Block, NumberFor},
		},
	},
};
pub use provider::BlockTemplateProvider;
use std::{sync::Arc, time::Duration};
use worker::BlockTemplateSyncWorker;

pub fn start_block_template_sync<B, C, S, SO>(
	target_chain: String,
	client: Arc<C>,
	select_chain: S,
	author: AccountId,
	genesis_hash: B::Hash,
	window_size: NumberFor<B>,
	sync_oracle: SO,
	timeout: Duration,
) -> Result<(BlockTemplateSyncWorker<B, C, S, SO>, BlockTemplateProvider<C>), BlockTemplateError>
where
	B: Block,
	C: AuxStore + BlockchainEvents<B> + HeaderBackend<B> + 'static,
	S: SelectChain<B>,
	SO: SyncOracle,
{
	Ok((
		BlockTemplateSyncWorker::new(
			target_chain,
			client.clone(),
			select_chain,
			author,
			genesis_hash,
			window_size,
			sync_oracle,
			timeout,
		)?,
		BlockTemplateProvider::new(client),
	))
}
