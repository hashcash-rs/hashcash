// Copyright (c) 2024 Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "runtime-benchmarks", allow(non_local_definitions))]
#![recursion_limit = "256"]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

mod preludes;
use preludes::*;

mod common;
mod config;
mod version;

pub use common::*;
pub use version::*;

use hashcash::primitives::core::{AccountId, AccountNonce, Difficulty, Moment};
use substrate::{
	frames::support::{
		construct_runtime,
		genesis_builder_helper::{build_config, create_default_config},
		weights::Weight,
	},
	primitives::{
		api::impl_runtime_apis,
		core::{crypto::KeyTypeId, OpaqueMetadata},
		genesis_builder::Result as GenesisBuilderResult,
		inherents::{CheckInherentsResult, InherentData},
		runtime::{
			traits::Block as BlockT,
			transaction_validity::{TransactionSource, TransactionValidity},
			ApplyExtrinsicResult,
		},
		version::RuntimeVersion,
	},
};

construct_runtime! {
	pub struct Runtime {
		System: frames::system = 0,
		Timestamp: pallets::timestamp = 2,
		DifficultyAdjustment: pallets::wtema = 17,
	}
}

impl_runtime_apis! {
	impl substrate::primitives::api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl substrate::primitives::api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl substrate::primitives::block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: InherentData,
		) -> CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl substrate::primitives::transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl substrate::primitives::session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(_seed: Option<Vec<u8>>) -> Vec<u8> {
			Default::default()
		}

		fn decode_session_keys(
			_encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			None
		}
	}
	impl substrate::primitives::offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl substrate::frames::system::rpc::runtime_api::AccountNonceApi<Block, AccountId, AccountNonce> for Runtime {
		fn account_nonce(account: AccountId) -> AccountNonce {
			System::account_nonce(account)
		}
	}

	impl substrate::primitives::consensus::pow::TimestampApi<Block, Moment> for Runtime {
		fn timestamp() -> Moment {
			Timestamp::get()
		}
	}

	impl substrate::primitives::consensus::pow::DifficultyApi<Block, Difficulty> for Runtime {
		fn difficulty() -> Difficulty {
			DifficultyAdjustment::difficulty()
		}
	}

	impl substrate::primitives::genesis_builder::GenesisBuilder<Block> for Runtime {
		fn create_default_config() -> Vec<u8> {
			create_default_config::<RuntimeGenesisConfig>()
		}

		fn build_config(config: Vec<u8>) -> GenesisBuilderResult {
			build_config::<RuntimeGenesisConfig>(config)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl substrate::frames::benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frames::benchmarking::BenchmarkList>,
			Vec<frames::support::traits::StorageInfo>,
		) {
			use frames::benchmarking::{baseline, Benchmarking, BenchmarkList};
			use frames::support::traits::StorageInfoTrait;
			use frames::system::benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();

			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frames::benchmarking::BenchmarkConfig
		) -> Result<Vec<frames::benchmarking::BenchmarkBatch>, substrate::primitives::runtime::RuntimeString> {
			use frames::benchmarking::{baseline, Benchmarking, BenchmarkBatch};
			use substrate::primitives::storage::TrackedStorageKey;
			use frames::system::benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;

			impl frames::system::benchmarking::Config for Runtime {}
			impl baseline::Config for Runtime {}

			use frames::support::traits::WhitelistedStorageKeys;
			let whitelist: Vec<TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);

			Ok(batches)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frames::try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: frames::try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
			use crate::config::system::BlockWeights;

			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, BlockWeights::get().max_block)
		}

		fn execute_block(
			block: Block,
			state_root_check: bool,
			signature_check: bool,
			select: frames::try_runtime::TryStateSelect
		) -> Weight {
			Executive::try_execute_block(block, state_root_check, signature_check, select).expect("execute-block failed")
		}
	}
}

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	use super::*;

	frames::benchmarking::define_benchmarks!(
		[frames::benchmarking, BaselineBench::<Runtime>]
		[frames::system, SystemBench::<Runtime>]
		[pallets::timestamp, Timestamp]
	);
}
