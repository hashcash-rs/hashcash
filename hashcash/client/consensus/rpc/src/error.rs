// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use hashcash::primitives::core::opaque::Block;
use jsonrpsee::types::{ErrorObject, ErrorObjectOwned};
use substrate::primitives::api::ApiError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error(transparent)]
	AuxStore(substrate::primitives::blockchain::Error),
	#[error(transparent)]
	Codec(substrate::codec::Error),
	#[error(transparent)]
	ConsensusPow(substrate::client::consensus::pow::Error<Block>),
	#[error(transparent)]
	RuntimeApi(#[from] ApiError),
	#[error("{0}")]
	StorageChanges(String),
}

mod codes {
	pub const BASE: i32 = 1000;
	pub const AUX_STORE: i32 = BASE + 1;
	pub const CODEC: i32 = BASE + 2;
	pub const CONSENSUS_POW: i32 = BASE + 3;
	pub const RUNTIME_API: i32 = BASE + 4;
	pub const STORAGE_CHANGES: i32 = BASE + 5;
}

impl From<Error> for ErrorObjectOwned {
	fn from(e: Error) -> Self {
		match e {
			Error::AuxStore(e) => ErrorObject::owned(codes::AUX_STORE, e.to_string(), None::<()>),
			Error::Codec(e) => ErrorObject::owned(codes::CODEC, e.to_string(), None::<()>),
			Error::ConsensusPow(e) =>
				ErrorObject::owned(codes::CONSENSUS_POW, e.to_string(), None::<()>),
			Error::RuntimeApi(e) =>
				ErrorObject::owned(codes::RUNTIME_API, e.to_string(), None::<()>),
			Error::StorageChanges(e) => ErrorObject::owned(codes::STORAGE_CHANGES, e, None::<()>),
		}
	}
}
