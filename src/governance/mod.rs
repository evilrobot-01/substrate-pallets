#![cfg_attr(not(feature = "std"), no_std)]

mod traits;
mod types;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::{traits::Oracle, types::*};
    use frame_support::{pallet_prelude::*, traits::ReservableCurrency};
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use sp_runtime::traits::Hash;

    type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    type HashOf<T> = <T as frame_system::Config>::Hash;
    type DisputeIdOf<T> = HashOf<T>;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: ReservableCurrency<Self::AccountId>;

        #[pallet::constant]
        type MaxDisputeVoteRounds: Get<u32>;

        type Oracle: Oracle<Self::Hash, u32, Self::AccountId, Self::Currency>;
    }

    #[pallet::storage]
    pub(super) type OpenDisputes<T: Config> =
        StorageMap<_, Blake2_128Concat, HashOf<T>, u32, ValueQuery>;

    #[pallet::storage]
    pub(super) type VoteRounds<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        HashOf<T>,
        BoundedVec<DisputeIdOf<T>, T::MaxDisputeVoteRounds>,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// [dispute_id, query_id, timestamp, disputer]
        NewDispute(HashOf<T>, HashOf<T>, u32, AccountIdOf<T>),
        /// [dispute_id, supports, voter, invalid_query]
        Voted(HashOf<T>, bool, AccountIdOf<T>, bool),
        /// [dispute_id, result, initiator, dispute_reporter]
        VoteTallied(HashOf<T>, VoteResult, AccountIdOf<T>, AccountIdOf<T>),
        /// [dispute_id, result]
        VoteExecuted(HashOf<T>, VoteResult),
    }

    #[pallet::error]
    pub enum Error<T> {
        NoValue,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Helps initialize a dispute by assigning it a disputeId
        #[pallet::weight(0)]
        pub fn begin_dispute(
            origin: OriginFor<T>,
            query_id: HashOf<T>,
            timestamp: u32,
        ) -> DispatchResult {
            ensure!(
                T::Oracle::get_block_number_by_timestamp(query_id, timestamp) != 0,
                <Error<T>>::NoValue
            );
            let disputer = ensure_signed(origin)?;
            let disputed_reporter = T::Oracle::get_reporter_by_timestamp(query_id, timestamp);
            let dispute_id = HashOf::<T>::default();
            let hash = T::Hashing::hash_of(&(query_id, timestamp));
            let vote_rounds = <VoteRounds<T>>::get(hash);
            if vote_rounds.len() == 1 {
                T::Oracle::slash_reporter(&disputed_reporter, &disputer);
                let _data = T::Oracle::retrieve_data(query_id, timestamp);
                T::Oracle::remove_value(query_id, timestamp);
            }

            Self::deposit_event(Event::NewDispute(dispute_id, query_id, timestamp, disputer));
            todo!()
        }

        /// Enables the sender address to cast a vote
        #[pallet::weight(0)]
        pub fn vote(
            origin: OriginFor<T>,
            dispute_id: HashOf<T>,
            supports: bool,
            invalid_query: bool,
        ) -> DispatchResult {
            let voter = ensure_signed(origin)?;
            let _staker_info = T::Oracle::get_staker_info(&voter);
            Self::deposit_event(Event::Voted(dispute_id, supports, voter, invalid_query));
            todo!()
        }

        /// Tallies the votes and begins the 1 day challenge period
        #[pallet::weight(0)]
        pub fn tally_votes(origin: OriginFor<T>, dispute_id: HashOf<T>) -> DispatchResult {
            let result = VoteResult::Invalid;
            let initiator = ensure_signed(origin)?;
            let dispute_reporter = initiator.clone();
            Self::deposit_event(Event::VoteTallied(
                dispute_id,
                result,
                initiator,
                dispute_reporter,
            ));
            todo!()
        }

        /// Executes vote and transfers corresponding balances to initiator/reporter
        #[pallet::weight(0)]
        pub fn execute_vote(origin: OriginFor<T>, dispute_id: HashOf<T>) -> DispatchResult {
            ensure_signed(origin)?;
            let result = VoteResult::Invalid;
            Self::deposit_event(Event::VoteExecuted(dispute_id, result));
            todo!()
        }
    }
}
