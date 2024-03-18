// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

pub use hashcash::client::api::BlockSubmitParams;

use parking_lot::Mutex;
use std::sync::Arc;
use substrate::{
	client::consensus::{
		pow::{PowIntermediate, INTERMEDIATE_KEY},
		BlockImport, BlockImportParams, JustificationSyncLink, StateAction, StorageChanges,
	},
	codec::Error as CodecError,
	primitives::{
		api::{ApiError, ApiExt, CallApiAt, Core, ProvideRuntimeApi},
		blockchain::HeaderBackend,
		consensus::{
			pow::{DifficultyApi, POW_ENGINE_ID},
			BlockOrigin,
		},
		runtime::{
			traits::{Block as BlockT, Header},
			DigestItem,
		},
	},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error(transparent)]
	Codec(CodecError),
	#[error(transparent)]
	RuntimeApi(#[from] ApiError),
	#[error("{0}")]
	StorageChanges(String),
}

pub struct BlockSubmit<C, I, L> {
	client: Arc<C>,
	block_import: Arc<Mutex<I>>,
	justification_sync_link: L,
}

impl<C, I, L> BlockSubmit<C, I, L> {
	pub fn new(client: Arc<C>, block_import: I, justification_sync_link: L) -> Self {
		Self { client, block_import: Arc::new(Mutex::new(block_import)), justification_sync_link }
	}
}

#[async_trait::async_trait]
impl<C, I, L> crate::traits::BlockSubmit<Block> for BlockSubmit<C, I, L>
where
	C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + CallApiAt<Block>,
	C::Api: DifficultyApi<Block, Difficulty> + ApiExt<Block>,
	I: BlockImport<Block> + Send,
	L: JustificationSyncLink<Block>,
{
	async fn submit_block(&self, params: BlockSubmitParams<Block>) -> Result<Hash, Error> {
		let BlockSubmitParams { block, seal } = params;
		let (header, body) = block.clone().deconstruct();

		// CallApiAt::StateBackend doesn't implement Send, so we need to drop it before await.
		let import_block = {
			let mut import_block =
				BlockImportParams::new(BlockOrigin::NetworkBroadcast, header.clone());
			let seal = DigestItem::Seal(POW_ENGINE_ID, seal);
			import_block.post_digests.push(seal);
			import_block.body = Some(body);

			let parent_hash = header.parent_hash();
			let api = self.client.runtime_api();
			api.execute_block(*parent_hash, block)?;

			let state = self.client.state_at(*parent_hash).map_err(Error::RuntimeApi)?;
			let storage_changes =
				api.into_storage_changes(&state, *parent_hash).map_err(Error::StorageChanges)?;
			import_block.state_action =
				StateAction::ApplyChanges(StorageChanges::Changes(storage_changes));

			let difficulty =
				self.client.runtime_api().difficulty(*parent_hash).map_err(Error::RuntimeApi)?;
			let intermediate = PowIntermediate { difficulty: Some(difficulty) };
			import_block.insert_intermediate(INTERMEDIATE_KEY, intermediate);

			import_block
		};

		let header = import_block.post_header();
		let mut block_import = self.block_import.lock();

		if let Ok(res) = block_import.import_block(import_block).await {
			res.handle_justification(
				&header.hash(),
				*header.number(),
				&self.justification_sync_link,
			);
		}
		Ok(header.hash())
	}
}
