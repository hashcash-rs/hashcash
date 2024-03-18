// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::*;

use hashcash::primitives::core::units::SECONDS;

pub const MINIMUM_PERIOD: Moment = 3 * SECONDS;

parameter_types! {
	pub const MinimumPeriod: Moment = MINIMUM_PERIOD;
}

impl substrate::pallets::timestamp::Config for Runtime {
	type MinimumPeriod = MinimumPeriod;
	type Moment = Moment;
	type OnTimestampSet = DifficultyAdjustment;
	type WeightInfo = substrate::pallets::timestamp::weights::SubstrateWeight<Self>;
}
