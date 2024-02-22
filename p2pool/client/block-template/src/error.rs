// Copyright (c) 2024 Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use substrate::codec;

#[derive(Debug, thiserror::Error)]
pub enum BlockTemplateError {
	#[error(transparent)]
	Codec(codec::Error),
	#[error(transparent)]
	HttpClient(jsonrpsee::core::Error),
	#[error(transparent)]
	AuxStore(substrate::primitives::blockchain::Error),
}
