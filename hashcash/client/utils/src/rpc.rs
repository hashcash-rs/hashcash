// Copyright (c) The Hashcash Authors
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;
use subxt::backend::rpc::reconnecting_rpc_client::{Client, FibonacciBackoff, PingConfig};

pub use subxt::{backend::rpc::*, Error};

pub async fn rpc_client_from_url<U: AsRef<str>>(url: U) -> Result<RpcClient, Error> {
	let rpc = Client::builder()
		.retry_policy(FibonacciBackoff::from_millis(100).max_delay(Duration::from_secs(10)))
		.enable_ws_ping(
			PingConfig::new()
				.ping_interval(Duration::from_secs(6))
				.inactive_limit(Duration::from_secs(30)),
		)
		.build(url.as_ref().to_string())
		.await
		.expect("Reconnecting client must be instantiated");

	Ok(RpcClient::new(rpc))
}
