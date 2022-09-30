use super::*;

/// A marketplace listing.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct CollectionListing<T: Config> {
	/// The mint price.
	pub(super) price: PriceOf<T>,
	/// The mint asset type.
	pub(super) asset: AssetIdOf<T>,
}

/// A marketplace listing.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ItemListing<T: Config> {
	/// The listed price.
	pub(super) price: PriceOf<T>,
	/// The listed asset type.
	pub(super) asset: AssetIdOf<T>,
}
