// Copyright (c) Ryuichi Sakamoto
// SPDX-License-Identifier: Apache-2.0

//! Coinbase pallet for block rewards.

#![cfg_attr(not(feature = "std"), no_std)]
// XXX: Suppress deprecation warnings for constant weight of coinbase call.
// Remove this line when the weight is properly implemented.
#![allow(deprecated)]

pub use pallet::*;

use frame_support::traits::{Currency, LockIdentifier, LockableCurrency, WithdrawReasons};
use hashcash_primitives::coinbase::{InherentError, InherentType, INHERENT_IDENTIFIER};
use parity_scale_codec::FullCodec;
use sp_inherents::{InherentData, InherentIdentifier};
use sp_runtime::traits::{AtLeast32BitUnsigned, Get, SaturatedConversion, Zero};

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type InherentTypeOf<T> =
	InherentType<<T as frame_system::Config>::AccountId, <T as Config>::Difficulty>;

const LOG_TARGET: &str = "runtime::coinbase";
const LOCK_IDENTIFIER: LockIdentifier = *b"coinbase";

///
pub trait EmissionCurve<T: Config> {
	fn emit() -> BalanceOf<T>;
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_runtime::sp_std::prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		///
		type EmissionCurve: EmissionCurve<Self>;
		///
		type Currency: LockableCurrency<Self::AccountId>;
		///
		type MaxRewardSplits: Get<u32>;
		///
		type MaturationTime: Get<BlockNumberFor<Self>>;
		///
		type Difficulty: FullCodec + Copy + AtLeast32BitUnsigned;
	}

	#[pallet::storage]
	#[pallet::getter(fn rewards)]
	pub type Rewards<T: Config> = StorageMap<
		_,
		Twox64Concat,
		BlockNumberFor<T>,
		BoundedVec<(T::AccountId, BalanceOf<T>), T::MaxRewardSplits>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn reward_locks)]
	pub type RewardLocks<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>>;

	#[pallet::error]
	pub enum Error<T> {
		TooManyRewardSplits,
		InvalidReward,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn coinbase(
			origin: OriginFor<T>,
			rewards: Vec<(T::AccountId, BalanceOf<T>)>,
		) -> DispatchResult {
			ensure_none(origin)?;
			ensure!(
				rewards.len() <= T::MaxRewardSplits::get() as usize,
				Error::<T>::TooManyRewardSplits
			);

			let reward_emitted = T::EmissionCurve::emit();
			let mut reward_given = BalanceOf::<T>::zero();
			for (dest, value) in &rewards {
				drop(T::Currency::deposit_creating(dest, *value));
				reward_given += *value;

				RewardLocks::<T>::mutate(dest, |lock| {
					let new_lock = match lock.take() {
						Some(lock) => lock + *value,
						None => *value,
					};
					T::Currency::set_lock(
						LOCK_IDENTIFIER,
						dest,
						new_lock,
						WithdrawReasons::except(WithdrawReasons::TRANSACTION_PAYMENT),
					);
					*lock = Some(new_lock);
				});
			}
			ensure!(reward_given == reward_emitted, Error::<T>::InvalidReward);

			Rewards::<T>::insert(
				frame_system::Pallet::<T>::block_number(),
				BoundedVec::<_, T::MaxRewardSplits>::try_from(rewards).unwrap(),
			);

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(height: BlockNumberFor<T>) -> Weight {
			if height > T::MaturationTime::get() {
				let unlocked_height = height - T::MaturationTime::get();

				for (dest, value) in Rewards::<T>::take(unlocked_height) {
					RewardLocks::<T>::mutate(&dest, |lock| {
						let locked = lock.unwrap();
						if locked > value {
							T::Currency::set_lock(
								LOCK_IDENTIFIER,
								&dest,
								locked - value,
								WithdrawReasons::except(WithdrawReasons::TRANSACTION_PAYMENT),
							);
							*lock = Some(locked - value);
						} else {
							T::Currency::remove_lock(LOCK_IDENTIFIER, &dest);
							*lock = None;
						}
					});
				}
			}

			Weight::from_parts(0, 0)
		}
	}

	#[pallet::inherent]
	impl<T: Config> ProvideInherent for Pallet<T> {
		type Call = Call<T>;
		type Error = InherentError;
		const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;

		fn create_inherent(data: &InherentData) -> Option<Self::Call> {
			let inherent_data = data
				.get_data::<InherentTypeOf<T>>(&INHERENT_IDENTIFIER)
				.expect("Coinbase inherent data not correctly encoded")
				.expect("Coinbase inherent data must be provided");

			let total_weight: u128 = inherent_data
				.iter()
				.fold(0u128, |acc, (_, weight)| acc + (*weight).saturated_into::<u128>());
			let reward: u128 = T::EmissionCurve::emit().saturated_into();

			let mut rewards = Vec::<(T::AccountId, BalanceOf<T>)>::new();
			let mut reward_given = 0u128;
			let mut cumulative_weight = 0u128;

			for (dest, weight) in inherent_data {
				cumulative_weight += weight.saturated_into::<u128>();
				let next_value = cumulative_weight * reward / total_weight;
				rewards.push((dest, (next_value - reward_given).saturated_into()));
				reward_given = next_value;
			}

			if rewards
				.iter()
				.fold(0u128, |acc, (_, reward)| acc + (*reward).saturated_into::<u128>()) !=
				reward
			{
				log::error!(target: LOG_TARGET, "Sum of reward splits is not equal to the total reward");
				return None
			}

			Some(Call::coinbase { rewards })
		}

		fn is_inherent(call: &Self::Call) -> bool {
			matches!(call, Call::coinbase { .. })
		}
	}
}
