// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

pub const LOG_TARGET: &str = "miner";

pub mod hashcash {
	pub mod client {
		pub use hashcash_client_api as api;
		pub use hashcash_client_randomx as randomx;
	}
	pub mod primitives {
		pub use hashcash_primitives_core as core;
	}
}

pub mod substrate {
	pub use parity_scale_codec as codec;
}

pub use hashcash::primitives::core::{Difficulty, Hash, Nonce};
