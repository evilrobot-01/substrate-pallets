#![cfg_attr(not(feature = "std"), no_std)]

mod traits;
mod types;

use frame_support::traits::Currency;
pub use pallet::*;

type AmountOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::{traits::QueryDataStorage, *};
    use frame_support::{pallet_prelude::*, sp_runtime::traits::Hash, traits::ReservableCurrency};
    use frame_system::pallet_prelude::*;
    use types::*;

    type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    type HashOf<T> = <T as frame_system::Config>::Hash;
    type QueryDataOf<T> = BoundedVec<u8, <T as Config>::MaxQueryDataLength>;
    type ClaimTimestampsOf<T> = BoundedVec<u8, <T as Config>::MaxClaimTimestamps>;
    type QueryFeedsOf<T> = BoundedVec<HashOf<T>, <T as Config>::MaxQueryFeeds>;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: ReservableCurrency<Self::AccountId>;

        #[pallet::constant]
        type Fee: Get<AmountOf<Self>>;

        #[pallet::constant]
        type MaxClaimTimestamps: Get<u32>;

        #[pallet::constant]
        type MaxQueryDataLength: Get<u32>;

        #[pallet::constant]
        type MaxQueryFeeds: Get<u32>;

        type QueryData: QueryDataStorage<Self::Hash, BoundedVec<u8, Self::MaxQueryDataLength>>;
    }

    #[pallet::storage]
    pub(super) type DataFeed<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        HashOf<T>, // query_id
        Blake2_128Concat,
        HashOf<T>, // feed_id
        FeedDetails<AmountOf<T>>,
        OptionQuery,
    >;

    #[pallet::storage]
    pub(super) type CurrentFeeds<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        HashOf<T>,
        BoundedVec<HashOf<T>, <T as Config>::MaxQueryFeeds>,
        ValueQuery,
    >;
    #[pallet::storage]
    pub(super) type Tips<T: Config> =
        StorageMap<_, Blake2_128Concat, HashOf<T>, AmountOf<T>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// [query_id, amount, query_data, tipper]
        TipAdded(HashOf<T>, AmountOf<T>, QueryDataOf<T>, AccountIdOf<T>),
        /// [query_id, feed_id, query_data, owner]
        NewDataFeed(HashOf<T>, HashOf<T>, QueryDataOf<T>, AccountIdOf<T>),
        /// [feed_id, query_id, amount, sender, feed]
        DataFeedFunded(
            HashOf<T>,
            HashOf<T>,
            AmountOf<T>,
            AccountIdOf<T>,
            FeedDetails<AmountOf<T>>,
        ),
        /// [query_id, cumulative_reward, claimer]
        OneTimeTipClaimed(HashOf<T>, AmountOf<T>, AccountIdOf<T>),
        /// [feed_id, query_id, cumulative_reward, claimer]
        TipClaimed(HashOf<T>, HashOf<T>, AmountOf<T>, AccountIdOf<T>),
    }

    #[pallet::error]
    pub enum Error<T> {
        QueryMismatch,
        FeedNotFound,
        ExceededMaxQueryFeeds,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Function to run a single tip
        #[pallet::weight(0)]
        pub fn tip(
            origin: OriginFor<T>,
            query_id: HashOf<T>,
            amount: AmountOf<T>,
            query_data: BoundedVec<u8, T::MaxQueryDataLength>,
        ) -> DispatchResult {
            let tipper = ensure_signed(origin)?;
            ensure!(
                query_id == T::Hashing::hash_of(&query_data),
                <Error<T>>::QueryMismatch
            );
            ensure!(amount > AmountOf::<T>::default(), Error::<T>::QueryMismatch);
            T::QueryData::store_data(&query_data);
            <Tips<T>>::insert(query_id, amount);
            Self::deposit_event(Event::TipAdded(query_id, amount, query_data, tipper));
            todo!()
        }

        /// Function to claim singular tip
        #[pallet::weight(0)]
        pub fn claim_one_time_tip(
            origin: OriginFor<T>,
            query_id: HashOf<T>,
            _timestamps: ClaimTimestampsOf<T>,
        ) -> DispatchResult {
            let claimer = ensure_signed(origin)?;
            let cumulative_reward = AmountOf::<T>::default();
            Self::deposit_event(Event::OneTimeTipClaimed(
                query_id,
                cumulative_reward,
                claimer,
            ));
            todo!()
        }

        /// Allows Tellor reporters to claim their tips in batches
        #[pallet::weight(0)]
        pub fn claim_tip(
            origin: OriginFor<T>,
            feed_id: HashOf<T>,
            query_id: HashOf<T>,
            _timestamps: ClaimTimestampsOf<T>,
        ) -> DispatchResult {
            let claimer = ensure_signed(origin)?;
            let cumulative_reward = AmountOf::<T>::default();
            Self::deposit_event(Event::TipClaimed(
                feed_id,
                query_id,
                cumulative_reward,
                claimer,
            ));
            todo!()
        }

        /// Initializes data feed parameters.
        #[pallet::weight(0)]
        pub fn setup_data_feed(
            origin: OriginFor<T>,
            query_id: HashOf<T>,
            reward: AmountOf<T>,
            start_time: u128,
            interval: u128,
            window: u128,
            price_threshold: AmountOf<T>,
            reward_increase_per_second: AmountOf<T>,
            query_data: QueryDataOf<T>,
            amount: Option<AmountOf<T>>,
        ) -> DispatchResult {
            let owner = ensure_signed(origin.clone())?;
            let feed_id = T::Hashing::hash_of(&(
                query_id,
                reward,
                start_time,
                interval,
                window,
                price_threshold,
                reward_increase_per_second,
            ));
            let _feed = FeedDetails {
                reward,
                balance: AmountOf::<T>::default(),
                start_time,
                interval,
                window,
                price_threshold,
                reward_increase_per_second,
                feeds_with_funding_index: 0,
            };
            <CurrentFeeds<T>>::try_mutate(&query_id, |feeds| feeds.try_push(feed_id))
                .map_err(|_| <Error<T>>::ExceededMaxQueryFeeds)?;
            T::QueryData::store_data(&query_data);
            Self::deposit_event(Event::NewDataFeed(query_id, feed_id, query_data, owner));
            if let Some(amount) = amount {
                Self::fund_feed(origin, feed_id, query_id, amount)?;
            }
            todo!()
        }

        /// Allows dataFeed account to be filled with tokens
        #[pallet::weight(0)]
        pub fn fund_feed(
            origin: OriginFor<T>,
            feed_id: HashOf<T>,
            query_id: HashOf<T>,
            amount: AmountOf<T>,
        ) -> DispatchResult {
            let funder = ensure_signed(origin)?;
            let mut feed = DataFeed::<T>::get(query_id, feed_id).ok_or(<Error<T>>::FeedNotFound)?;
            feed.balance += amount;

            Self::deposit_event(Event::DataFeedFunded(
                feed_id, query_id, amount, funder, feed,
            ));
            todo!()
        }
    }
}
