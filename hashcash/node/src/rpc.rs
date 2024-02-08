// Copyright (c) 2024 Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use hashcash::primitives::core::{opaque::Block, AccountId, AccountNonce, Balance};
use jsonrpsee::RpcModule;
use std::{error::Error, sync::Arc};
use substrate::{
	client::{rpc::api::DenyUnsafe, transaction_pool::api::TransactionPool},
	frames::system::rpc::AccountNonceApi,
	pallets::transaction_payment::rpc::TransactionPaymentRuntimeApi,
	primitives::{
		api::ProvideRuntimeApi,
		block_builder::BlockBuilder,
		blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata},
	},
};

pub struct FullDeps<C, P> {
	pub client: Arc<C>,
	pub pool: Arc<P>,
	pub deny_unsafe: DenyUnsafe,
}

pub fn create_full<C, P>(
	deps: FullDeps<C, P>,
) -> Result<RpcModule<()>, Box<dyn Error + Send + Sync>>
where
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
	C: Send + Sync + 'static,
	C::Api: AccountNonceApi<Block, AccountId, AccountNonce>,
	C::Api: TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: BlockBuilder<Block>,
	P: TransactionPool + 'static,
{
	use substrate::{
		frames::system::rpc::{System, SystemApiServer},
		pallets::transaction_payment::rpc::{TransactionPayment, TransactionPaymentApiServer},
	};

	let mut module = RpcModule::new(());
	let FullDeps { client, pool, deny_unsafe } = deps;

	module.merge(System::new(client.clone(), pool, deny_unsafe).into_rpc())?;
	module.merge(TransactionPayment::new(client).into_rpc())?;

	Ok(module)
}
