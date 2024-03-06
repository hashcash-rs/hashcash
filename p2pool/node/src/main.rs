// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

mod preludes;
use preludes::*;

mod benchmarking;
mod chain_spec;
mod cli;
mod command;
mod rpc;
mod service;

use substrate::client::cli::Result;

fn main() -> Result<()> {
	command::run()
}
