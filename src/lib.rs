#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use frame_support::{pallet_prelude::*, sp_runtime::Saturating, traits::Time};
    use frame_system::pallet_prelude::*;
    use sp_arithmetic::traits::AtLeast32BitUnsigned;
    use tellor::traits::UsingTellor;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        // The origin which may configure the pallet.
        type ConfigureOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// The type of price.
        type Price: AtLeast32BitUnsigned + MaybeSerializeDeserialize + Parameter + From<Self::Value>;

        /// The type of query identifier.
        type QueryId: Copy + MaybeSerializeDeserialize + Parameter;

        /// The UsingTellor trait helps pallets read data from Tellor.
        type Tellor: UsingTellor<
            Self::AccountId,
            Self::Price,
            Self::QueryId,
            <Self::Time as Time>::Moment,
        >;

        /// The on-chain time provider.
        type Time: Time<Moment = Self::Timestamp>;

        /// The type of timestamp.
        type Timestamp: AtLeast32BitUnsigned + Copy + From<u64>;

        // The type of resulting value stored.
        type Value: AtLeast32BitUnsigned + Copy + Parameter + From<Self::Price>;
    }

    // The pallet's runtime storage items.
    #[pallet::storage]
    #[pallet::getter(fn config)]
    pub type Configuration<T> = StorageValue<_, <T as Config>::QueryId>;
    #[pallet::storage]
    #[pallet::getter(fn values)]
    pub type Values<T> = StorageMap<
        _,
        Blake2_128Concat,
        <T as frame_system::Config>::AccountId,
        <T as Config>::Value,
    >;

    // Pallets use events to inform users when important changes are made.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The pallet was configured with a query identifier. [queryId]
        Configured { query_id: T::QueryId },
        /// A value was stored. [value, who]
        ValueStored { value: T::Value, who: T::AccountId },
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// The pallet has not been configured.
        NotConfigured,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// A sample dispatchable that takes a query identifier as a parameter, writes it to
        /// storage and emits an event. This function must be dispatched by the configured origin.
        #[pallet::call_index(0)]
        pub fn configure(origin: OriginFor<T>, query_id: T::QueryId) -> DispatchResult {
            // Only the configured origin can configure the pallet.
            T::ConfigureOrigin::ensure_origin(origin)?;
            // Store the query identifier
            <Configuration<T>>::put(query_id);
            // Emit an event
            Self::deposit_event(Event::Configured { query_id });
            Ok(())
        }

        /// A sample dispatchable that takes a single value as a parameter, derives some new value
        /// and then writes that derived value to storage and emits an event. This function must be
        /// dispatched by a signed extrinsic.
        #[pallet::call_index(1)]
        pub fn do_something(origin: OriginFor<T>, value: T::Value) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;
            // Get the query identifier, ensuring that the pallet has been configured
            let Some(query_id) = <Configuration<T>>::get() else { return Err(Error::<T>::NotConfigured.into()) };
            // Get the price from the configured query identifier
            if let Some(price) = Self::get_price(query_id) {
                // Derive some value from the price
                let derived_value = price.saturating_mul(value.into()).into();
                // Update storage
                <Values<T>>::set(&who, Some(derived_value));
                // Emit an event
                Self::deposit_event(Event::ValueStored {
                    value: derived_value,
                    who,
                });
            }
            Ok(())
        }
    }

    const FIFTEEN_MINUTES: u64 = 15 * 60 * 1_000;
    const ONE_DAY: u64 = 24 * 60 * 60 * 1_000;

    impl<T: Config> Pallet<T> {
        fn get_price(query_id: T::QueryId) -> Option<T::Price> {
            let timestamp = T::Time::now();

            // Retrieve data at least 15 minutes old to allow time for disputes
            T::Tellor::get_data_before(query_id, timestamp.saturating_sub(FIFTEEN_MINUTES.into()))
                .and_then(|(value, timestamp_retrieved)| {
                    // Check that the data is not too old
                    if timestamp.saturating_sub(timestamp_retrieved) < ONE_DAY.into() {
                        // Use the helper function to parse the value to a price
                        T::Tellor::value_to_price(value)
                    } else {
                        None
                    }
                })
        }
    }
}
