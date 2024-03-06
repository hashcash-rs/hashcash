// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use hashcash::{
	client::consensus::rpc::{Miner, MinerApiServer},
	primitives::core::{opaque::Block, AccountId, AccountNonce, Balance, Difficulty},
};
use jsonrpsee::RpcModule;
use parking_lot::Mutex;
use std::{error::Error, sync::Arc, time::Duration};
use substrate::{
	client::{
		api::AuxStore,
		consensus::{pow::PreRuntimeProvider, BlockImport, JustificationSyncLink},
		rpc::api::DenyUnsafe,
		transaction_pool::api::TransactionPool,
	},
	frames::system::rpc::AccountNonceApi,
	pallets::transaction_payment::rpc::TransactionPaymentRuntimeApi,
	primitives::{
		api::{CallApiAt, ProvideRuntimeApi},
		block_builder::BlockBuilder,
		blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata},
		consensus::{pow::DifficultyApi, Environment, Proposer, SelectChain},
		inherents::CreateInherentDataProviders,
	},
};

pub struct FullDeps<C, CIDP, I, L, P, PF, PP, S> {
	pub client: Arc<C>,
	pub pool: Arc<P>,
	pub block_import: Arc<Mutex<I>>,
	pub justification_sync_link: Arc<L>,
	pub deny_unsafe: DenyUnsafe,
	pub build_time: Duration,
	pub create_inherent_data_providers: CIDP,
	pub pre_runtime_provider: PP,
	pub proposer_factory: Arc<Mutex<PF>>,
	pub select_chain: S,
}

pub fn create_full<C, CIDP, I, L, P, PF, PP, S>(
	deps: FullDeps<C, CIDP, I, L, P, PF, PP, S>,
) -> Result<RpcModule<()>, Box<dyn Error + Send + Sync>>
where
	C: AuxStore,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
	C: CallApiAt<Block>,
	C: Send + Sync + 'static,
	C::Api: AccountNonceApi<Block, AccountId, AccountNonce>,
	C::Api: TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: BlockBuilder<Block>,
	C::Api: DifficultyApi<Block, Difficulty>,
	P: TransactionPool + 'static,
	I: BlockImport<Block> + Send + Sync + 'static,
	L: JustificationSyncLink<Block> + 'static,
	CIDP: CreateInherentDataProviders<Block, ()> + 'static,
	PF: Environment<Block> + Send + Sync + 'static,
	PF::Error: std::fmt::Debug,
	PF::Proposer: Proposer<Block>,
	PP: PreRuntimeProvider<Block> + Send + Sync + 'static,
	S: SelectChain<Block> + 'static,
{
	use substrate::{
		frames::system::rpc::{System, SystemApiServer},
		pallets::transaction_payment::rpc::{TransactionPayment, TransactionPaymentApiServer},
	};

	let mut module = RpcModule::new(());
	let FullDeps {
		client,
		create_inherent_data_providers,
		pool,
		block_import,
		justification_sync_link,
		deny_unsafe,
		build_time,
		pre_runtime_provider,
		proposer_factory,
		select_chain,
	} = deps;

	module.merge(System::new(client.clone(), pool, deny_unsafe).into_rpc())?;
	module.merge(TransactionPayment::new(client.clone()).into_rpc())?;

	module.merge(
		Miner::new(
			block_import,
			build_time,
			client,
			create_inherent_data_providers,
			justification_sync_link,
			pre_runtime_provider,
			proposer_factory,
			select_chain,
		)
		.into_rpc(),
	)?;

	Ok(module)
}
