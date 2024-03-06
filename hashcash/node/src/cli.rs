// Copyright (c) 2024 Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{chain_spec, preludes::*};

use hashcash::primitives::core::AccountId;
use substrate::{
	client::cli::{self, commands::*, CliConfiguration, Error, SubstrateCli},
	frames::benchmarking::cli::BenchmarkCmd,
	primitives::{core::crypto::Ss58Codec, keyring::AccountKeyring},
};

#[derive(Debug, clap::Parser)]
pub struct CliOptions {
	/// Specify the number of threads to use for mining.
	#[arg(long, value_name = "COUNT")]
	pub threads: Option<usize>,
	/// Account for block mining rewards.
	#[arg(long)]
	pub author: Option<String>,

	// Hidden field to store a parsed author.
	#[arg(long, hide(true))]
	pub author_id: Option<AccountId>,
}

#[derive(Debug, clap::Parser)]
pub struct Cli {
	#[command(subcommand)]
	pub subcommand: Option<Subcommand>,

	#[clap(flatten)]
	pub run: RunCmd,

	#[clap(flatten)]
	pub options: CliOptions,
}

impl Cli {
	pub fn finalize(mut self) -> cli::Result<Self> {
		// RunCmd when subcommand is not specified
		if self.subcommand.is_none() {
			let is_dev = self.run.is_dev()?;
			if let Some(author) = &self.options.author {
				let author = AccountId::from_string(author)
					.map_err(|_| Error::Input("Invalid author".into()))?;
				self.options.author_id = Some(author);
			} else if is_dev {
				let keyring = &self.run.get_keyring().unwrap_or(AccountKeyring::Alice);
				self.options.author_id = Some(keyring.to_account_id());
			} else {
				if self.run.get_keyring().is_some() {
					return Err(Error::Input("Test keyring cannot be used in non-dev mode".into()));
				}
				return Err(Error::Input("No author specified".into()));
			}
		}
		Ok(self)
	}
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
