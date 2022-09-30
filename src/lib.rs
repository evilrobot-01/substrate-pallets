#![cfg_attr(not(feature = "std"), no_std)]
use codec::HasCompact;
use frame_support::{
	dispatch::DispatchResult, pallet_prelude::*, traits::fungibles::Inspect,
	traits::fungibles::Mutate as MutateFungible,
	traits::tokens::fungibles::Inspect as FungibleInspect,
	traits::tokens::nonfungibles::Inspect as NonFungibleInspect,
	traits::tokens::nonfungibles::Mutate as NonFungibleMutate, traits::Currency,
	traits::ReservableCurrency,
};
use frame_system::pallet_prelude::OriginFor;
use frame_system::pallet_prelude::*;
pub use pallet::*;
use pallet_dex::traits::Swap;
use sp_runtime::{traits::AtLeast32BitUnsigned, traits::Bounded};
pub use types::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod functions;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
mod types;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type AssetIdOf<T> =
	<<T as Config>::Assets as FungibleInspect<<T as frame_system::Config>::AccountId>>::AssetId;
type BalanceOf<T> =
	<<T as Config>::Assets as FungibleInspect<<T as frame_system::Config>::AccountId>>::Balance;
type CollectionIdOf<T> = <<T as Config>::Uniques as NonFungibleInspect<
	<T as frame_system::Config>::AccountId,
>>::CollectionId;
type ItemIdOf<T> =
	<<T as Config>::Uniques as NonFungibleInspect<<T as frame_system::Config>::AccountId>>::ItemId;
type NativeBalanceOf<T> =
	<<T as Config>::NativeCurrency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type PriceOf<T> =
	<<T as Config>::Assets as FungibleInspect<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::traits::tokens::nonfungibles::Transfer;
	use pallet_dex::traits::Price;

	/// The configuration for the pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		// Identifier type for a fungible asset
		type AssetId: AtLeast32BitUnsigned
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
			+ PartialOrd;

		// Balance inspection for fungible assets
		type Assets: FungibleInspect<
				Self::AccountId,
				AssetId = Self::AssetId,
				Balance = NativeBalanceOf<Self>,
			> + MutateFungible<
				Self::AccountId,
				AssetId = Self::AssetId,
				Balance = NativeBalanceOf<Self>,
			>;

		/// Identifier type for a collection of items
		type CollectionId: Member + Parameter + MaxEncodedLen + Copy;

		// Auto-swapping to facilitate buying/selling using any asset/token.
		type DEX: Swap<
				Self::AccountId,
				AssetId = Self::AssetId,
				Balance = <Self::Assets as FungibleInspect<
					<Self as frame_system::Config>::AccountId,
				>>::Balance,
			> + Price<
				AssetId = Self::AssetId,
				Balance = <Self::Assets as FungibleInspect<
					<Self as frame_system::Config>::AccountId,
				>>::Balance,
			>;

		/// The type used to identify a unique item within a collection
		type ItemId: Member + Parameter + MaxEncodedLen + Copy;

		/// Identifier of the native asset identifier (proxy between native token and asset)
		#[pallet::constant]
		type NativeAssetId: Get<Self::AssetId>;

		// Native currency: for swaps between native token and other assets
		type NativeCurrency: ReservableCurrency<Self::AccountId>;

		// Balance inspection for non-fungible assets
		type Uniques: NonFungibleMutate<
				Self::AccountId,
				CollectionId = Self::CollectionId,
				ItemId = Self::ItemId,
			> + Transfer<Self::AccountId, CollectionId = Self::CollectionId, ItemId = Self::ItemId>;

		// Call out to runtime to have it provide result
		// NOTE: no easy way to determine if an asset exists via loose-coupling, so this provides a simple layer of
		// indirection to work around this without tight coupling to the assets pallet
		fn exists(id: Self::AssetId) -> bool;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Stores item listings based on composite key of collection/item.
	#[pallet::storage]
	pub(super) type CollectionListings<T: Config> =
		StorageMap<_, Twox64Concat, CollectionIdOf<T>, CollectionListing<T>>;

	/// Stores item listings based on composite key of collection/item.
	#[pallet::storage]
	pub(super) type ItemListings<T: Config> =
		StorageMap<_, Twox64Concat, (CollectionIdOf<T>, ItemIdOf<T>), ItemListing<T>>;

	// The various events emitted by the pallet.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A collection was listed for minting. [collection, mint_price, asset]
		CollectionListed(CollectionIdOf<T>, PriceOf<T>, AssetIdOf<T>),
		/// A collection was delisted from minting. [collection]
		CollectionDelisted(CollectionIdOf<T>),
		/// A collection was listed for sale. [collection, item, price, asset]
		ItemListed(CollectionIdOf<T>, ItemIdOf<T>, PriceOf<T>, AssetIdOf<T>),
		/// A collection item was delisted. [collection, item]
		ItemDelisted(CollectionIdOf<T>, ItemIdOf<T>),
		/// An item was minted. [collection, item, price, asset]
		Minted(CollectionIdOf<T>, ItemIdOf<T>, PriceOf<T>, AssetIdOf<T>),
		/// An item was sold. [collection, item, price, asset]
		Sold(CollectionIdOf<T>, ItemIdOf<T>, PriceOf<T>, AssetIdOf<T>),
	}

	// The various errors returned by the pallet.
	#[pallet::error]
	pub enum Error<T> {
		// The current balance is insufficient.
		InsufficientBalance,
		/// The asset does not exist.
		InvalidAsset,
		/// The item does not exist.
		InvalidItem,
		// The collection item has already been minted.
		ItemAlreadyMinted,
		/// The asset is not currently owned.
		NoOwnership,
		/// The item is not listed.
		NoListing,
	}

	// The various calls made available by the pallet (dispatchable functions which materialize as "extrinsics").
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Sets a collection price for minting via the marketplace.
		/// # Arguments
		/// * `origin` - The origin of the call.
		/// * `collection` - The collection identifier.
		/// * `mint_price` - The mint price of an item.
		/// * `asset` - The asset type of the listing.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn list_collection(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			mint_price: PriceOf<T>,
			asset: AssetIdOf<T>,
		) -> DispatchResult {
			// Check signed
			let who = ensure_signed(origin)?;
			// Check inputs
			ensure!(
				T::Uniques::collection_owner(&collection).map_or(false, |account| account == who),
				Error::<T>::NoOwnership
			); // Ensure owner
			ensure!(T::exists(asset), Error::<T>::InvalidAsset); // Ensure assets exists

			// Create a listing
			let listing = CollectionListing { price: mint_price, asset };
			<CollectionListings<T>>::set(collection, Some(listing));
			Self::deposit_event(Event::CollectionListed(collection, mint_price, asset));
			Ok(())
		}

		/// Lists a collection item for sale.
		/// # Arguments
		/// * `origin` - The origin of the call.
		/// * `collection` - The collection identifier.
		/// * `item` - The collection item identifier.
		/// * `price` - The list price of the item.
		/// * `asset` - The asset type of the listing.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn list_item(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
			price: PriceOf<T>,
			asset: AssetIdOf<T>,
		) -> DispatchResult {
			// Check signed
			let who = ensure_signed(origin)?;
			// Check inputs
			ensure!(
				T::Uniques::owner(&collection, &item).map_or(false, |account| account == who),
				Error::<T>::NoOwnership
			); // Ensure owner
			ensure!(T::exists(asset), Error::<T>::InvalidAsset); // Ensure assets exists

			// Create a listing
			let listing = ItemListing { price, asset };
			<ItemListings<T>>::set((collection, item), Some(listing));
			Self::deposit_event(Event::ItemListed(collection, item, price, asset));
			Ok(())
		}

		/// Mints a collection item.
		/// # Arguments
		/// * `origin` - The origin of the call.
		/// * `collection` - The collection identifier.
		/// * `item` - The collection item identifier.
		/// * `asset` - The asset type to be used to mint.
		/// **Note:** the collection item should ideally be randomly generated, rather than being passed in as an
		/// argument. This _appears_ to be related to the uniques pallet API.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn mint(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
			asset: AssetIdOf<T>,
		) -> DispatchResult {
			let buyer = ensure_signed(origin)?;

			// Check inputs
			ensure!(T::Uniques::owner(&collection, &item).is_none(), Error::<T>::ItemAlreadyMinted);

			// Lookup collection listing
			match <CollectionListings<T>>::get(collection) {
				None => return Err(DispatchError::from(Error::<T>::NoListing)),
				Some(listing) => {
					match T::Uniques::collection_owner(&collection) {
						None => return Err(DispatchError::from(Error::<T>::InvalidItem)),
						Some(owner) => {
							// Check if asset matches listing
							if listing.asset == asset {
								// Ensure buyer has sufficient funds
								ensure!(
									Self::balance(asset, &buyer) >= listing.price,
									<Error<T>>::InsufficientBalance
								);
								// Exchange funds for unique
								Self::transfer(asset, &buyer, &owner, listing.price)?;
								T::Uniques::mint_into(&collection, &item, &buyer)?;
							} else {
								let swap_price =
									T::DEX::price(listing.price, listing.asset, asset)?;
								// Ensure buyer has sufficient funds
								ensure!(
									Self::balance(asset, &buyer) >= swap_price,
									<Error<T>>::InsufficientBalance
								);

								// Swap the funds via the dex
								// todo: DEX needs functionality to be able to specify minimum quantity returned
								T::DEX::swap(swap_price, asset, listing.asset, &buyer)?;

								// Exchange funds for unique
								Self::transfer(listing.asset, &buyer, &owner, listing.price)?;
								T::Uniques::mint_into(&collection, &item, &buyer)?;
							}
						},
					}
				},
			}

			// Finally remove listing and emit event
			<ItemListings<T>>::remove((collection, item));
			Self::deposit_event(Event::ItemDelisted(collection, item));
			Ok(())
		}

		/// Delists a collection from sale.
		/// # Arguments
		/// * `origin` - The origin of the call.
		/// * `collection` - The collection identifier.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn delist_collection(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
		) -> DispatchResult {
			// Check signed
			let who = ensure_signed(origin)?;

			match T::Uniques::collection_owner(&collection) {
				None => Err(DispatchError::from(Error::<T>::InvalidItem)),
				Some(account) => {
					// Ensure owner
					ensure!(account == who, Error::<T>::NoOwnership);

					// Check for listing and delist if found
					if let Some(_) = <CollectionListings<T>>::get(collection) {
						<CollectionListings<T>>::remove(collection);
						Self::deposit_event(Event::CollectionDelisted(collection));
					}

					Ok(())
				},
			}
		}

		/// Delists a collection item from sale.
		/// # Arguments
		/// * `origin` - The origin of the call.
		/// * `collection` - The collection identifier.
		/// * `item` - The collection item identifier.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn delist_item(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
		) -> DispatchResult {
			// Check signed
			let who = ensure_signed(origin)?;

			match T::Uniques::owner(&collection, &item) {
				None => Err(DispatchError::from(Error::<T>::InvalidItem)),
				Some(account) => {
					// Ensure owner
					ensure!(account == who, Error::<T>::NoOwnership);

					// Check for listing and delist if found
					if let Some(_) = <ItemListings<T>>::get((collection, item)) {
						<ItemListings<T>>::remove((collection, item));
						Self::deposit_event(Event::ItemDelisted(collection, item));
					}

					Ok(())
				},
			}
		}

		/// Purchases a collection item, using the specified asset type.
		/// # Arguments
		/// * `origin` - The origin of the call.
		/// * `collection` - The collection identifier.
		/// * `item` - The collection item identifier.
		/// * `asset` - The asset type to be used to conclude the purchase.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn purchase(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
			asset: AssetIdOf<T>,
		) -> DispatchResult {
			let buyer = ensure_signed(origin)?;

			// Lookup item listing
			match <ItemListings<T>>::get((collection, item)) {
				None => return Err(DispatchError::from(Error::<T>::NoListing)),
				Some(listing) => {
					match T::Uniques::owner(&collection, &item) {
						None => return Err(DispatchError::from(Error::<T>::InvalidItem)),
						Some(owner) => {
							// Check if asset matches listing
							if listing.asset == asset {
								// Ensure buyer has sufficient funds
								ensure!(
									Self::balance(asset, &buyer) >= listing.price,
									<Error<T>>::InsufficientBalance
								);
								// Exchange funds for unique
								Self::transfer(asset, &buyer, &owner, listing.price)?;
								T::Uniques::transfer(&collection, &item, &buyer)?;
							} else {
								let swap_price =
									T::DEX::price(listing.price, listing.asset, asset)?;
								// Ensure buyer has sufficient funds
								ensure!(
									Self::balance(asset, &buyer) >= swap_price,
									<Error<T>>::InsufficientBalance
								);

								// Swap the funds via the dex
								// todo: DEX needs functionality to be able to specify minimum quantity returned
								T::DEX::swap(swap_price, asset, listing.asset, &buyer)?;

								// Exchange funds for unique
								Self::transfer(listing.asset, &buyer, &owner, listing.price)?;
								T::Uniques::transfer(&collection, &item, &buyer)?;
							}
						},
					}
				},
			}

			// Finally remove listing and emit event
			<ItemListings<T>>::remove((collection, item));
			Self::deposit_event(Event::ItemDelisted(collection, item));
			Ok(())
		}
	}
}
