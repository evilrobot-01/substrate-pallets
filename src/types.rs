use super::*;

/// A marketplace listing for minting of a collection.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct CollectionListing<T: Config> {
    /// The mint price.
    pub(super) mint_price: PriceOf<T>,
    /// The mint asset type.
    pub(super) asset: AssetIdOf<T>,
}

/// A secondary marketplace listing of collection item.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ItemListing<T: Config> {
    /// The listed price.
    pub(super) list_price: PriceOf<T>,
    /// The listed asset type.
    pub(super) asset: AssetIdOf<T>,
}
