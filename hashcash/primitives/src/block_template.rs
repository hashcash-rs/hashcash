// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "std")]

use crate::preludes::*;

use hashcash::primitives::core::{opaque::Block, Bytes, Difficulty, Hash};
use serde::{Deserialize, Serialize};
use substrate::codec::{Decode, Encode};

#[derive(Clone, Encode, Decode, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockTemplate {
	pub block: Block,
	pub difficulty: Difficulty,
	pub seed_hash: Hash,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq)]
pub struct BlockSubmitParams {
	pub block: Block,
	pub seal: Vec<u8>,
}

impl BlockSubmitParams {
	pub fn to_bytes(&self) -> Bytes {
		Bytes::from(self.encode())
	}
}
