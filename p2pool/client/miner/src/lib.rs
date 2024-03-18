// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

mod backend;
mod error;
mod preludes;
mod provider;

pub const LOG_TARGET: &str = "miner-data";

pub use backend::MiningWorkerBackend;
pub use provider::MinerDataProvider;
