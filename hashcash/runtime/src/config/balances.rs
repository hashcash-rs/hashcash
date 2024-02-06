// Copyright (c) 2024 Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::*;

use hashcash::primitives::core::{units::DOLLARS, Balance};
use substrate::primitives::runtime::traits::ConstU32;

parameter_types! {
	pub const ExistentialDeposit: Balance = 1 * DOLLARS;
}

impl substrate::pallets::balances::Config for Runtime {
	type AccountStore = System;
	type Balance = Balance;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type FreezeIdentifier = RuntimeFreezeReason;
	type MaxFreezes = ConstU32<8>;
	type MaxHolds = ConstU32<1>;
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type RuntimeEvent = RuntimeEvent;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type RuntimeHoldReason = RuntimeHoldReason;
	type WeightInfo = substrate::pallets::balances::weights::SubstrateWeight<Self>;
}
