// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

mod algorithm;
mod block_validation;
mod import;
mod mainchain;
mod preludes;
mod submit;

pub use algorithm::*;
pub use block_validation::*;
pub use import::*;
pub use mainchain::*;
pub use submit::*;

pub use preludes::PreDigest;

pub const P2POOL_AUX_PREFIX: [u8; 4] = *b"P2P:";
