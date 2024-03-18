// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: Apache-2.0

//! WTEMA difficulty adjustment algorithm.

// target = prior_target * (1 + t/T/N - 1/N);
// where
//   N = smoothing constant aka filter
//   t = prior block solvetime
//   T = desired average block time

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use frame_support::traits::{Get, OnTimestampSet};
use parity_scale_codec::FullCodec;
use sp_core::U256;
use sp_runtime::{
	sp_std::fmt::Debug,
	traits::{One, SaturatedConversion, UniqueSaturatedFrom},
};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_timestamp::Config {
		type Difficulty: FullCodec
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ Debug
			+ MaxEncodedLen
			+ TypeInfo
			+ UniqueSaturatedFrom<U256>
			+ Into<U256>
			+ PartialOrd;

		#[pallet::constant]
		type TargetBlockTime: Get<Self::Moment>;

		#[pallet::constant]
		type Filter: Get<u32>;

		#[pallet::constant]
		type MinDifficulty: Get<Self::Difficulty>;
	}

	#[pallet::storage]
	#[pallet::getter(fn difficulty)]
	pub type Difficulty<T: Config> = StorageValue<_, T::Difficulty, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn timestamps)]
	pub type MostRecentTimestamp<T: Config> = StorageValue<_, T::Moment, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub difficulty: T::Difficulty,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { difficulty: T::Difficulty::saturated_from(U256::from(10000)) }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			assert!(self.difficulty.into() != U256::from(0));
			Difficulty::<T>::put(self.difficulty);
		}
	}
}

impl<T: Config> OnTimestampSet<T::Moment> for Pallet<T>
where
	T::Moment: Into<U256>,
{
	fn on_timestamp_set(now: T::Moment) {
		let block_time = match frame_system::Pallet::<T>::block_number() {
			n if n <= One::one() => T::TargetBlockTime::get(),
			_ => now - MostRecentTimestamp::<T>::get(),
		};
		let desired_block_time = T::TargetBlockTime::get().into();
		let prior_target = U256::max_value() / Difficulty::<T>::get().into();
		let filter = T::Filter::get();

		let target = (prior_target / (desired_block_time * filter))
			.saturating_mul(desired_block_time * filter + block_time - desired_block_time);
		let mut difficulty = T::Difficulty::saturated_from(U256::max_value() / target);

		if difficulty < T::MinDifficulty::get() {
			difficulty = T::MinDifficulty::get();
		}

		Difficulty::<T>::put(difficulty);
		MostRecentTimestamp::<T>::put(now);
	}
}
