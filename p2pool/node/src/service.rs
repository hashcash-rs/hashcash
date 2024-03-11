// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{cli::CliOptions, preludes::*};

use futures::FutureExt;
use hashcash::primitives::core::{opaque::Block, AccountId};
use p2pool::{
	client::{
		block_template,
		consensus::{start_miner, BlockSubmitter, MinerParams, P2PoolAlgorithm, P2PoolBlockImport},
	},
	runtime::RuntimeApi,
};
use std::{sync::Arc, time::Duration};
use substrate::{
	client::{
		api::Backend,
		basic_authorship::ProposerFactory,
		consensus::{
			pow::{ImportQueueParams, PowBlockImport, PowParams},
			DefaultImportQueue, LongestChain,
		},
		executor::WasmExecutor,
		network::config::FullNetworkConfiguration,
		offchain::{OffchainWorkerOptions, OffchainWorkers},
		service::{self, Configuration, Error, TaskManager},
		telemetry::{Error as TelemetryError, Telemetry, TelemetryWorker},
		transaction_pool::{api::OffchainTransactionPoolFactory, BasicPool, FullPool},
	},
	codec::Encode,
	primitives::{
		io::SubstrateHostFunctions,
		runtime::{traits::Block as BlockT, ConsensusEngineId},
		timestamp::InherentDataProvider as TimestampInherentDataProvider,
	},
};

pub(crate) type FullClient =
	service::TFullClient<Block, RuntimeApi, WasmExecutor<SubstrateHostFunctions>>;
type FullBackend = service::TFullBackend<Block>;
type FullSelectChain = LongestChain<FullBackend, Block>;

pub type Service = service::PartialComponents<
	FullClient,
	FullBackend,
	FullSelectChain,
	DefaultImportQueue<Block>,
	FullPool<Block, FullClient>,
	(
		PowBlockImport<
			Block,
			P2PoolBlockImport<Arc<FullClient>, FullClient>,
			FullClient,
			FullSelectChain,
			P2PoolAlgorithm<FullClient>,
		>,
		Option<Telemetry>,
	),
>;

pub fn new_partial(config: &Configuration) -> Result<Service, Error> {
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, TelemetryError> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;

	let executor = service::new_wasm_executor::<SubstrateHostFunctions>(config);
	let (client, backend, keystore_container, task_manager) =
		service::new_full_parts::<Block, RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;
	let client = Arc::new(client);

	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});

	let select_chain = LongestChain::new(backend.clone());

	let transaction_pool = BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);

	let algorithm = P2PoolAlgorithm::new(client.clone());

	let p2pool_block_import = P2PoolBlockImport::new(client.clone(), client.clone());

	let pow_block_import = PowBlockImport::new(
		p2pool_block_import.clone(),
		client.clone(),
		select_chain.clone(),
		algorithm.clone(),
	);

	let import_queue = substrate::client::consensus::pow::import_queue(ImportQueueParams {
		block_import: pow_block_import.clone(),
		justification_import: None,
		client: client.clone(),
		algorithm: algorithm.clone(),
		create_inherent_data_providers: move |_, ()| async move {
			Ok(TimestampInherentDataProvider::from_system_time())
		},
		spawner: &task_manager.spawn_essential_handle(),
		registry: config.prometheus_registry(),
	})?;

	Ok(service::PartialComponents {
		client,
		backend,
		task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other: (pow_block_import, telemetry),
	})
}

struct PreRuntimeProvider {
	provider: block_template::BlockTemplateProvider<Block, FullClient>,
	author: AccountId,
}

impl PreRuntimeProvider {
	fn new(
		provider: block_template::BlockTemplateProvider<Block, FullClient>,
		author: AccountId,
	) -> Self {
		Self { provider, author }
	}
}

#[async_trait::async_trait]
impl substrate::client::consensus::pow::PreRuntimeProvider<Block> for PreRuntimeProvider {
	async fn pre_runtime(
		&self,
		best_hash: &<Block as BlockT>::Hash,
	) -> Vec<(ConsensusEngineId, Option<Vec<u8>>)> {
		let block_template = self.provider.block_template(best_hash).await;
		vec![(
			sp_consensus_pow::POW_ENGINE_ID,
			Some((self.author.clone(), block_template).encode()),
		)]
	}
}

pub fn new_full(config: Configuration, options: CliOptions) -> Result<TaskManager, Error> {
	let service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other: (block_import, mut telemetry),
	} = new_partial(&config)?;

	let net_config = FullNetworkConfiguration::new(&config.network);

	let (network, system_rpc_tx, tx_handler_controller, network_starter, sync_service) =
		service::build_network(service::BuildNetworkParams {
			config: &config,
			net_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			block_announce_validator_builder: None,
			warp_sync_params: None,
			block_relay: None,
		})?;

	if config.offchain_worker.enabled {
		task_manager.spawn_handle().spawn(
			"offchain-workers-runner",
			"offchain-worker",
			OffchainWorkers::new(OffchainWorkerOptions {
				runtime_api_provider: client.clone(),
				is_validator: config.role.is_authority(),
				keystore: Some(keystore_container.keystore()),
				offchain_db: backend.offchain_storage(),
				transaction_pool: Some(OffchainTransactionPoolFactory::new(
					transaction_pool.clone(),
				)),
				network_provider: network.clone(),
				enable_http_requests: true,
				custom_extensions: |_| vec![],
			})
			.run(client.clone(), task_manager.spawn_handle())
			.boxed(),
		);
	}

	let role = config.role.clone();
	let prometheus_registry = config.prometheus_registry().cloned();

	let rpc_extensions_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();

		Box::new(move |deny_unsafe, _| {
			let deps =
				crate::rpc::FullDeps { client: client.clone(), pool: pool.clone(), deny_unsafe };
			crate::rpc::create_full(deps).map_err(Into::into)
		})
	};

	let _rpc_handlers = service::spawn_tasks(service::SpawnTasksParams {
		network: network.clone(),
		client: client.clone(),
		keystore: keystore_container.keystore(),
		task_manager: &mut task_manager,
		transaction_pool: transaction_pool.clone(),
		rpc_builder: rpc_extensions_builder,
		backend: backend.clone(),
		system_rpc_tx,
		tx_handler_controller,
		sync_service: sync_service.clone(),
		config,
		telemetry: telemetry.as_mut(),
	})?;

	if role.is_authority() {
		let author = options.author_id.clone().unwrap();
		let window_size = options.window_size;
		let genesis_hash = client.chain_info().genesis_hash;
		let provider = block_template::BlockTemplateProvider::new(
			options.mainchain_rpc.clone(),
			client.clone(),
			author.clone(),
			genesis_hash,
			window_size,
		)
		.map_err(|e| Error::Other(e.to_string()))?;

		let worker =
			BlockSubmitter::new(options.mainchain_rpc).map_err(|e| Error::Other(e.to_string()))?;
		let submitter = worker.tx.clone();
		task_manager.spawn_handle().spawn("block-submitter", None, worker.run());

		let algorithm = P2PoolAlgorithm::new(client.clone());

		let proposer_factory = ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);

		let (worker, worker_task) =
			substrate::client::consensus::pow::start_mining_worker(PowParams {
				client: client.clone(),
				select_chain,
				block_import,
				algorithm,
				proposer_factory,
				sync_oracle: sync_service.clone(),
				justification_sync_link: sync_service.clone(),
				pre_runtime_provider: PreRuntimeProvider::new(provider, author),
				create_inherent_data_providers: move |_, ()| async move {
					Ok(TimestampInherentDataProvider::from_system_time())
				},
				timeout: Duration::new(10, 0),
				build_time: Duration::new(10, 0),
			});

		task_manager
			.spawn_handle()
			.spawn_blocking("pow", Some("block-authoring"), worker_task);

		start_miner(MinerParams {
			client,
			handle: Arc::new(worker),
			threads_count: options.threads.unwrap_or(1),
			submitter,
		});
	}

	network_starter.start_network();
	Ok(task_manager)
}
