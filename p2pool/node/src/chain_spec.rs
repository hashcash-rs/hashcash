// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use hashcash::primitives::core::Difficulty;
use p2pool::runtime::{RuntimeGenesisConfig, WASM_BINARY};
use substrate::client::service::{ChainType, GenericChainSpec};

pub type ChainSpec = GenericChainSpec<RuntimeGenesisConfig>;

pub fn development_config() -> Result<ChainSpec, String> {
	Ok(ChainSpec::builder(
		WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
		None,
	)
	.with_name("Development")
	.with_id("dev")
	.with_chain_type(ChainType::Development)
	.with_genesis_config_patch(testnet_genesis(5_000, true))
	.build())
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	Ok(ChainSpec::builder(
		WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
		None,
	)
	.with_name("Local Testnet")
	.with_id("local_testnet")
	.with_chain_type(ChainType::Local)
	.with_genesis_config_patch(testnet_genesis(20_000, true))
	.build())
}

fn testnet_genesis(initial_difficulty: Difficulty, _enable_println: bool) -> serde_json::Value {
	serde_json::json!({
		"difficultyAdjustment": {
			"difficulty": initial_difficulty,
		},
	})
}
