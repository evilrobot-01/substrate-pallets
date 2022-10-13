use super::*;
use frame_support::{
    traits::tokens::fungibles::Transfer,
    traits::{ExistenceRequirement, Get},
};

impl<T: Config> Pallet<T> {
    /// Get the balance of asset `id` for `who`.   
    /// **Note:** this is a wrapper function for handling native and custom asset balances.
    pub(super) fn balance(id: AssetIdOf<T>, who: &AccountIdOf<T>) -> BalanceOf<T> {
        // Return balance of native currency if supplied asset id matches configured native asset id
        if id == T::NativeAssetId::get() {
            T::NativeCurrency::total_balance(who)
        } else {
            // Otherwise use asset balance
            T::Assets::balance(id, &who)
        }
    }

    /// Ensures that the collection exists and that the sender is the owner.
    pub(super) fn ensure_collection_owner(
        sender: &AccountIdOf<T>,
        collection: &CollectionIdOf<T>,
    ) -> DispatchResult {
        match T::Uniques::collection_owner(collection) {
            None => Err(DispatchError::from(Error::<T>::InvalidCollection)),
            Some(owner) if sender != &owner => Err(DispatchError::from(Error::<T>::NoOwnership)),
            _ => Ok(()),
        }
    }

    /// Ensures that the collection item exists and that the sender is the owner.
    pub(super) fn ensure_item_owner(
        sender: &AccountIdOf<T>,
        collection: &CollectionIdOf<T>,
        item: &ItemIdOf<T>,
    ) -> DispatchResult {
        match T::Uniques::owner(collection, item) {
            // Ensure item exists and sender is owner
            None => Err(DispatchError::from(Error::<T>::InvalidItem)),
            Some(owner) if sender != &owner => Err(DispatchError::from(Error::<T>::NoOwnership)),
            _ => Ok(()),
        }
    }

    /// Transfer `amount` of `asset` from the `source` account to the `destination` account.
    /// **Note:** this is a wrapper function for handling native and custom asset transfers.
    pub(super) fn transfer(
        asset: AssetIdOf<T>,
        source: &AccountIdOf<T>,
        destination: &AccountIdOf<T>,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        // Use native currency if supplied asset id matches configured native asset id
        if asset == T::NativeAssetId::get() {
            T::NativeCurrency::transfer(
                source,
                destination,
                amount,
                ExistenceRequirement::AllowDeath,
            )
        } else {
            // Otherwise use asset transfer.
            T::Assets::transfer(asset, source, destination, amount, false).map(|_| ())
        }
    }
}
