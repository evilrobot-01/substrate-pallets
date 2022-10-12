use crate::mock::*;
use crate::{CollectionListings, Error, ItemListings};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError;

const ADMIN: u64 = 1;
const ASSET_0: u32 = 1;
//const ASSET_1: u32 = 2;
const COLLECTION: u64 = 1;
const ITEM: u32 = 100;
const LIST_PRICE: u128 = 2;
const MIN_BALANCE: u128 = 1;
const MINT_PRICE: u128 = 1;
const OWNER: u64 = 123;

#[test]
fn list_collection_ensures_signed() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Marketplace::list_collection(RuntimeOrigin::none(), COLLECTION, MINT_PRICE, ASSET_0),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn list_collection_ensures_collection_exists() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Marketplace::list_collection(RuntimeOrigin::signed(0), COLLECTION, MINT_PRICE, ASSET_0),
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
            Marketplace::list_collection(RuntimeOrigin::signed(0), COLLECTION, MINT_PRICE, ASSET_0),
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
                ASSET_0
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
        assert_ok!(Assets::force_create(
            RuntimeOrigin::root(),
            ASSET_0,
            ADMIN,
            true,
            MIN_BALANCE
        ));

        assert_ok!(Marketplace::list_collection(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            MINT_PRICE,
            ASSET_0
        ));

        let listing = CollectionListings::<Test>::get(COLLECTION).unwrap();
        assert_eq!(MINT_PRICE, listing.price);
        assert_eq!(ASSET_0, listing.asset);
    });
}

#[test]
fn list_item_ensures_signed() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Marketplace::list_item(RuntimeOrigin::none(), COLLECTION, ITEM, LIST_PRICE, ASSET_0),
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
                ASSET_0
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
                ASSET_0
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
                ASSET_0
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
        assert_ok!(Assets::force_create(
            RuntimeOrigin::root(),
            ASSET_0,
            ADMIN,
            true,
            MIN_BALANCE
        ));

        assert_ok!(Marketplace::list_item(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            ITEM,
            LIST_PRICE,
            ASSET_0
        ));

        let listing = ItemListings::<Test>::get((COLLECTION, ITEM)).unwrap();
        assert_eq!(LIST_PRICE, listing.price);
        assert_eq!(ASSET_0, listing.asset);
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
        assert_ok!(Assets::force_create(
            RuntimeOrigin::root(),
            ASSET_0,
            ADMIN,
            true,
            MIN_BALANCE
        ));

        // Create collection listing
        assert_ok!(Marketplace::list_collection(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            MINT_PRICE,
            ASSET_0
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
        assert_ok!(Assets::force_create(
            RuntimeOrigin::root(),
            ASSET_0,
            ADMIN,
            true,
            MIN_BALANCE
        ));

        // Create item listing
        assert_ok!(Marketplace::list_item(
            RuntimeOrigin::signed(OWNER),
            COLLECTION,
            ITEM,
            LIST_PRICE,
            ASSET_0
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
