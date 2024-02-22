// Copyright (c) 2024 Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::*;

use hashcash::primitives::core::{
	constants::SS58_PREFIX, units::MiB, AccountId, AccountNonce, Hash,
};
use substrate::{
	frames::{
		support::{
			derive_impl,
			dispatch::DispatchClass,
			traits::{ConstU32, Contains},
			weights::{
				constants::{
					BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight,
					WEIGHT_REF_TIME_PER_SECOND,
				},
				Weight,
			},
		},
		system::{config_preludes::SolochainDefaultConfig, limits},
	},
	primitives::{runtime::Perbill, version::RuntimeVersion},
};

pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(1);
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
pub const MAXIMUM_BLOCK_WEIGHT: Weight =
	Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2), u64::MAX);

parameter_types! {
	pub const BlockHashCount: u32 = 4096;
	pub BlockLength: limits::BlockLength = limits::BlockLength
		::max_with_normal_ratio(5 * MiB, NORMAL_DISPATCH_RATIO);
	pub BlockWeights: limits::BlockWeights = limits::BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT,
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u16 = SS58_PREFIX;
	pub const Version: RuntimeVersion = VERSION;
}

pub struct BaseFilter;
impl Contains<RuntimeCall> for BaseFilter {
	fn contains(c: &RuntimeCall) -> bool {
		matches!(c, RuntimeCall::Timestamp(pallets::timestamp::Call::set { .. }))
	}
}

#[derive_impl(SolochainDefaultConfig as substrate::frames::system::DefaultConfig)]
impl substrate::frames::system::Config for Runtime {
	type AccountData = ();
	type AccountId = AccountId;
	type BaseCallFilter = BaseFilter;
	type Block = Block;
	type BlockHashCount = BlockHashCount;
	type BlockLength = BlockLength;
	type BlockWeights = BlockWeights;
	type DbWeight = RocksDbWeight;
	type Hash = Hash;
	type MaxConsumers = ConstU32<16>;
	type Nonce = AccountNonce;
	type SS58Prefix = SS58Prefix;
	type Version = Version;
}
