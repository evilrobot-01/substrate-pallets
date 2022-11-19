use frame_support::pallet_prelude::*;

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct FeedDetails<Amount> {
    pub(crate) reward: Amount,
    pub(crate) balance: Amount,
    pub(crate) start_time: u128,
    pub(crate) interval: u128,
    pub(crate) window: u128,
    pub(crate) price_threshold: Amount,
    pub(crate) reward_increase_per_second: Amount,
    pub(crate) feeds_with_funding_index: u128,
}
