// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: Apache-2.0

pub mod substrate {
	pub mod primitives {
		pub use sp_core as core;
		pub use sp_runtime as runtime;
	}
}
