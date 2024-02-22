// Copyright (c) 2024 Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod hashcash {
	pub mod primitives {
		pub use hashcash_primitives_core as core;
	}
}

pub mod p2pool {
	pub use p2pool_runtime as runtime;
	pub mod client {
		pub use p2pool_client_block_template as block_template;
		pub use p2pool_client_consensus as consensus;
	}
}

pub mod substrate {
	pub mod client {
		pub use sc_basic_authorship as basic_authorship;
		pub use sc_cli as cli;
		pub use sc_client_api as api;
		pub mod consensus {
			pub use sc_consensus::*;
			pub mod pow {
				pub use sc_consensus_pow::*;
			}
		}
		pub use sc_executor as executor;
		pub use sc_network as network;
		pub use sc_offchain as offchain;
		pub mod rpc {
			pub use sc_rpc_api as api;
		}
		pub use sc_service as service;
		pub use sc_telemetry as telemetry;
		pub mod transaction_pool {
			pub use sc_transaction_pool::*;
			pub use sc_transaction_pool_api as api;
		}
	}

	pub mod codec {
		pub use parity_scale_codec::*;
	}

	pub mod frames {
		pub mod benchmarking {
			pub use frame_benchmarking_cli as cli;
		}
		pub mod system {
			pub use substrate_frame_rpc_system as rpc;
		}
	}

	pub mod primitives {
		pub use sp_api as api;
		pub use sp_block_builder as block_builder;
		pub use sp_blockchain as blockchain;
		pub use sp_io as io;
		pub use sp_runtime as runtime;
		pub use sp_timestamp as timestamp;
	}
}
