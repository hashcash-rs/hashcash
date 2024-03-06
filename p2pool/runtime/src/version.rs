// Copyright (c) 2024 Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::*;

#[cfg(feature = "std")]
use substrate::primitives::version::NativeVersion;

use substrate::primitives::{runtime::create_runtime_str, version::runtime_version};

#[runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("p2pool"),
	impl_name: create_runtime_str!("p2pool"),
	authoring_version: 1,
	// spec_version: MAJOR_MINOR_PATCH
	spec_version: 000_001_000,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}
