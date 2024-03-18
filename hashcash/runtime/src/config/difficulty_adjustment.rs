// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::*;

use super::timestamp::MINIMUM_PERIOD;
use hashcash::primitives::core::units::SECONDS;

const FILTER: u32 = 72;
const TARGET_BLOCK_TIME: Moment = 120 * SECONDS;

parameter_types! {
	/// Smoothing factor for difficulty adjustment.
	pub const Filter: u32 = FILTER;
	/// Desired block time in milliseconds.
	pub const TargetBlockTime: Moment = TARGET_BLOCK_TIME;
	/// Minimum difficulty to be adjusted according to block time changes.
	///
	/// If the difficulty drops below the minimum difficulty, it stops adjusting because of rounding errors.
	pub const MinDifficulty: Difficulty = ((FILTER as Moment * TARGET_BLOCK_TIME - 1) / (TARGET_BLOCK_TIME - MINIMUM_PERIOD)) as Difficulty;
}

impl hashcash::pallets::wtema::Config for Runtime {
	type Difficulty = Difficulty;
	type Filter = Filter;
	type TargetBlockTime = TargetBlockTime;
	type MinDifficulty = MinDifficulty;
}
