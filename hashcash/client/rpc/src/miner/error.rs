// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use jsonrpsee::types::{ErrorObject, ErrorObjectOwned};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error(transparent)]
	Codec(substrate::codec::Error),
	#[error(transparent)]
	MinerData(hashcash::client::miner::data::Error),
	#[error(transparent)]
	BlockSubmit(hashcash::client::miner::block_submit::Error),
}

mod codes {
	pub const BASE: i32 = 1000;
	pub const CODEC: i32 = BASE + 1;
	pub const MINER_DATA: i32 = BASE + 2;
	pub const BLOCK_SUBMIT: i32 = BASE + 3;
}

impl From<Error> for ErrorObjectOwned {
	fn from(e: Error) -> Self {
		match e {
			Error::Codec(e) => ErrorObjectOwned::owned(codes::CODEC, e.to_string(), None::<()>),
			Error::MinerData(e) => ErrorObject::owned(codes::MINER_DATA, e.to_string(), None::<()>),
			Error::BlockSubmit(e) =>
				ErrorObject::owned(codes::BLOCK_SUBMIT, e.to_string(), None::<()>),
		}
	}
}
