// Copyright (c) 2024 Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{error::*, preludes::*, STORAGE_KEY};

use hashcash::client::consensus::rpc::BlockTemplate;
use std::sync::Arc;
use substrate::{client::api::backend::AuxStore, codec::Decode};

#[derive(Clone)]
pub struct BlockTemplateProvider<C> {
	client: Arc<C>,
}

impl<C> BlockTemplateProvider<C>
where
	C: AuxStore + 'static,
{
	pub fn new(client: Arc<C>) -> Self {
		Self { client }
	}

	pub fn block_template(&self) -> Result<Option<BlockTemplate>, BlockTemplateError> {
		if let Some(value) = self
			.client
			.as_ref()
			.get_aux(STORAGE_KEY)
			.map_err(BlockTemplateError::AuxStore)?
		{
			Ok(Some(BlockTemplate::decode(&mut &value[..]).map_err(BlockTemplateError::Codec)?))
		} else {
			Ok(None)
		}
	}
}
