// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use hashcash::{
	primitives::core::{AccountId, Difficulty},
	runtime::{RuntimeGenesisConfig, WASM_BINARY},
};
use substrate::client::service::{ChainType, GenericChainSpec, Properties};

pub type ChainSpec = GenericChainSpec<RuntimeGenesisConfig>;

fn props() -> Properties {
	let mut properties = Properties::new();
	properties.insert("tokenDecimals".to_string(), 18.into());
	properties.insert("tokenSymbol".to_string(), "HCD".into());
	properties
}

pub fn development_config() -> Result<ChainSpec, String> {
	Ok(ChainSpec::builder(
		WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
		None,
	)
	.with_name("Development")
	.with_id("dev")
	.with_chain_type(ChainType::Development)
	.with_genesis_config_patch(testnet_genesis(vec![], 10_000, true))
	.with_properties(props())
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
	.with_genesis_config_patch(testnet_genesis(vec![], 40_000, true))
	.with_properties(props())
	.build())
}

fn testnet_genesis(
	_endowed_accounts: Vec<AccountId>,
	initial_difficulty: Difficulty,
	_enable_println: bool,
) -> serde_json::Value {
	serde_json::json!({
		"difficultyAdjustment": {
			"difficulty": initial_difficulty,
		},
	})
}
