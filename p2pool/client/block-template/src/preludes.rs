// Copyright (c) 2024 Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod hashcash {
	pub mod client {
		pub mod consensus {
			pub use hashcash_client_consensus_rpc as rpc;
		}
	}
}
pub mod substrate {
	pub use parity_scale_codec as codec;
	pub mod client {
		pub use sc_client_api as api;
	}
	pub mod primitives {
		pub use sp_blockchain as blockchain;
	}
}
