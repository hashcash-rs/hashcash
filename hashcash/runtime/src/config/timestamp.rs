// Copyright (c) 2024 Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::*;

use hashcash::primitives::core::{units::SECONDS, Moment};

parameter_types! {
	pub const MinimumPeriod: Moment = 3 * SECONDS;
}

impl substrate::pallets::timestamp::Config for Runtime {
	type MinimumPeriod = MinimumPeriod;
	type Moment = Moment;
	type OnTimestampSet = Wtema;
	type WeightInfo = substrate::pallets::timestamp::weights::SubstrateWeight<Self>;
}