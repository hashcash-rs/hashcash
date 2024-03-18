// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod hashcash {
	pub mod primitives {
		pub use hashcash_primitives_core as core;
	}
}

pub mod substrate {
	pub mod client {
		pub mod consensus {
			pub use sc_consensus::*;
			pub use sc_consensus_pow as pow;
		}
	}
	pub use parity_scale_codec as codec;
	pub mod primitives {
		pub mod consensus {
			pub use sp_consensus_pow as pow;
		}
		pub use sp_runtime as runtime;
	}
}
