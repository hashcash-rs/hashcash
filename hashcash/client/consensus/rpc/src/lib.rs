// Copyright (c) 2024 Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

mod preludes;
use preludes::*;

use hashcash::{
	client::consensus,
	primitives::core::{opaque::Block, Difficulty, Hash},
};
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use substrate::{
	client::{
		api::backend::AuxStore,
		consensus::{
			pow::{PowIntermediate, INTERMEDIATE_KEY},
			BlockImport, BlockImportParams, JustificationSyncLink, StateAction, StorageChanges,
		},
	},
	codec::{Decode, Encode},
	primitives::{
		api::{ApiError, ApiExt, CallApiAt, ProvideRuntimeApi},
		blockchain::HeaderBackend,
		consensus::{
			pow::{DifficultyApi, POW_ENGINE_ID},
			BlockOrigin,
		},
		core::{Bytes, H256},
		runtime::{
			generic::BlockId,
			traits::{Block as BlockT, Header},
			DigestItem,
		},
	},
};

#[derive(Debug, thiserror::Error)]
pub enum MinerError {
	#[error(transparent)]
	AuxStore(substrate::primitives::blockchain::Error),
	#[error(transparent)]
	Codec(substrate::codec::Error),
	#[error(transparent)]
	ConsensusPow(substrate::client::consensus::pow::Error<Block>),
	#[error(transparent)]
	RuntimeApi(#[from] ApiError),
	#[error("{0}")]
	StorageChanges(String),
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockTemplate {
	pub block: Block,
	pub difficulty: Difficulty,
	pub seed_hash: Hash,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq)]
pub struct BlockSubmitParams {
	pub block: Block,
	pub seal: Vec<u8>,
}

#[rpc(client, server)]
pub trait MinerApi {
	#[method(name = "miner_getBlockTemplate")]
	fn block_template(&self) -> RpcResult<Option<BlockTemplate>>;

	#[method(name = "miner_submitBlock")]
	fn submit_block(&self, block_submit_params: Bytes) -> RpcResult<Hash>;
}

pub struct Miner<C, I, L> {
	block_import: Arc<Mutex<I>>,
	client: Arc<C>,
	sync_link: Arc<L>,
}

impl<C, I, L> Miner<C, I, L>
where
	C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + CallApiAt<Block> + AuxStore,
	C::Api: DifficultyApi<Block, Difficulty> + ApiExt<Block>,
	I: BlockImport<Block>,
	L: JustificationSyncLink<Block>,
{
	pub fn new(client: Arc<C>, block_import: Arc<Mutex<I>>, sync_link: Arc<L>) -> Self {
		Self { client, block_import, sync_link }
	}

	pub fn block_template_inner(&self) -> Result<Option<BlockTemplate>, MinerError> {
		if let Some(value) = self
			.client
			.as_ref()
			.get_aux(consensus::STORAGE_KEY)
			.map_err(MinerError::AuxStore)?
		{
			let block = Block::decode(&mut &value[..]).map_err(MinerError::Codec)?;

			let parent_hash = *block.header().parent_hash();
			let seed_hash = consensus::seed_hash(&self.client, &BlockId::Hash(parent_hash))
				.map_err(MinerError::ConsensusPow)?;
			let difficulty = self
				.client
				.runtime_api()
				.difficulty(parent_hash)
				.map_err(MinerError::RuntimeApi)?;

			Ok(Some(BlockTemplate { block, difficulty, seed_hash }))
		} else {
			Ok(None)
		}
	}

	pub async fn submit_block_inner(&self, block_submit_params: Bytes) -> Result<H256, MinerError> {
		let block_submit_params =
			BlockSubmitParams::decode(&mut &block_submit_params[..]).map_err(MinerError::Codec)?;

		let (header, body) = block_submit_params.block.deconstruct();
		let mut import_block =
			BlockImportParams::new(BlockOrigin::NetworkBroadcast, header.clone());
		let seal = DigestItem::Seal(POW_ENGINE_ID, block_submit_params.seal);
		import_block.post_digests.push(seal);
		import_block.body = Some(body);

		let parent_hash = header.parent_hash();
		let state = self.client.state_at(*parent_hash).map_err(MinerError::RuntimeApi)?;
		let storage_changes = self
			.client
			.runtime_api()
			.into_storage_changes(&state, *parent_hash)
			.map_err(MinerError::StorageChanges)?;

		import_block.state_action =
			StateAction::ApplyChanges(StorageChanges::Changes(storage_changes));
		let difficulty = self
			.client
			.runtime_api()
			.difficulty(*parent_hash)
			.map_err(MinerError::RuntimeApi)?;

		let intermediate = PowIntermediate { difficulty: Some(difficulty) };
		import_block.insert_intermediate(INTERMEDIATE_KEY, intermediate);

		let header = import_block.post_header();
		let mut block_import = self.block_import.lock();
		match block_import.import_block(import_block).await {
			Ok(res) => {
				res.handle_justification(&header.hash(), *header.number(), &self.sync_link);
			},
			Err(_) => (),
		}
		Ok(header.hash())
	}
}

#[async_trait]
impl<C, I, L> MinerApiServer for Miner<C, I, L>
where
	C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + CallApiAt<Block> + AuxStore + 'static,
	C::Api: DifficultyApi<Block, Difficulty> + ApiExt<Block>,
	I: BlockImport<Block> + std::marker::Send + std::marker::Sync + 'static,
	L: JustificationSyncLink<Block> + 'static,
{
	fn block_template(&self) -> RpcResult<Option<BlockTemplate>> {
		Ok(self
			.block_template_inner()
			.map_err(|e| jsonrpsee::core::Error::Custom(e.to_string()))?)
	}

	fn submit_block(&self, block_submit_params: Bytes) -> RpcResult<H256> {
		Ok(futures::executor::block_on(self.submit_block_inner(block_submit_params))
			.map_err(|e| jsonrpsee::core::Error::Custom(e.to_string()))?)
	}
}
