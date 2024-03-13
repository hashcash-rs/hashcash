// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{cli::CliOptions, preludes::*};

use futures::FutureExt;
use hashcash::{
	client::consensus::{inherents::coinbase::InherentDataProvider, MinerParams, RandomXAlgorithm},
	primitives::core::{constants::SS58_PREFIX, opaque::Block, AccountId, Hash},
	runtime::RuntimeApi,
};
use log::{info, Level};
use parking_lot::Mutex;
use std::{sync::Arc, time::Duration};
use substrate::{
	client::{
		api::Backend,
		basic_authorship::ProposerFactory,
		consensus::{
			pow::{
				EmptyPreRuntimeProvider, ImportQueueParams, PowBlockImport, PowParams,
				PreRuntimeProvider,
			},
			DefaultImportQueue, LongestChain,
		},
		executor::WasmExecutor,
		network::config::FullNetworkConfiguration,
		offchain::{OffchainWorkerOptions, OffchainWorkers},
		service::{self, Configuration, Error, TaskManager},
		telemetry::{Error as TelemetryError, Telemetry, TelemetryWorker},
		transaction_pool::{api::OffchainTransactionPoolFactory, BasicPool, FullPool},
	},
	primitives::{
		consensus::pow::POW_ENGINE_ID,
		core::{
			crypto::{Ss58AddressFormat, Ss58Codec},
			Encode,
		},
		io::SubstrateHostFunctions,
		runtime::ConsensusEngineId,
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
			Arc<FullClient>,
			FullClient,
			FullSelectChain,
			RandomXAlgorithm<FullClient>,
		>,
		Option<Telemetry>,
	),
>;

struct AuthorProvider {
	pub author: AccountId,
}

#[async_trait::async_trait]
impl PreRuntimeProvider<Block> for AuthorProvider {
	async fn pre_runtime(&self, _best_hash: &Hash) -> Vec<(ConsensusEngineId, Option<Vec<u8>>)> {
		vec![(POW_ENGINE_ID, Some(self.author.encode()))]
	}
}

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

	let algorithm = RandomXAlgorithm::new(client.clone());

	let pow_block_import = PowBlockImport::new(
		client.clone(),
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
			let timestamp = TimestampInherentDataProvider::from_system_time();
			Ok(timestamp)
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
		let block_import = block_import.clone();
		let sync_service = sync_service.clone();
		let select_chain = select_chain.clone();

		let mut proposer_factory = ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);
		proposer_factory.set_log_level(Level::Trace);
		let proposer_factory = Arc::new(Mutex::new(proposer_factory));

		let proposer_factory = proposer_factory.clone();

		Box::new(move |deny_unsafe, _| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: pool.clone(),
				block_import: block_import.clone(),
				justification_sync_link: sync_service.clone(),
				deny_unsafe,
				build_time: Duration::new(10, 0),
				create_inherent_data_providers: move |_, ()| async move {
					Ok(TimestampInherentDataProvider::from_system_time())
				},
				pre_runtime_provider: EmptyPreRuntimeProvider::<Block>::new(),
				proposer_factory: proposer_factory.clone(),
				select_chain: select_chain.clone(),
			};
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
		let algorithm = RandomXAlgorithm::new(client.clone());

		let proposer_factory = ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);

		let author = options.author_id.clone().unwrap();
		let (worker, worker_task) =
			substrate::client::consensus::pow::start_mining_worker(PowParams {
				client: client.clone(),
				select_chain,
				block_import,
				algorithm,
				proposer_factory,
				sync_oracle: sync_service.clone(),
				justification_sync_link: sync_service.clone(),
				pre_runtime_provider: AuthorProvider { author: author.clone() },
				create_inherent_data_providers: move |_, ()| {
					let author = author.clone();
					async move {
						let coinbase = InherentDataProvider::new(author);
						let timestamp = TimestampInherentDataProvider::from_system_time();
						Ok((coinbase, timestamp))
					}
				},
				timeout: Duration::new(10, 0),
				build_time: Duration::new(10, 0),
			});

		task_manager
			.spawn_handle()
			.spawn_blocking("pow", Some("block-authoring"), worker_task);

		let author = options.author_id.unwrap();
		info!(
			"⚒️  Miner address is: {}",
			author.to_ss58check_with_version(Ss58AddressFormat::custom(SS58_PREFIX))
		);

		hashcash::client::consensus::start_miner(MinerParams {
			client,
			handle: Arc::new(worker),
			threads_count: options.threads.unwrap_or(1),
		});
	}

	network_starter.start_network();
	Ok(task_manager)
}
