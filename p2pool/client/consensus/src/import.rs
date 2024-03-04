// Copyright (c) 2024 Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::preludes::*;

use hashcash::client::consensus::{self, rpc::BlockTemplate, Seal};
use sp_runtime::{testing::DigestItem, SaturatedConversion};
use std::sync::Arc;
use substrate::{
	client::{
		api::AuxStore,
		consensus::{
			pow::{fetch_seal, find_pre_digest},
			BlockCheckParams, BlockImport, BlockImportParams, ForkChoiceStrategy, ImportResult,
		},
	},
	codec::{Decode, Encode},
	primitives::{
		consensus::{pow::POW_ENGINE_ID, Error as ConsensusError},
		core::{H256, U256},
		runtime::traits::{Block, Header},
	},
};

pub const MAINCHAIN_AUX_PREFIX: [u8; 4] = *b"MCH:";
pub const P2POOL_AUX_PREFIX: [u8; 4] = *b"P2P:";

struct P2PoolBlockImport<I, C> {
	inner: I,
	client: Arc<C>,
}

impl<I, C> P2PoolBlockImport<I, C> {
	pub fn new(inner: I, client: Arc<C>) -> Self {
		Self { inner, client }
	}
}

#[async_trait::async_trait]
impl<B, I, C> BlockImport<B> for P2PoolBlockImport<I, C>
where
	B: Block<Hash = H256>,
	I: BlockImport<B> + Send + Sync,
	I::Error: Into<ConsensusError>,
	C: AuxStore + Send + Sync,
{
	type Error = ConsensusError;

	async fn check_block(
		&mut self,
		block: BlockCheckParams<B>,
	) -> Result<ImportResult, Self::Error> {
		self.inner.check_block(block).await.map_err(Into::into)
	}

	async fn import_block(
		&mut self,
		block: BlockImportParams<B>,
	) -> Result<ImportResult, Self::Error> {
		if block.fork_choice == Some(ForkChoiceStrategy::Custom(true)) {
			let block_template = find_pre_digest::<B>(&block.header)?
				.map(|v| BlockTemplate::decode(&mut &v[..]))
				.ok_or(ConsensusError::ClientImport("Invalid pre digest".to_string()))?
				.map_err(|e| ConsensusError::ClientImport(e.to_string()))?;

			let mut mainchain_block = block_template.block.clone();
			let inner_seal = fetch_seal::<B>(block.post_digests.last(), block.header.hash())?;
			mainchain_block
				.header
				.digest_mut()
				.push(DigestItem::Seal(POW_ENGINE_ID, inner_seal.clone()));
			let key: Vec<u8> = MAINCHAIN_AUX_PREFIX
				.iter()
				.chain(mainchain_block.hash().as_ref())
				.copied()
				.collect();
			if let Some(_) = self
				.client
				.get_aux(&key[..])
				.map_err(|e| ConsensusError::ClientImport(e.to_string()))?
			{
				return Err(ConsensusError::ClientImport("Already imported".to_string()));
			}

			let _ = self
				.client
				.insert_aux(&[(&key[..], block.post_hash().as_bytes())], &[])
				.map_err(|e| ConsensusError::ClientImport(e.to_string()))?;

			let seal = Seal::decode(&mut &inner_seal[..])
				.map_err(|e| ConsensusError::ClientImport(e.to_string()))?;
			let work = consensus::calculate_hash(
				&block_template.seed_hash,
				(block_template.block.hash(), seal.nonce).encode().as_slice(),
			)
			.map_err(|_| {
				ConsensusError::ClientImport("Failed to calculate a RandomX hash".to_string())
			})?;
			let share: u128 = U256::max_value()
				.checked_div(U256::from_big_endian(work.as_bytes()))
				.ok_or(ConsensusError::ClientImport("Invalid RandomX hash".to_string()))?
				.saturated_into();
			let key: Vec<u8> = P2POOL_AUX_PREFIX
				.iter()
				.chain(block.post_hash().as_ref())
				.copied()
				.collect();
			let _ = self
				.client
				.insert_aux(&[(&key[..], &share.encode()[..])], &[])
				.map_err(|e| ConsensusError::ClientImport(e.to_string()))?;
		}

		self.inner.import_block(block).await.map_err(Into::into)
	}
}
