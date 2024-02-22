// Copyright (c) 2024 Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use hashcash::{
	client::consensus::rpc::{Miner, MinerApiServer},
	primitives::core::{opaque::Block, AccountId, AccountNonce, Balance, Difficulty},
};
use jsonrpsee::RpcModule;
use parking_lot::Mutex;
use std::{error::Error, sync::Arc};
use substrate::{
	client::{
		api::AuxStore,
		consensus::{BlockImport, JustificationSyncLink},
		rpc::api::DenyUnsafe,
		transaction_pool::api::TransactionPool,
	},
	frames::system::rpc::AccountNonceApi,
	pallets::transaction_payment::rpc::TransactionPaymentRuntimeApi,
	primitives::{
		api::{CallApiAt, ProvideRuntimeApi},
		block_builder::BlockBuilder,
		blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata},
		consensus::pow::DifficultyApi,
	},
};

pub struct FullDeps<C, I, L, P> {
	pub client: Arc<C>,
	pub pool: Arc<P>,
	pub block_import: Arc<Mutex<I>>,
	pub justification_sync_link: Arc<L>,
	pub deny_unsafe: DenyUnsafe,
}

pub fn create_full<C, I, L, P>(
	deps: FullDeps<C, I, L, P>,
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
{
	use substrate::{
		frames::system::rpc::{System, SystemApiServer},
		pallets::transaction_payment::rpc::{TransactionPayment, TransactionPaymentApiServer},
	};

	let mut module = RpcModule::new(());
	let FullDeps { client, pool, block_import, justification_sync_link, deny_unsafe } = deps;

	module.merge(System::new(client.clone(), pool, deny_unsafe).into_rpc())?;
	module.merge(TransactionPayment::new(client.clone()).into_rpc())?;

	module.merge(Miner::new(client, block_import, justification_sync_link).into_rpc())?;

	Ok(module)
}
