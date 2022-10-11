#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;

sp_api::decl_runtime_apis! {
    pub trait DexApi<Balance, AssetId>
        where Balance: Codec, AssetId: Codec
    {
        /// Calculates the output amount of asset `other`, given an input `amount` and `asset` type.
        /// # Arguments
        /// * `amount` - An amount to be valued.
        /// * `asset` - The asset type of the amount.
        /// * `other` - The required asset type.
        fn price(amount: Balance, asset: AssetId, other: AssetId) -> Balance;
    }
}
