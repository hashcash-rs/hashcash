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
		client::api::AuxStore,
		primitives::{
			blockchain::HeaderBackend,
			consensus::SelectChain,
			runtime::traits::{Block, NumberFor},
		},
	},
};
pub use provider::BlockTemplateProvider;
use std::sync::Arc;
use worker::BlockTemplateSyncWorker;

pub fn start_block_template_sync<B, C, S>(
	target_chain: String,
	client: Arc<C>,
	select_chain: S,
	author: AccountId,
	genesis_hash: B::Hash,
	window_size: NumberFor<B>,
) -> Result<(BlockTemplateSyncWorker<B, C, S>, BlockTemplateProvider<C>), BlockTemplateError>
where
	B: Block,
	C: AuxStore + HeaderBackend<B> + 'static,
	S: SelectChain<B>,
{
	Ok((
		BlockTemplateSyncWorker::new(
			target_chain,
			client.clone(),
			select_chain,
			author,
			genesis_hash,
			window_size,
		)?,
		BlockTemplateProvider::new(client),
	))
}
