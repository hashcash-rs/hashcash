// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod hashcash {
	pub mod client {
		pub use hashcash_client_miner as miner;
	}
	pub mod primitives {
		pub use hashcash_primitives_core as core;
	}
}
pub mod substrate {
	pub use parity_scale_codec as codec;
}
