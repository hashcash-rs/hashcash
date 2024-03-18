// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod hashcash {
	pub mod client {
		pub use hashcash_client_api as api;
		pub use hashcash_client_miner as miner;
	}
	pub use hashcash_primitives as primitives;
}
pub mod substrate {
	pub mod primitives {
		pub use sp_core as core;
	}
	pub use parity_scale_codec as codec;
}
