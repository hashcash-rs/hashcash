// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: Apache-2.0

pub mod hashcash {
	pub mod primitives {
		pub use hashcash_primitives_core as core;
	}
}
pub mod substrate {
	pub use parity_scale_codec as codec;
	pub mod primitives {
		pub use sp_inherents as inherents;
		pub use sp_runtime::sp_std as std;
	}
}
