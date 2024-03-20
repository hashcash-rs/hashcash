// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

pub use hashcash::client::api::MinerData;

use hashcash::{client::api::consensus, primitives::coinbase};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use substrate::{
	client::consensus::pow::PreRuntimeProvider,
	codec::{Decode, Encode},
	primitives::{
		api::ProvideRuntimeApi,
		blockchain::HeaderBackend,
		consensus::{
			pow::{DifficultyApi, POW_ENGINE_ID},
			Environment, Proposer, SelectChain,
		},
		inherents::{CreateInherentDataProviders, InherentDataProvider},
		runtime::{traits::Header, Digest, DigestItem},
	},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error(transparent)]
	Codec(substrate::codec::Error),
	#[error(transparent)]
	Consensus(substrate::primitives::consensus::Error),
	#[error(transparent)]
	ConsensusPow(substrate::client::consensus::pow::Error<Block>),
	#[error("Empty shares")]
	EmptyShares,
	#[error(transparent)]
	Inherents(substrate::primitives::inherents::Error),
	#[error(transparent)]
	RuntimeApi(#[from] substrate::primitives::api::ApiError),
	#[error("{0}")]
	Proposer(String),
	/// Some other error.
	#[error(transparent)]
	Other(#[from] Box<dyn std::error::Error + Sync + Send + 'static>),
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Serialize, Deserialize)]
pub struct MinerDataParams {
	/// A block author.
	pub author: AccountId,
	/// A set of cumulative miner shares.
	pub shares: Vec<(AccountId, Difficulty)>,
}

pub struct MinerDataBuilderParams<C, CIDP, PF, PP, S> {
	/// The client to interact with the chain.
	pub client: Arc<C>,
	/// Something that can create the inherent data providers.
	pub create_inherent_data_providers: CIDP,
	/// Pre-runtime digest to be inserted into blocks.
	pub pre_runtime_provider: PP,
	/// The proposer factory to build proposer instances.
	pub proposer_factory: Arc<Mutex<PF>>,
	/// A select chain implementation to select the best block.
	pub select_chain: S,
	/// Maximum time allowed for building a block.
	pub build_time: Duration,
}

pub struct MinerDataBuilder<C, CIDP, PF, PP, S> {
	client: Arc<C>,
	create_inherent_data_providers: CIDP,
	pre_runtime_provider: PP,
	proposer_factory: Arc<Mutex<PF>>,
	select_chain: S,
	build_time: Duration,
}

impl<C, CIDP, PF, PP, S> MinerDataBuilder<C, CIDP, PF, PP, S> {
	pub fn new(params: MinerDataBuilderParams<C, CIDP, PF, PP, S>) -> Self {
		let MinerDataBuilderParams {
			client,
			create_inherent_data_providers,
			pre_runtime_provider,
			proposer_factory,
			select_chain,
			build_time,
		} = params;

		Self {
			client,
			create_inherent_data_providers,
			pre_runtime_provider,
			proposer_factory,
			select_chain,
			build_time,
		}
	}
}

#[async_trait::async_trait]
impl<C, CIDP, PF, PP, S> crate::traits::MinerDataBuilder for MinerDataBuilder<C, CIDP, PF, PP, S>
where
	C: ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: DifficultyApi<Block, Difficulty>,
	CIDP: CreateInherentDataProviders<Block, ()>,
	PF: Environment<Block> + Send,
	PF::Error: std::fmt::Debug,
	PF::Proposer: Proposer<Block>,
	PP: PreRuntimeProvider<Block> + Send + Sync,
	S: SelectChain<Block>,
{
	type Params = MinerDataParams;

	async fn build(
		&self,
		MinerDataParams { author, shares }: Self::Params,
	) -> Result<MinerData, Error> {
		if shares.is_empty() {
			return Err(Error::EmptyShares);
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
		match self.pre_runtime_provider.pre_runtime(&best_hash).await {
			Ok(pre_runtimes) =>
				for (id, data) in pre_runtimes {
					inherent_digest.push(DigestItem::PreRuntime(id, data));
				},
			Err(e) => return Err(Error::ConsensusPow(e)),
		}
		inherent_digest.push(DigestItem::PreRuntime(POW_ENGINE_ID, author.encode()));

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
		let parent_hash = proposal.block.header.parent_hash();
		let seed_hash = consensus::seed_hash(&self.client, &BlockId::Hash(*parent_hash))
			.map_err(Error::ConsensusPow)?;
		let difficulty =
			self.client.runtime_api().difficulty(*parent_hash).map_err(Error::RuntimeApi)?;

		Ok(MinerData { block: proposal.block, difficulty, seed_hash })
	}
}
