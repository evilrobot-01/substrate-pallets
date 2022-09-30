use super::*;
use frame_support::traits::{ExistenceRequirement, Get};

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
			T::Assets::teleport(asset, source, destination, amount).map(|_| ())
		}
	}
}
