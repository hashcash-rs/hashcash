// Copyright (c) Hisaishi Joe
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::*;

use hashcash::primitives::core::{AccountIndex, BlakeTwo256, BlockNumber, Signature};
use substrate::primitives::runtime::{generic, MultiAddress};

pub type Address = MultiAddress<AccountId, AccountIndex>;

pub type Block = generic::Block<Header, UncheckedExtrinsic>;

pub type Executive = substrate::frames::executive::Executive<
	Runtime,
	Block,
	substrate::frames::system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	Migrations,
>;

pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

pub type Migrations = ();

pub type SignedExtra = (
	substrate::frames::system::CheckNonZeroSender<Runtime>,
	substrate::frames::system::CheckSpecVersion<Runtime>,
	substrate::frames::system::CheckTxVersion<Runtime>,
	substrate::frames::system::CheckGenesis<Runtime>,
	substrate::frames::system::CheckMortality<Runtime>,
	substrate::frames::system::CheckNonce<Runtime>,
	substrate::frames::system::CheckWeight<Runtime>,
);

pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;

pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
