// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

mod algorithm;
mod import;
mod miner;
mod preludes;
mod submit;

pub use algorithm::*;
pub use import::*;
pub use miner::{start_miner, MinerParams};
pub use submit::*;

pub const P2POOL_AUX_PREFIX: [u8; 4] = *b"P2P:";
