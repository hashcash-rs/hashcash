// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

use crate::{preludes::*, STORAGE_KEY};

use futures::{Future, StreamExt};
use log::*;
use substrate::{
	client::{
		api::{backend::AuxStore, BlockchainEvents},
		consensus::{
			pow::{
				MiningBuild, MiningHandle, MiningMetadata, PowAlgorithm, PowParams,
				PreRuntimeProvider, UntilImportedOrTimeout,
			},
			BlockImport,
		},
	},
	primitives::{
		consensus::{pow::POW_ENGINE_ID, Environment, Proposer, SelectChain, SyncOracle},
		inherents::{CreateInherentDataProviders, InherentDataProvider},
		runtime::{
			generic::{Digest, DigestItem},
			traits::{Block as BlockT, Header as HeaderT},
		},
	},
};

/// Start the mining worker for PoW. This function provides the necessary helper functions that can
/// be used to implement a miner. However, it does not do the CPU-intensive mining itself.
///
/// Two values are returned -- a worker, which contains functions that allows querying the current
/// mining metadata and submitting mined blocks, and a future, which must be polled to fill in
/// information in the worker.
///
/// `pre_runtime` is a parameter that allows a custom additional pre-runtime digest to be inserted
/// for blocks being built. This can encode authorship information, or just be a graffiti.
pub fn start_mining_worker<Block, C, S, I, Algorithm, PF, SO, L, CIDP, PP>(
	pow_params: PowParams<C, S, I, Algorithm, PF, SO, L, CIDP, PP>,
) -> (
	MiningHandle<Block, Algorithm, L, <PF::Proposer as Proposer<Block>>::Proof, I>,
	impl Future<Output = ()>,
)
where
	Block: BlockT,
	C: BlockchainEvents<Block> + AuxStore + 'static,
	S: SelectChain<Block> + 'static,
	I: BlockImport<Block> + Send + Sync + 'static,
	Algorithm: PowAlgorithm<Block> + Clone,
	Algorithm::Difficulty: Send + 'static,
	PF: Environment<Block> + Send + Sync + 'static,
	PF::Error: std::fmt::Debug,
	PF::Proposer: Proposer<Block>,
	SO: SyncOracle + Clone + Send + Sync + 'static,
	L: sc_consensus::JustificationSyncLink<Block>,
	CIDP: CreateInherentDataProviders<Block, ()>,
	PP: PreRuntimeProvider<Block>,
{
	let PowParams {
		client,
		select_chain,
		block_import,
		algorithm,
		mut proposer_factory,
		sync_oracle,
		justification_sync_link,
		create_inherent_data_providers,
		pre_runtime_provider,
		timeout,
		build_time,
	} = pow_params;

	let mut timer = UntilImportedOrTimeout::new(client.import_notification_stream(), timeout);
	let worker = MiningHandle::new(algorithm.clone(), block_import, justification_sync_link);
	let worker_ret = worker.clone();

	let task = async move {
		loop {
			if timer.next().await.is_none() {
				break
			}

			if sync_oracle.is_major_syncing() {
				debug!(target: LOG_TARGET, "Skipping proposal due to sync.");
				worker.on_major_syncing();
				continue
			}

			let best_header = match select_chain.best_chain().await {
				Ok(x) => x,
				Err(err) => {
					warn!(
						target: LOG_TARGET,
						"Unable to pull new block for authoring. \
						 Select best chain error: {}",
						err
					);
					continue
				},
			};
			let best_hash = best_header.hash();

			if worker.best_hash() == Some(best_hash) {
				continue
			}

			// The worker is locked for the duration of the whole proposing period. Within this
			// period, the mining target is outdated and useless anyway.

			let difficulty = match algorithm.difficulty(best_hash) {
				Ok(x) => x,
				Err(err) => {
					warn!(
						target: LOG_TARGET,
						"Unable to propose new block for authoring. \
						 Fetch difficulty failed: {}",
						err,
					);
					continue
				},
			};

			let inherent_data_providers = match create_inherent_data_providers
				.create_inherent_data_providers(best_hash, ())
				.await
			{
				Ok(x) => x,
				Err(err) => {
					warn!(
						target: LOG_TARGET,
						"Unable to propose new block for authoring. \
						 Creating inherent data providers failed: {}",
						err,
					);
					continue
				},
			};

			let inherent_data = match inherent_data_providers.create_inherent_data().await {
				Ok(r) => r,
				Err(e) => {
					warn!(
						target: LOG_TARGET,
						"Unable to propose new block for authoring. \
						 Creating inherent data failed: {}",
						e,
					);
					continue
				},
			};

			let mut inherent_digest = Digest::default();

			let mut pre_runtime = None;

			for (id, data) in pre_runtime_provider.pre_runtime(&best_hash) {
				if let Some(data) = data {
					if id == POW_ENGINE_ID {
						pre_runtime = Some(data.clone());
					}
					inherent_digest.push(DigestItem::PreRuntime(id, data));
				}
			}

			let proposer = match proposer_factory.init(&best_header).await {
				Ok(x) => x,
				Err(err) => {
					warn!(
						target: LOG_TARGET,
						"Unable to propose new block for authoring. \
						 Creating proposer failed: {:?}",
						err,
					);
					continue
				},
			};

			let proposal =
				match proposer.propose(inherent_data, inherent_digest, build_time, None).await {
					Ok(x) => x,
					Err(err) => {
						warn!(
							target: LOG_TARGET,
							"Unable to propose new block for authoring. \
							 Creating proposal failed: {}",
							err,
						);
						continue
					},
				};

			if let Err(e) =
				client.as_ref().insert_aux(&[(STORAGE_KEY, &proposal.block.encode()[..])], &[])
			{
				warn!(target: LOG_TARGET, "Unable to store block template: {:?}", e);
			}

			let build = MiningBuild::<Block, Algorithm, _> {
				metadata: MiningMetadata {
					best_hash,
					pre_hash: proposal.block.header().hash(),
					pre_runtime: pre_runtime.clone(),
					difficulty,
				},
				proposal,
			};

			worker.on_build(build);
		}
	};

	(worker_ret, task)
}
