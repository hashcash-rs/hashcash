// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use hashcash::primitives::core::{opaque::Block, AccountId, AccountNonce};
use jsonrpsee::RpcModule;
use std::{error::Error, sync::Arc};
use substrate::{
	client::{rpc::api::DenyUnsafe, transaction_pool::api::TransactionPool},
	frames::system::rpc::AccountNonceApi,
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
	C::Api: BlockBuilder<Block>,
	P: TransactionPool + 'static,
{
	use substrate::frames::system::rpc::{System, SystemApiServer};

	let mut module = RpcModule::new(());
	let FullDeps { client, pool, deny_unsafe } = deps;

	module.merge(System::new(client.clone(), pool, deny_unsafe).into_rpc())?;

	Ok(module)
}
