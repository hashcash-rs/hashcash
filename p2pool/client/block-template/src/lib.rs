// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

mod error;
mod preludes;
mod provider;
mod worker;

pub const LOG_TARGET: &str = "block-template";
pub const STORAGE_KEY: &[u8] = b"block_template";

use error::BlockTemplateError;
use preludes::substrate::client::api::backend::AuxStore;
pub use provider::BlockTemplateProvider;
use std::sync::Arc;
use worker::BlockTemplateSyncWorker;

pub fn start_block_template_sync<C>(
	target_chain: String,
	client: Arc<C>,
) -> Result<(BlockTemplateSyncWorker<C>, BlockTemplateProvider<C>), BlockTemplateError>
where
	C: AuxStore + 'static,
{
	Ok((
		BlockTemplateSyncWorker::new(target_chain, client.clone())?,
		BlockTemplateProvider::new(client),
	))
}
