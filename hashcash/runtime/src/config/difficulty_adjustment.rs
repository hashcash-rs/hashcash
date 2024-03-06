// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::*;

use hashcash::primitives::core::units::SECONDS;

parameter_types! {
	/// Smoothing factor for difficulty adjustment.
	pub const Filter: u32 = 72;
	/// Desired block time in milliseconds.
	pub const TargetBlockTime: Moment = 120 * SECONDS;
}

impl hashcash::pallets::wtema::Config for Runtime {
	type Difficulty = Difficulty;
	type Filter = Filter;
	type TargetBlockTime = TargetBlockTime;
}
