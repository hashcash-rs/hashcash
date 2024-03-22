// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod substrate {
	pub mod client {
		pub use sc_cli as cli;
		pub use sc_network as network;
		pub use sc_service as service;
		pub use sc_telemetry as telemetry;
	}
	pub mod primitives {
		pub use sp_keyring as keyring;
	}
}
