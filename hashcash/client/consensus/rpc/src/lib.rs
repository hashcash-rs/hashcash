// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

mod preludes;
use preludes::*;

mod error;
use error::Error;

use hashcash::{
	client::consensus,
	primitives::{
		block_template::{BlockSubmitParams, BlockTemplate},
		coinbase,
		core::{opaque::Block, AccountId, Difficulty, Hash},
	},
};
use jsonrpsee::{core::async_trait, proc_macros::rpc};
use parking_lot::Mutex;
use std::{sync::Arc, time::Duration};
use substrate::{
	client::consensus::{
		pow::{PowIntermediate, PreRuntimeProvider, INTERMEDIATE_KEY},
		BlockImport, BlockImportParams, JustificationSyncLink, StateAction, StorageChanges,
	},
	codec::Decode,
	primitives::{
		api::{ApiExt, CallApiAt, ProvideRuntimeApi},
		blockchain::HeaderBackend,
		consensus::{
			pow::{DifficultyApi, POW_ENGINE_ID},
			BlockOrigin, Environment, Proposer, SelectChain,
		},
		core::{Bytes, H256},
		inherents::{CreateInherentDataProviders, InherentDataProvider},
		runtime::{
			generic::{BlockId, Digest},
			traits::{Block as BlockT, Header},
			DigestItem,
		},
	},
};

#[rpc(client, server)]
pub trait MinerApi {
	#[method(name = "miner_getBlockTemplate")]
	fn block_template(&self, shares: Vec<(AccountId, Difficulty)>) -> Result<BlockTemplate, Error>;

	#[method(name = "miner_submitBlock")]
	fn submit_block(&self, block_submit_params: Bytes) -> Result<Hash, Error>;
}

pub struct Miner<C, CIDP, I, L, PF, PP, S> {
	block_import: Arc<Mutex<I>>,
	build_time: Duration,
	client: Arc<C>,
	create_inherent_data_providers: CIDP,
	justification_sync_link: Arc<L>,
	pre_runtime_provider: PP,
	proposer_factory: Arc<Mutex<PF>>,
	select_chain: S,
}

impl<C, CIDP, I, L, PF, PP, S> Miner<C, CIDP, I, L, PF, PP, S>
where
	C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + CallApiAt<Block>,
	C::Api: DifficultyApi<Block, Difficulty> + ApiExt<Block>,
	CIDP: CreateInherentDataProviders<Block, ()>,
	I: BlockImport<Block>,
	L: JustificationSyncLink<Block>,
	PF: Environment<Block>,
	PF::Error: std::fmt::Debug,
	PF::Proposer: Proposer<Block>,
	PP: PreRuntimeProvider<Block>,
	S: SelectChain<Block>,
{
	pub fn new(
		block_import: Arc<Mutex<I>>,
		build_time: Duration,
		client: Arc<C>,
		create_inherent_data_providers: CIDP,
		justification_sync_link: Arc<L>,
		pre_runtime_provider: PP,
		proposer_factory: Arc<Mutex<PF>>,
		select_chain: S,
	) -> Self {
		Self {
			block_import,
			build_time,
			client,
			create_inherent_data_providers,
			justification_sync_link,
			pre_runtime_provider,
			proposer_factory,
			select_chain,
		}
	}

	pub async fn block_template_inner(
		&self,
		shares: Vec<(AccountId, Difficulty)>,
	) -> Result<BlockTemplate, Error> {
		if shares.is_empty() {
			return Err(Error::EmptyShares("Empty shares".to_string()));
		}
		let best_header = self.select_chain.best_chain().await.map_err(Error::Consensus)?;
		let best_hash = best_header.hash();

		let inherent_data_providers = self
			.create_inherent_data_providers
			.create_inherent_data_providers(best_hash, ())
			.await
			.map_err(Error::Other)?;
		let mut inherent_data =
			inherent_data_providers.create_inherent_data().await.map_err(Error::Inherents)?;
		let _ = inherent_data.put_data(coinbase::INHERENT_IDENTIFIER, &shares);

		let mut inherent_digest = Digest::default();
		for (id, data) in self.pre_runtime_provider.pre_runtime(&best_hash) {
			if let Some(data) = data {
				inherent_digest.push(DigestItem::PreRuntime(id, data));
			}
		}

		let proposer = self.proposer_factory.lock().init(&best_header).await.map_err(|e| {
			Error::Proposer(format!(
				"Unable to propose new block for authoring. Creating proposer failed: {:?}",
				e
			))
		})?;
		let proposal = proposer
			.propose(inherent_data, inherent_digest, self.build_time, None)
			.await
			.map_err(|e| {
				Error::Proposer(format!(
					"Unable to propose new block for authoring. Creating proposal failed: {}",
					e,
				))
			})?;
		let parent_hash = proposal.block.header().parent_hash();
		let seed_hash = consensus::seed_hash(&self.client, &BlockId::Hash(*parent_hash))
			.map_err(Error::ConsensusPow)?;
		let difficulty =
			self.client.runtime_api().difficulty(*parent_hash).map_err(Error::RuntimeApi)?;

		Ok(BlockTemplate { block: proposal.block, difficulty, seed_hash })
	}

	pub async fn submit_block_inner(&self, block_submit_params: Bytes) -> Result<H256, Error> {
		let block_submit_params =
			BlockSubmitParams::decode(&mut &block_submit_params[..]).map_err(Error::Codec)?;

		let (header, body) = block_submit_params.block.deconstruct();
		let mut import_block =
			BlockImportParams::new(BlockOrigin::NetworkBroadcast, header.clone());
		let seal = DigestItem::Seal(POW_ENGINE_ID, block_submit_params.seal);
		import_block.post_digests.push(seal);
		import_block.body = Some(body);

		let parent_hash = header.parent_hash();
		let state = self.client.state_at(*parent_hash).map_err(Error::RuntimeApi)?;
		let storage_changes = self
			.client
			.runtime_api()
			.into_storage_changes(&state, *parent_hash)
			.map_err(Error::StorageChanges)?;

		import_block.state_action =
			StateAction::ApplyChanges(StorageChanges::Changes(storage_changes));
		let difficulty =
			self.client.runtime_api().difficulty(*parent_hash).map_err(Error::RuntimeApi)?;

		let intermediate = PowIntermediate { difficulty: Some(difficulty) };
		import_block.insert_intermediate(INTERMEDIATE_KEY, intermediate);

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

#[async_trait]
impl<C, CIDP, I, L, PF, PP, S> MinerApiServer for Miner<C, CIDP, I, L, PF, PP, S>
where
	C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + CallApiAt<Block> + 'static,
	C::Api: DifficultyApi<Block, Difficulty> + ApiExt<Block>,
	CIDP: CreateInherentDataProviders<Block, ()> + 'static,
	I: BlockImport<Block> + Send + Sync + 'static,
	L: JustificationSyncLink<Block> + 'static,
	PF: Environment<Block> + Send + Sync + 'static,
	PF::Error: std::fmt::Debug,
	PF::Proposer: Proposer<Block>,
	PP: PreRuntimeProvider<Block> + Send + Sync + 'static,
	S: SelectChain<Block> + 'static,
{
	fn block_template(&self, shares: Vec<(AccountId, Difficulty)>) -> Result<BlockTemplate, Error> {
		futures::executor::block_on(self.block_template_inner(shares))
	}

	fn submit_block(&self, block_submit_params: Bytes) -> Result<H256, Error> {
		futures::executor::block_on(self.submit_block_inner(block_submit_params))
	}
}
