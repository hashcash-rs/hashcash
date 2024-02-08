// Copyright (c) 2024 Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{chain_spec, preludes::*};

use substrate::{
	client::cli::{commands::*, RunCmd, SubstrateCli},
	frames::benchmarking::cli::BenchmarkCmd,
};

#[derive(Debug, clap::Parser)]
pub struct Cli {
	#[command(subcommand)]
	pub subcommand: Option<Subcommand>,

	#[clap(flatten)]
	pub run: RunCmd,
}

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Hashcash".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/hashcash-rs/hashcash/issues".into()
	}

	fn copyright_start_year() -> i32 {
		2024
	}

	fn load_spec(
		&self,
		id: &str,
	) -> Result<Box<dyn substrate::client::service::ChainSpec>, String> {
		Ok(match id {
			"dev" => Box::new(chain_spec::development_config()?),
			"" | "local" => Box::new(chain_spec::local_testnet_config()?),
			path =>
				Box::new(chain_spec::ChainSpec::from_json_file(std::path::PathBuf::from(path))?),
		})
	}
}
#[derive(Debug, clap::Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Subcommand {
	#[command(subcommand)]
	Benchmark(BenchmarkCmd),
	BuildSpec(BuildSpecCmd),
	ChainInfo(ChainInfoCmd),
	CheckBlock(CheckBlockCmd),
	ExportBlocks(ExportBlocksCmd),
	ExportState(ExportStateCmd),
	ImportBlocks(ImportBlocksCmd),
	#[command(subcommand)]
	Key(KeySubcommand),
	PurgeChain(PurgeChainCmd),
	Revert(RevertCmd),
}
