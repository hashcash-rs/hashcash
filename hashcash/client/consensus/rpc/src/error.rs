// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use hashcash::primitives::core::opaque::Block;
use jsonrpsee::types::{ErrorObject, ErrorObjectOwned};
use substrate::primitives::api::ApiError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error(transparent)]
	Codec(substrate::codec::Error),
	#[error(transparent)]
	Consensus(substrate::primitives::consensus::Error),
	#[error(transparent)]
	ConsensusPow(substrate::client::consensus::pow::Error<Block>),
	#[error(transparent)]
	Inherents(substrate::primitives::inherents::Error),
	#[error(transparent)]
	RuntimeApi(#[from] ApiError),
	#[error("{0}")]
	StorageChanges(String),
	#[error("{0}")]
	Proposer(String),
	/// Some other error.
	#[error(transparent)]
	Other(#[from] Box<dyn std::error::Error + Sync + Send + 'static>),
}

mod codes {
	pub const BASE: i32 = 1000;
	pub const CODEC: i32 = BASE + 1;
	pub const CONSENSUS: i32 = BASE + 2;
	pub const CONSENSUS_POW: i32 = BASE + 3;
	pub const INHERENTS: i32 = BASE + 4;
	pub const RUNTIME_API: i32 = BASE + 5;
	pub const STORAGE_CHANGES: i32 = BASE + 6;
	pub const PROPOSER: i32 = BASE + 7;
	pub const OTHER: i32 = BASE + 8;
}

impl From<Error> for ErrorObjectOwned {
	fn from(e: Error) -> Self {
		match e {
			Error::Codec(e) => ErrorObject::owned(codes::CODEC, e.to_string(), None::<()>),
			Error::Consensus(e) => ErrorObject::owned(codes::CONSENSUS, e.to_string(), None::<()>),
			Error::ConsensusPow(e) =>
				ErrorObject::owned(codes::CONSENSUS_POW, e.to_string(), None::<()>),
			Error::Inherents(e) => ErrorObject::owned(codes::INHERENTS, e.to_string(), None::<()>),
			Error::RuntimeApi(e) =>
				ErrorObject::owned(codes::RUNTIME_API, e.to_string(), None::<()>),
			Error::StorageChanges(e) => ErrorObject::owned(codes::STORAGE_CHANGES, e, None::<()>),
			Error::Proposer(e) => ErrorObject::owned(codes::PROPOSER, e, None::<()>),
			Error::Other(e) => ErrorObject::owned(codes::OTHER, e.to_string(), None::<()>),
		}
	}
}
