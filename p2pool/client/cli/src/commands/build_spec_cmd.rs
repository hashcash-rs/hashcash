// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{config::P2PoolConfigurationValues, preludes::*};

use clap::Parser;
use log::info;
use std::io::Write;
use substrate::client::{
	cli::{CliConfiguration, NodeKeyParams, Result, SharedParams},
	network::config::build_multiaddr,
	service::{
		config::{MultiaddrWithPeerId, NetworkConfiguration},
		ChainSpec,
	},
};

/// The `build-spec` command used to build a specification.
#[derive(Debug, Clone, Parser)]
pub struct BuildSpecCmd {
	/// Force raw genesis storage output.
	#[arg(long)]
	pub raw: bool,

	/// Disable adding the default bootnode to the specification.
	/// By default the `/ip4/127.0.0.1/tcp/30433/p2p/NODE_PEER_ID` bootnode is added to the
	/// specification when no bootnode exists.
	#[arg(long)]
	pub disable_default_bootnode: bool,

	#[allow(missing_docs)]
	#[clap(flatten)]
	pub shared_params: SharedParams,

	#[allow(missing_docs)]
	#[clap(flatten)]
	pub node_key_params: NodeKeyParams,
}

impl BuildSpecCmd {
	/// Run the build-spec command
	pub fn run(
		&self,
		mut spec: Box<dyn ChainSpec>,
		network_config: NetworkConfiguration,
	) -> Result<()> {
		info!("Building chain spec");
		let raw_output = self.raw;

		if spec.boot_nodes().is_empty() && !self.disable_default_bootnode {
			let keys = network_config.node_key.into_keypair()?;
			let peer_id = keys.public().to_peer_id();
			let addr = MultiaddrWithPeerId {
				multiaddr: build_multiaddr![Ip4([127, 0, 0, 1]), Tcp(30433u16)],
				peer_id,
			};
			spec.add_boot_node(addr)
		}

		let json = sc_service::chain_ops::build_spec(&*spec, raw_output)?;
		if std::io::stdout().write_all(json.as_bytes()).is_err() {
			let _ = std::io::stderr().write_all(b"Error writing to stdout\n");
		}
		Ok(())
	}
}

impl CliConfiguration<P2PoolConfigurationValues> for BuildSpecCmd {
	fn shared_params(&self) -> &SharedParams {
		&self.shared_params
	}

	fn node_key_params(&self) -> Option<&NodeKeyParams> {
		Some(&self.node_key_params)
	}
}
