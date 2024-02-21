// Copyright (c) 2024 Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod hashcash {
	pub mod client {
		pub use hashcash_client_consensus as consensus;
	}
	pub mod primitives {
		pub use hashcash_primitives_core as core;
	}
	pub use hashcash_runtime as runtime;
}

pub mod substrate {
	pub mod client {
		pub use sc_basic_authorship as basic_authorship;
		pub use sc_cli as cli;
		pub use sc_client_api as api;
		pub mod consensus {
			pub use sc_consensus::*;
			pub use sc_consensus_pow as pow;
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

	pub mod frames {
		pub mod benchmarking {
			pub use frame_benchmarking_cli as cli;
		}
		pub mod system {
			pub use substrate_frame_rpc_system as rpc;
		}
	}

	pub mod pallets {
		pub mod transaction_payment {
			pub use pallet_transaction_payment_rpc as rpc;
		}
	}

	pub mod primitives {
		pub use sp_api as api;
		pub use sp_block_builder as block_builder;
		pub use sp_blockchain as blockchain;
		pub use sp_core as core;
		pub use sp_io as io;
		pub use sp_keyring as keyring;
		pub use sp_timestamp as timestamp;
	}
}
