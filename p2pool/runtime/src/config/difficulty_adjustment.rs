// Copyright (c) 2024 Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::*;

use hashcash::primitives::core::{units::SECONDS, Difficulty, Moment};

parameter_types! {
	/// Smoothing factor for difficulty adjustment.
	pub const Filter: u32 = 72;
	/// Desired block time in milliseconds.
	pub const TargetBlockTime: Moment = 10 * SECONDS;
}

impl hashcash::pallets::wtema::Config for Runtime {
	type Difficulty = Difficulty;
	type Filter = Filter;
	type TargetBlockTime = TargetBlockTime;
}
