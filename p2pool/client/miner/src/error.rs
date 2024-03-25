// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use substrate::codec;

#[derive(Debug, thiserror::Error)]
pub enum MinerDataError {
	#[error(transparent)]
	Codec(codec::Error),
	#[error(transparent)]
	RpcClient(hashcash::client::utils::rpc::Error),
	#[error(transparent)]
	Blockchain(substrate::primitives::blockchain::Error),
	#[error("{0}")]
	Other(String),
}
