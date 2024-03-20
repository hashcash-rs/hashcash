// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

pub const LOG_TARGET: &str = "randomx";

pub mod hashcash {
	pub mod primitives {
		pub use hashcash_primitives_core as core;
	}
	pub use hashcash_randomx as randomx;
}

pub use hashcash::primitives::core::Hash;
