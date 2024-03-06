// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::*;

use hashcash::{
	pallets::coinbase::{BalanceOf, EmissionCurve},
	primitives::core::{units::DOLLARS, BlockNumber},
};

const TARGET_ISSUANCE: Balance = 1_000_000_000 * DOLLARS;

parameter_types! {
	pub const MaxRewardSplits: u32 = 1024;
	pub const MaturationTime: BlockNumber = 60;
}

impl hashcash::pallets::coinbase::Config for Runtime {
	type EmissionCurve = HashcashEmissionCurve;
	type Currency = Balances;
	type MaxRewardSplits = MaxRewardSplits;
	type MaturationTime = MaturationTime;
	type Difficulty = Difficulty;
}

pub struct HashcashEmissionCurve;

impl EmissionCurve<Runtime> for HashcashEmissionCurve {
	fn emit() -> BalanceOf<Runtime> {
		let total_issuance = <Runtime as pallet_coinbase::Config>::Currency::total_issuance();

		(TARGET_ISSUANCE - total_issuance) / ((2 as Balance) << 20)
	}
}
