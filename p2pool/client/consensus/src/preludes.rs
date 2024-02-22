// Copyright (c) 2024 Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

pub const LOG_TARGET: &'static str = "consensus";

pub mod hashcash {
	pub mod client {
		pub mod consensus {
			pub use hashcash_client_consensus::*;
			pub use hashcash_client_consensus_rpc as rpc;
		}
	}
	pub mod primitives {
		pub use hashcash_primitives_core as core;
	}
	pub use hashcash_randomx as randomx;
}

pub mod substrate {
	pub mod client {
		pub use sc_client_api as api;
		pub mod consensus {
			pub use sc_consensus_pow as pow;
		}
	}
	pub mod primitives {
		pub use sp_api as api;
		pub use sp_core as core;
		pub mod consensus {
			pub use sp_consensus_pow as pow;
		}
		pub use sp_runtime as runtime;
	}
	pub use parity_scale_codec as codec;
}

pub use hashcash::primitives::core::{
	opaque::{Block, BlockId},
	Difficulty, Hash, Nonce,
};
