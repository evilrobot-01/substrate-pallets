#![cfg_attr(not(feature = "std"), no_std)]

mod traits;
mod types;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::{traits::Governance, types::*};
    use codec::HasCompact;
    use frame_support::{
        pallet_prelude::*, traits::Currency, traits::ReservableCurrency, StorageHasher,
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::{traits::AtLeast32BitUnsigned, traits::Bounded};

    type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    type AmountOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    type BlockNumberOf<T> = <T as frame_system::Config>::BlockNumber;
    type HashOf<T> = <T as frame_system::Config>::Hash;
    type QueryDataOf<T> = BoundedVec<u8, <T as Config>::MaxQueryDataLength>;
    type NonceOf<T> = <T as Config>::Nonce;
    type ValueOf<T> = BoundedVec<u8, <T as Config>::MaxValueDataLength>;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: ReservableCurrency<Self::AccountId>;

        #[pallet::constant]
        type MinimumStakeAmount: Get<AmountOf<Self>>;

        #[pallet::constant]
        type MaxQueryDataLength: Get<u32>;

        #[pallet::constant]
        type MaxValueDataLength: Get<u32>;

        type Nonce: AtLeast32BitUnsigned
            + Bounded
            + HasCompact
            + MaybeSerializeDeserialize
            + Member
            + TypeInfo
            + Member
            + Parameter
            + Default
            + Copy
            + MaxEncodedLen
            + PartialOrd
            + StorageHasher;

        #[pallet::constant]
        type ReportingLock: Get<u32>;

        type Governance: Governance<Self::AccountId>;
    }

    #[pallet::storage]
    pub(super) type Disputes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        HashOf<T>,
        Blake2_128Concat,
        NonceOf<T>,
        AmountOf<T>,
        OptionQuery,
    >;

    #[pallet::storage]
    pub(super) type Reports<T: Config> =
        StorageMap<_, Blake2_128Concat, HashOf<T>, Report, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        NewStaker(AccountIdOf<T>, AmountOf<T>),
        StakeWithdrawRequested(AccountIdOf<T>, AmountOf<T>),
        StakeWithdrawn(AccountIdOf<T>),
        /// [query_id, time, value, nonce, query_data, reporter]
        NewReport(
            HashOf<T>,
            BlockNumberOf<T>,
            ValueOf<T>,
            NonceOf<T>,
            QueryDataOf<T>,
            AccountIdOf<T>,
        ),
    }

    #[pallet::error]
    pub enum Error<T> {
        NoneValue,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Allows a reporter to submit stake
        #[pallet::weight(0)]
        pub fn deposit_stake(origin: OriginFor<T>, amount: AmountOf<T>) -> DispatchResult {
            let staker = ensure_signed(origin)?;
            let _vote_count = T::Governance::get_vote_count();
            let _vote_tally = T::Governance::get_vote_tally_by_address(&staker);
            Self::deposit_event(Event::NewStaker(staker, amount));
            todo!()
        }

        /// Allows a reporter to request to withdraw their stake
        #[pallet::weight(0)]
        pub fn request_staking_withdraw(
            origin: OriginFor<T>,
            amount: AmountOf<T>,
        ) -> DispatchResult {
            let staker = ensure_signed(origin)?;
            Self::deposit_event(Event::StakeWithdrawRequested(staker, amount));
            todo!()
        }

        /// Withdraws a reporter's stake
        #[pallet::weight(0)]
        pub fn withdraw_stake(origin: OriginFor<T>) -> DispatchResult {
            let staker = ensure_signed(origin)?;
            Self::deposit_event(Event::StakeWithdrawn(staker));
            todo!()
        }

        /// Allows a reporter to submit a value to the oracle
        #[pallet::weight(0)]
        pub fn submit_value(
            origin: OriginFor<T>,
            query_id: HashOf<T>,
            value: ValueOf<T>,
            nonce: NonceOf<T>,
            query_data: QueryDataOf<T>,
        ) -> DispatchResult {
            let reporter = ensure_signed(origin)?;
            let block = <frame_system::Pallet<T>>::block_number();
            Self::deposit_event(Event::NewReport(
                query_id, block, value, nonce, query_data, reporter,
            ));
            todo!()
        }
    }
}
