// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

mod algorithm;
mod import;
mod preludes;
mod submit;

pub use algorithm::*;
pub use import::*;
pub use submit::*;

pub const P2POOL_AUX_PREFIX: [u8; 4] = *b"P2P:";
