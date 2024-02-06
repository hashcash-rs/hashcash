// Copyright (c) 2024 Ryuichi Sakamoto
// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

mod preludes;
use preludes::*;

use substrate::primitives::runtime::{
	traits::{IdentifyAccount, Verify},
	MultiSignature,
};

pub use substrate::primitives::{
	core::{H256, U256},
	runtime::traits::BlakeTwo256,
};

pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;
pub type AccountIndex = ();
pub type AccountNonce = u32;
pub type AccountPublic = <Signature as Verify>::Signer;
pub type Balance = u128;
pub type BlockNumber = u32;
pub type Difficulty = u128;
pub type Hash = H256;
pub type Moment = u64;
pub type Nonce = u64;
pub type Signature = MultiSignature;

pub mod opaque {
	use super::*;
	use substrate::primitives::runtime::{generic, OpaqueExtrinsic};

	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	pub type BlockId = generic::BlockId<Block>;
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	pub type UncheckedExtrinsic = OpaqueExtrinsic;
}

#[cfg(not(feature = "runtime"))]
pub use opaque::*;

#[allow(non_upper_case_globals)]
pub mod units {
	use super::*;

	/// A unit of base currency.
	pub const DOLLARS: Balance = 1_000_000_000_000_000_000;
	/// One hundredth of a dollar.
	pub const CENTS: Balance = DOLLARS / 100;

	/// Kibibytes.
	pub const KiB: u32 = 1024;
	/// Mebibytes.
	pub const MiB: u32 = 1024 * KiB;

	/// A second in milliseconds.
	pub const SECONDS: Moment = 1000;
}
