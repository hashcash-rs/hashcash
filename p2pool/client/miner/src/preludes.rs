// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod hashcash {
	pub mod client {
		pub use hashcash_client_api as api;
	}
	pub mod primitives {
		pub use hashcash_primitives_core as core;
	}
}
pub mod substrate {
	pub use parity_scale_codec as codec;
	pub mod client {
		pub use sc_client_api as api;
		pub use sc_utils as utils;
	}
	pub mod primitives {
		pub use sp_blockchain as blockchain;
		pub mod consensus {
			pub use sp_consensus_pow as pow;
		}
		pub use sp_runtime as runtime;
	}
}
pub mod p2pool {
	pub mod client {
		pub use p2pool_client_consensus as consensus;
	}
}
