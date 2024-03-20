// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

pub const LOG_TARGET: &str = "consensus";

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
	pub mod client {
		pub use sc_client_api as api;
		pub mod consensus {
			pub use sc_consensus::*;
			pub use sc_consensus_pow as pow;
		}
		pub use sc_utils as utils;
	}
	pub mod primitives {
		pub use sp_api as api;
		pub use sp_core as core;
		pub mod consensus {
			pub use sp_consensus::*;
			pub use sp_consensus_pow as pow;
		}
		pub use sp_runtime as runtime;
	}
	pub use parity_scale_codec as codec;
}

pub use hashcash::primitives::core::{
	opaque::{Block, BlockId},
	Difficulty, Hash,
};
