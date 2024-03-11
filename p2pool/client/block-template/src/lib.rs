// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

mod error;
mod preludes;
mod provider;

pub const LOG_TARGET: &str = "block-template";

pub use provider::BlockTemplateProvider;
