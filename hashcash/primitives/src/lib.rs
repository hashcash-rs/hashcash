// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

mod preludes;

pub mod coinbase;

pub use hashcash_primitives_core as core;
