// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use names::{Generator, Name};
use substrate::client::cli::DefaultConfigurationValues;

/// The maximum number of characters for a node name.
pub(crate) const NODE_NAME_MAX_LENGTH: usize = 64;

pub struct P2PoolConfigurationValues;

impl DefaultConfigurationValues for P2PoolConfigurationValues {
	fn p2p_listen_port() -> u16 {
		30433
	}
	fn rpc_listen_port() -> u16 {
		10044
	}
	fn prometheus_listen_port() -> u16 {
		9715
	}
}

/// Generate a valid random name for the node
pub fn generate_node_name() -> String {
	loop {
		let node_name = Generator::with_naming(Name::Numbered)
			.next()
			.expect("RNG is available on all supported platforms; qed");
		let count = node_name.chars().count();

		if count < NODE_NAME_MAX_LENGTH {
			return node_name
		}
	}
}
