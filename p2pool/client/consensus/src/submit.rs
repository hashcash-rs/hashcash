// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use futures::stream::StreamExt;
use hashcash::{
	client::api::BlockSubmitParams,
	primitives::core::{Bytes, H256},
};
use jsonrpsee::{
	core::{client::ClientT, params::ArrayParams},
	http_client::{HttpClient, HttpClientBuilder},
	rpc_params,
};
use substrate::{
	client::utils::mpsc::{tracing_unbounded, TracingUnboundedReceiver, TracingUnboundedSender},
	codec::Encode,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error(transparent)]
	HttpClient(jsonrpsee::core::client::Error),
}

pub struct BlockSubmitWorker {
	rpc_client: HttpClient,
	pub tx: TracingUnboundedSender<BlockSubmitParams<Block>>,
	rx: TracingUnboundedReceiver<BlockSubmitParams<Block>>,
}

impl BlockSubmitWorker {
	pub fn new(mainchain_rpc: String) -> Result<Self, Error> {
		let (tx, rx) = tracing_unbounded("mpsc_block_submit", 100_000);
		Ok(Self {
			rpc_client: HttpClientBuilder::default()
				.build(mainchain_rpc)
				.map_err(Error::HttpClient)?,
			tx,
			rx,
		})
	}

	pub async fn run(mut self) {
		loop {
			if let Some(BlockSubmitParams { block, seal }) = self.rx.next().await {
				self.submit_block(block, seal).await;
			}
		}
	}

	async fn submit_block(&mut self, block: Block, seal: Vec<u8>) {
		match self
			.rpc_client
			.request::<H256, ArrayParams>(
				"miner_submitBlock",
				rpc_params!(Bytes::from((block, seal).encode())),
			)
			.await
		{
			Ok(hash) => log::info!(target: LOG_TARGET, "📡 Block submitted: {}", hash),
			Err(err) => log::error!(target: LOG_TARGET, "Failed to submit block: {}", err),
		}
	}
}
