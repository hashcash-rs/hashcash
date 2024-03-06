// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::*;

use hashcash::primitives::core::units::CENTS;
use smallvec::smallvec;
use substrate::{
	frames::support::weights::{
		constants::ExtrinsicBaseWeight, ConstantMultiplier, WeightToFeeCoefficient,
		WeightToFeeCoefficients, WeightToFeePolynomial,
	},
	pallets::transaction_payment::{CurrencyAdapter, Multiplier, TargetedFeeAdjustment},
	primitives::runtime::{traits::Bounded, FixedPointNumber, Perbill, Perquintill},
};

parameter_types! {
	pub const OperationalFeeMultiplier: u8 = 5;
	pub const TransactionByteFee: Balance = (1 * CENTS) / 100;

	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(75, 1_000_000);
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 10u128);
	pub MaximumMultiplier: Multiplier = Bounded::max_value();
}

pub type SlowAdjustingFeeUpdate<R> = TargetedFeeAdjustment<
	R,
	TargetBlockFullness,
	AdjustmentVariable,
	MinimumMultiplier,
	MaximumMultiplier,
>;

pub struct WeightToFee;

impl WeightToFeePolynomial for WeightToFee {
	type Balance = Balance;
	fn polynomial() -> WeightToFeeCoefficients<Balance> {
		let p = 1 * CENTS;
		let q = 10 * Balance::from(ExtrinsicBaseWeight::get().ref_time());
		smallvec![WeightToFeeCoefficient {
			coeff_integer: p / q,
			coeff_frac: Perbill::from_rational(p % q, q),
			negative: false,
			degree: 1,
		}]
	}
}

impl substrate::pallets::transaction_payment::Config for Runtime {
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type OnChargeTransaction = CurrencyAdapter<Balances, () /* TODO: DealWithFees */>;
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
	type RuntimeEvent = RuntimeEvent;
	type WeightToFee = WeightToFee;
}
