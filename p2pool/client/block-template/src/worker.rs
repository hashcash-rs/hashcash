// Copyright (c) 2024 Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{error::*, preludes::*, LOG_TARGET, STORAGE_KEY};

use hashcash::client::consensus::rpc::BlockTemplate;
use jsonrpsee::{
	core::{client::ClientT, params::ArrayParams},
	http_client::{HttpClient, HttpClientBuilder},
};
use std::sync::Arc;
use substrate::{client::api::backend::AuxStore, codec::Encode};

pub struct BlockTemplateSyncWorker<C> {
	rpc_client: HttpClient,
	client: Arc<C>,
}

impl<C> BlockTemplateSyncWorker<C>
where
	C: AuxStore + 'static,
{
	pub fn new(mainchain_rpc: String, client: Arc<C>) -> Result<Self, BlockTemplateError> {
		Ok(Self {
			rpc_client: HttpClientBuilder::default()
				.build(mainchain_rpc)
				.map_err(BlockTemplateError::HttpClient)?,
			client,
		})
	}

	pub async fn run(self) {
		loop {
			self.update_block_template().await;
			tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
		}
	}

	async fn update_block_template(&self) {
		match self
			.rpc_client
			.request::<Option<BlockTemplate>, ArrayParams>(
				"miner_getBlockTemplate",
				ArrayParams::default(),
			)
			.await
		{
			Ok(Some(res)) => {
				let _ = self.client.as_ref().insert_aux(&[(STORAGE_KEY, &res.encode()[..])], &[]);
			},
			Ok(None) => (),
			Err(e) => {
				log::warn!(
					target: LOG_TARGET,
					"Unable to get block template: {:?}",
					e
				);
			},
		}
	}
}
