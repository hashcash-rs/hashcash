// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

mod error;
use error::Error;

use hashcash::{
	client::{
		api::BlockSubmitParams,
		miner::{
			traits::{BlockSubmit, MinerDataBuilder},
			MinerData, MinerDataParams,
		},
	},
	primitives::core::{opaque::Block, AccountId, Difficulty, Hash},
};
use jsonrpsee::{core::async_trait, proc_macros::rpc};
use substrate::{
	codec::Decode,
	primitives::core::{Bytes, H256},
};

#[rpc(client, server)]
pub trait MinerApi {
	#[method(name = "miner_getMinerData")]
	fn miner_data(
		&self,
		author: AccountId,
		shares: Vec<(AccountId, Difficulty)>,
	) -> Result<MinerData, Error>;

	#[method(name = "miner_submitBlock")]
	async fn submit_block(&self, data: Bytes) -> Result<Hash, Error>;
}

pub struct Miner<MD, BS> {
	miner_data_builder: MD,
	block_submit: BS,
}

impl<MD, BS> Miner<MD, BS>
where
	MD: MinerDataBuilder<Params = MinerDataParams> + Send,
	BS: BlockSubmit<Block> + Send,
{
	pub fn new(miner_data_builder: MD, block_submit: BS) -> Self {
		Self { miner_data_builder, block_submit }
	}
}

#[async_trait]
impl<MD, BS> MinerApiServer for Miner<MD, BS>
where
	MD: MinerDataBuilder<Params = MinerDataParams> + Send + Sync + 'static,
	BS: BlockSubmit<Block> + Send + Sync + 'static,
{
	fn miner_data(
		&self,
		author: AccountId,
		shares: Vec<(AccountId, Difficulty)>,
	) -> Result<MinerData, Error> {
		futures::executor::block_on(
			self.miner_data_builder.build(MinerDataParams { author, shares }),
		)
		.map_err(Error::MinerData)
	}

	async fn submit_block(&self, data: Bytes) -> Result<H256, Error> {
		let BlockSubmitParams { block, seal } =
			BlockSubmitParams::<Block>::decode(&mut &data[..]).map_err(Error::Codec)?;

		self.block_submit
			.submit_block(BlockSubmitParams { block, seal })
			.await
			.map_err(Error::BlockSubmit)
	}
}
