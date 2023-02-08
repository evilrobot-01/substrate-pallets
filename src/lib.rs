#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::Time};
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type AuthorisedSigners: Get<Vec<[u8; 33]>>;
        #[pallet::constant]
        type DataFeedId: Get<[u8; 32]>;
        #[pallet::constant]
        type MaxPayloadLen: Get<u32>;
        #[pallet::constant]
        type UniqueSignersThreshold: Get<u8>;

        type Moment: Into<u128>;
        type Time: Time<Moment = Self::Moment>;
    }

    #[pallet::storage]
    #[pallet::getter(fn value)]
    pub type Value<T> = StorageValue<_, u128>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// [value, who]
        ValueStored { value: u128, who: T::AccountId },
    }

    #[pallet::error]
    pub enum Error<T> {
        NoneValue,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        pub fn submit_value(
            origin: OriginFor<T>,
            payload: BoundedVec<u8, T::MaxPayloadLen>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let value = redstone_rust_sdk::get_oracle_value(
                &T::DataFeedId::get(),
                T::UniqueSignersThreshold::get(),
                &T::AuthorisedSigners::get(),
                T::Time::now().into(),
                &payload,
            );

            <Value<T>>::put(value);
            Self::deposit_event(Event::ValueStored { value, who });
            Ok(())
        }
    }
}
