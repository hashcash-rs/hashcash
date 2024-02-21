// Copyright (c) 2024 Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use hashcash::{
	primitives::core::{AccountId, Difficulty},
	runtime::{RuntimeGenesisConfig, WASM_BINARY},
};
use substrate::{
	client::service::{ChainType, GenericChainSpec},
	primitives::keyring::AccountKeyring,
};

pub type ChainSpec = GenericChainSpec<RuntimeGenesisConfig>;

pub fn development_config() -> Result<ChainSpec, String> {
	Ok(ChainSpec::builder(
		WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?,
		None,
	)
	.with_name("Development")
	.with_id("dev")
	.with_chain_type(ChainType::Development)
	.with_genesis_config_patch(testnet_genesis(
		vec![
			AccountKeyring::Alice.to_account_id(),
			AccountKeyring::Bob.to_account_id(),
			AccountKeyring::AliceStash.to_account_id(),
			AccountKeyring::BobStash.to_account_id(),
		],
		10_000,
		true,
	))
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
	.with_genesis_config_patch(testnet_genesis(
		vec![
			AccountKeyring::Alice.to_account_id(),
			AccountKeyring::Bob.to_account_id(),
			AccountKeyring::Charlie.to_account_id(),
			AccountKeyring::Dave.to_account_id(),
			AccountKeyring::Eve.to_account_id(),
			AccountKeyring::Ferdie.to_account_id(),
			AccountKeyring::AliceStash.to_account_id(),
			AccountKeyring::BobStash.to_account_id(),
			AccountKeyring::CharlieStash.to_account_id(),
			AccountKeyring::DaveStash.to_account_id(),
			AccountKeyring::EveStash.to_account_id(),
			AccountKeyring::FerdieStash.to_account_id(),
		],
		40_000,
		true,
	))
	.build())
}

fn testnet_genesis(
	endowed_accounts: Vec<AccountId>,
	initial_difficulty: Difficulty,
	_enable_println: bool,
) -> serde_json::Value {
	serde_json::json!({
		"balances": {
			"balances": endowed_accounts.iter().cloned().map(|k| (k, 1u64 << 60)).collect::<Vec<_>>(),
		},
		"difficultyAdjustment": {
			"difficulty": initial_difficulty,
		},
	})
}
