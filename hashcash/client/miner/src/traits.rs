// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

pub use crate::{block_submit::Error as BlockSubmitError, data::Error as MinerDataError};

use hashcash::client::api::{BlockSubmitParams, MinerData};
use substrate::primitives::runtime::traits::Block as BlockT;

#[async_trait::async_trait]
pub trait MinerDataBuilder {
	type Params;

	async fn build(&self, params: Self::Params) -> Result<MinerData, MinerDataError>;
}

#[async_trait::async_trait]
pub trait BlockSubmit<B: BlockT> {
	async fn submit_block(&self, params: BlockSubmitParams<B>)
		-> Result<B::Hash, BlockSubmitError>;
}
