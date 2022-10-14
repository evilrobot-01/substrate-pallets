use crate::mock::*;
use crate::{AccountIdOf, AssetIdOf, BalanceOf, CollectionListings, Error, ItemListings};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError;

const COLLECTION: u64 = 1;
const ITEM: u32 = 100;
const INVALID_ASSET: u32 = 999;
const LIST_PRICE: u128 = 2;
const MINT_PRICE: u128 = 1;
const VAULT: u64 = 321;

#[test]
fn list_collection_ensures_signed() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Marketplace::list_collection(
                RuntimeOrigin::none(),
                COLLECTION,
                MINT_PRICE,
                NATIVE_TOKEN
            ),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn list_collection_ensures_collection_exists() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Marketplace::list_collection(
                RuntimeOrigin::signed(0),
                COLLECTION,
                MINT_PRICE,
                NATIVE_TOKEN
            ),
            Error::<Test>::InvalidCollection
        );
    });
}

#[test]
fn list_collection_ensures_owner() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));

        assert_noop!(
            Marketplace::list_collection(
                RuntimeOrigin::signed(0),
                COLLECTION,
                MINT_PRICE,
                NATIVE_TOKEN
            ),
            Error::<Test>::NoOwnership
        );
    });
}

#[test]
fn list_collection_ensures_asset_exists() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));

        assert_noop!(
            Marketplace::list_collection(
                RuntimeOrigin::signed(OWNER),
                COLLECTION,
                MINT_PRICE,
                INVALID_ASSET
            ),
            Error::<Test>::InvalidAsset
        );
    });
}

#[test]
fn lists_collection() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));

        assert_ok!(Marketplace::list_collection(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            MINT_PRICE,
            NATIVE_TOKEN
        ));

        let listing = CollectionListings::<Test>::get(COLLECTION).unwrap();
        assert_eq!(MINT_PRICE, listing.mint_price);
        assert_eq!(NATIVE_TOKEN, listing.asset);
    });
}

#[test]
fn list_item_ensures_signed() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Marketplace::list_item(
                RuntimeOrigin::none(),
                COLLECTION,
                ITEM,
                LIST_PRICE,
                NATIVE_TOKEN
            ),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn list_item_ensures_collection_exists() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Marketplace::list_item(
                RuntimeOrigin::signed(OWNER),
                COLLECTION,
                ITEM,
                LIST_PRICE,
                NATIVE_TOKEN
            ),
            Error::<Test>::InvalidCollection
        );
    });
}

#[test]
fn list_item_ensures_item_exists() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));

        assert_noop!(
            Marketplace::list_item(
                RuntimeOrigin::signed(OWNER),
                COLLECTION,
                ITEM,
                LIST_PRICE,
                NATIVE_TOKEN
            ),
            Error::<Test>::InvalidItem
        );
    });
}

#[test]
fn list_item_ensures_asset_exists() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));
        assert_ok!(Uniques::mint(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            ITEM,
            OWNER
        ));

        assert_noop!(
            Marketplace::list_item(
                RuntimeOrigin::signed(OWNER),
                COLLECTION,
                ITEM,
                LIST_PRICE,
                INVALID_ASSET
            ),
            Error::<Test>::InvalidAsset
        );
    });
}

#[test]
fn lists_item() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));
        assert_ok!(Uniques::mint(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            ITEM,
            OWNER
        ));

        assert_ok!(Marketplace::list_item(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            ITEM,
            LIST_PRICE,
            NATIVE_TOKEN
        ));

        let listing = ItemListings::<Test>::get((COLLECTION, ITEM)).unwrap();
        assert_eq!(LIST_PRICE, listing.list_price);
        assert_eq!(NATIVE_TOKEN, listing.asset);
    });
}

#[test]
fn listed_item_cannot_be_transferred() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));
        assert_ok!(Uniques::mint(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            ITEM,
            OWNER
        ));
        assert_ok!(Marketplace::list_item(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            ITEM,
            LIST_PRICE,
            NATIVE_TOKEN
        ));
        assert!(ItemListings::<Test>::get((COLLECTION, ITEM)).is_some());

        assert_noop!(
            Uniques::transfer(RuntimeOrigin::signed(OWNER), COLLECTION, ITEM, VAULT),
            pallet_uniques::Error::<Test>::Locked
        );
    });
}

#[test]
fn delist_collection_ensures_signed() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Marketplace::delist_collection(RuntimeOrigin::none(), COLLECTION),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn delist_collection_ensures_collection_exists() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Marketplace::delist_collection(RuntimeOrigin::signed(0), COLLECTION),
            Error::<Test>::InvalidCollection
        );
    });
}

#[test]
fn delist_collection_ensures_owner() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));

        assert_noop!(
            Marketplace::delist_collection(RuntimeOrigin::signed(0), COLLECTION),
            Error::<Test>::NoOwnership
        );
    });
}

#[test]
fn delists_collection() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));

        // Create collection listing
        assert_ok!(Marketplace::list_collection(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            MINT_PRICE,
            NATIVE_TOKEN
        ));
        assert!(CollectionListings::<Test>::get(COLLECTION).is_some());

        // Delist collection
        assert_ok!(Marketplace::delist_collection(
            RuntimeOrigin::signed(OWNER),
            COLLECTION
        ));
        assert!(CollectionListings::<Test>::get(COLLECTION).is_none());
    });
}

#[test]
fn delist_item_ensures_signed() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Marketplace::delist_item(RuntimeOrigin::none(), COLLECTION, ITEM),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn delist_item_ensures_collection_exists() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Marketplace::delist_item(RuntimeOrigin::signed(OWNER), COLLECTION, ITEM,),
            Error::<Test>::InvalidCollection
        );
    });
}

#[test]
fn delist_item_ensures_item_exists() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));

        assert_noop!(
            Marketplace::delist_item(RuntimeOrigin::signed(OWNER), COLLECTION, ITEM,),
            Error::<Test>::InvalidItem
        );
    });
}

#[test]
fn delists_item() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));
        assert_ok!(Uniques::mint(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            ITEM,
            OWNER
        ));

        // Create item listing
        assert_ok!(Marketplace::list_item(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            ITEM,
            LIST_PRICE,
            NATIVE_TOKEN
        ));
        assert!(ItemListings::<Test>::get((COLLECTION, ITEM)).is_some());

        // Delist item
        assert_ok!(Marketplace::delist_item(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            ITEM,
        ));
        assert!(ItemListings::<Test>::get((COLLECTION, ITEM)).is_none());
    });
}

#[test]
fn purchase_ensures_signed() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Marketplace::purchase(RuntimeOrigin::none(), COLLECTION, ITEM, NATIVE_TOKEN),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn purchase_ensures_collection_exists() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Marketplace::purchase(RuntimeOrigin::signed(BUYER), COLLECTION, ITEM, NATIVE_TOKEN),
            Error::<Test>::InvalidCollection
        );
    });
}

#[test]
fn purchase_ensures_item_exists() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));

        assert_noop!(
            Marketplace::purchase(RuntimeOrigin::signed(BUYER), COLLECTION, ITEM, NATIVE_TOKEN),
            Error::<Test>::InvalidItem
        );
    });
}

#[test]
fn purchase_ensures_listing_exists() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));
        assert_ok!(Uniques::mint(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            ITEM,
            OWNER
        ));

        assert_noop!(
            Marketplace::purchase(RuntimeOrigin::signed(BUYER), COLLECTION, ITEM, NATIVE_TOKEN),
            Error::<Test>::NoListing
        );
    });
}

#[test]
fn purchase_ensures_sufficient_balance() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));
        assert_ok!(Uniques::mint(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            ITEM,
            OWNER
        ));
        assert_ok!(Marketplace::list_item(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            ITEM,
            balance(NATIVE_TOKEN, &BUYER) + 1,
            NATIVE_TOKEN
        ));

        assert_noop!(
            Marketplace::purchase(RuntimeOrigin::signed(BUYER), COLLECTION, ITEM, NATIVE_TOKEN),
            Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn purchase_transfers_funds() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));
        assert_ok!(Uniques::mint(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            ITEM,
            OWNER
        ));
        assert_ok!(Marketplace::list_item(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            ITEM,
            LIST_PRICE,
            NATIVE_TOKEN
        ));

        let owner_balance = balance(NATIVE_TOKEN, &OWNER);
        let buyer_balance = balance(NATIVE_TOKEN, &BUYER);
        assert_ok!(Marketplace::purchase(
            RuntimeOrigin::signed(BUYER),
            COLLECTION,
            ITEM,
            NATIVE_TOKEN
        ),);

        // Ensure balances changed
        assert_eq!(balance(NATIVE_TOKEN, &OWNER), owner_balance + LIST_PRICE);
        assert_eq!(balance(NATIVE_TOKEN, &BUYER), buyer_balance - LIST_PRICE);
    });
}

#[test]
fn purchase_removes_listing() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));
        assert_ok!(Uniques::mint(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            ITEM,
            OWNER
        ));
        assert_ok!(Marketplace::list_item(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            ITEM,
            LIST_PRICE,
            NATIVE_TOKEN
        ));

        assert_ok!(Marketplace::purchase(
            RuntimeOrigin::signed(BUYER),
            COLLECTION,
            ITEM,
            NATIVE_TOKEN
        ),);

        // Ensure listing removed
        assert!(ItemListings::<Test>::get((COLLECTION, ITEM)).is_none());
    });
}

#[test]
fn purchase_transfers_item() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));
        assert_ok!(Uniques::mint(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            ITEM,
            OWNER
        ));
        assert_ok!(Marketplace::list_item(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            ITEM,
            LIST_PRICE,
            NATIVE_TOKEN
        ));

        assert_ok!(Marketplace::purchase(
            RuntimeOrigin::signed(BUYER),
            COLLECTION,
            ITEM,
            NATIVE_TOKEN
        ),);

        // Ensure owner of unique has changed
        assert_eq!(Uniques::owner(COLLECTION, ITEM).unwrap(), BUYER);
    });
}

#[test]
fn purchase_via_swap_ensures_sufficient_balance() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));
        assert_ok!(Uniques::mint(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            ITEM,
            OWNER
        ));
        assert_ok!(Marketplace::list_item(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            ITEM,
            20,
            NATIVE_TOKEN
        ));

        assert_noop!(
            Marketplace::purchase(RuntimeOrigin::signed(BUYER), COLLECTION, ITEM, ASSET_1),
            Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn purchase_via_swap() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));
        assert_ok!(Uniques::mint(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            ITEM,
            OWNER
        ));
        assert_ok!(Marketplace::list_item(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            ITEM,
            LIST_PRICE,
            NATIVE_TOKEN
        ));

        let owner_balance = balance(NATIVE_TOKEN, &OWNER);
        let buyer_balance = balance(ASSET_1, &BUYER);
        let swap_price = DEX::price(LIST_PRICE, NATIVE_TOKEN, ASSET_1).unwrap();

        assert_ok!(Marketplace::purchase(
            RuntimeOrigin::signed(BUYER),
            COLLECTION,
            ITEM,
            ASSET_1
        ),);

        // Ensure balances changed
        assert_eq!(balance(NATIVE_TOKEN, &OWNER), owner_balance + LIST_PRICE);
        assert_eq!(balance(ASSET_1, &BUYER), buyer_balance - swap_price);
    });
}

#[test]
fn mint_ensures_signed() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Marketplace::mint(RuntimeOrigin::none(), COLLECTION, ITEM, NATIVE_TOKEN),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn mint_ensures_collection_exists() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Marketplace::mint(RuntimeOrigin::signed(BUYER), COLLECTION, ITEM, NATIVE_TOKEN),
            Error::<Test>::InvalidCollection
        );
    });
}

#[test]
fn mint_ensures_item_not_already_minted() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));
        assert_ok!(Uniques::mint(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            ITEM,
            OWNER
        ));

        assert_noop!(
            Marketplace::mint(RuntimeOrigin::signed(BUYER), COLLECTION, ITEM, NATIVE_TOKEN),
            Error::<Test>::ItemAlreadyMinted
        );
    });
}

#[test]
fn mint_ensures_collection_listed() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));

        assert_noop!(
            Marketplace::mint(RuntimeOrigin::signed(BUYER), COLLECTION, ITEM, NATIVE_TOKEN),
            Error::<Test>::NoListing
        );
    });
}

#[test]
fn mint_ensures_sufficient_balance() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));
        assert_ok!(Marketplace::list_collection(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            balance(NATIVE_TOKEN, &BUYER) + 1, // Ensure buyer cannot afford to mint
            NATIVE_TOKEN
        ));

        assert_noop!(
            Marketplace::mint(RuntimeOrigin::signed(BUYER), COLLECTION, ITEM, NATIVE_TOKEN),
            Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn mint_transfers_funds() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));
        assert_ok!(Marketplace::list_collection(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            MINT_PRICE,
            NATIVE_TOKEN
        ));

        let owner_balance = balance(NATIVE_TOKEN, &OWNER);
        let buyer_balance = balance(NATIVE_TOKEN, &BUYER);
        assert_ok!(Marketplace::mint(
            RuntimeOrigin::signed(BUYER),
            COLLECTION,
            ITEM,
            NATIVE_TOKEN
        ));

        // Ensure balances changed
        assert_eq!(balance(NATIVE_TOKEN, &OWNER), owner_balance + MINT_PRICE);
        assert_eq!(balance(NATIVE_TOKEN, &BUYER), buyer_balance - MINT_PRICE);
    });
}

#[test]
fn mint_transfers_item() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));
        assert_ok!(Marketplace::list_collection(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            MINT_PRICE,
            NATIVE_TOKEN
        ));

        assert_ok!(Marketplace::mint(
            RuntimeOrigin::signed(BUYER),
            COLLECTION,
            ITEM,
            NATIVE_TOKEN
        ));

        // Ensure owner of unique has changed
        assert_eq!(Uniques::owner(COLLECTION, ITEM).unwrap(), BUYER);
    });
}

#[test]
fn mint_via_swap_ensures_sufficient_balance() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));
        assert_ok!(Marketplace::list_collection(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            20,
            NATIVE_TOKEN
        ));

        assert_noop!(
            Marketplace::mint(RuntimeOrigin::signed(BUYER), COLLECTION, ITEM, ASSET_1),
            Error::<Test>::InsufficientBalance
        );
    });
}

#[test]
fn mint_via_swap() {
    new_test_ext().execute_with(|| {
        assert_ok!(Uniques::create(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            OWNER
        ));
        assert_ok!(Marketplace::list_collection(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            MINT_PRICE,
            NATIVE_TOKEN
        ));

        let owner_balance = balance(NATIVE_TOKEN, &OWNER);
        let buyer_balance = balance(ASSET_1, &BUYER);
        let swap_price = DEX::price(MINT_PRICE, NATIVE_TOKEN, ASSET_1).unwrap();

        assert_ok!(Marketplace::mint(
            RuntimeOrigin::signed(BUYER),
            COLLECTION,
            ITEM,
            ASSET_1
        ),);

        // Ensure balances changed
        assert_eq!(balance(NATIVE_TOKEN, &OWNER), owner_balance + MINT_PRICE);
        assert_eq!(balance(ASSET_1, &BUYER), buyer_balance - swap_price);
    });
}

fn balance(id: AssetIdOf<Test>, who: &AccountIdOf<Test>) -> BalanceOf<Test> {
    crate::Pallet::<Test>::balance(id, who)
}
