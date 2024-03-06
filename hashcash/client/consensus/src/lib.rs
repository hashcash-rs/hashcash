// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

mod preludes;

mod algorithm;
mod common;
pub mod inherents;
mod miner;
pub mod pow;
pub mod randomx;

pub use algorithm::*;
pub use common::*;
pub use miner::{start_miner, MinerParams, MiningHandle};

pub const STORAGE_KEY: &[u8] = b"block_template";
