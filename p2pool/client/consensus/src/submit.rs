// Copyright (c) 2024 Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use futures::{channel::mpsc, StreamExt};
use jsonrpsee::{
	core::{client::ClientT, params::ArrayParams},
	http_client::{HttpClient, HttpClientBuilder},
	rpc_params,
};
use substrate::primitives::core::{Bytes, H256};

#[derive(Debug, thiserror::Error)]
pub enum BlockSubmitterError {
	#[error(transparent)]
	HttpClient(jsonrpsee::core::Error),
}

pub struct BlockSubmitter {
	rpc_client: HttpClient,
	pub tx: mpsc::Sender<Bytes>,
	rx: mpsc::Receiver<Bytes>,
}

impl BlockSubmitter {
	pub fn new(mainchain_rpc: String) -> Result<Self, BlockSubmitterError> {
		// TODO: Adjust channel buffer size
		let (tx, rx) = mpsc::channel(1024);
		Ok(Self {
			rpc_client: HttpClientBuilder::default()
				.build(mainchain_rpc)
				.map_err(BlockSubmitterError::HttpClient)?,
			tx,
			rx,
		})
	}

	pub async fn run(mut self) {
		loop {
			if let Some(block_submit_params) = self.rx.next().await {
				self.submit_block(block_submit_params).await;
			}
		}
	}

	async fn submit_block(&mut self, block_submit_params: Bytes) {
		match self
			.rpc_client
			.request::<H256, ArrayParams>("miner_submitBlock", rpc_params!(block_submit_params))
			.await
		{
			Ok(hash) => log::info!(target: LOG_TARGET, "📡 Block submitted: {}", hash),
			Err(err) => log::error!(target: LOG_TARGET, "Failed to submit block: {}", err),
		}
	}
}
