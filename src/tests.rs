use crate::{mock::*, Error, LiquidityPools};
use frame_support::traits::fungibles::Inspect;
use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError;
use std::time::{SystemTime, UNIX_EPOCH};

const ADMIN: u64 = 1;
const ANOTHER_LP: u64 = 321; // Liquidity Provider
const ASSET_0: u32 = 1;
const ASSET_1: u32 = 2;
const BUYER: u64 = 827364;
const DEADLINE: u64 = u64::MAX;
const INVALID_ASSET: u32 = 21762531;
const LP: u64 = 123; // Liquidity Provider
const MIN_BALANCE: u128 = 1;
const UNITS: u128 = 1000;

// NOTE: type-specific tests located in types.rs

#[test]
fn add_liquidity_ensures_signed() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			DEX::add_liquidity(Origin::none(), 0, ASSET_0, 0, ASSET_1, DEADLINE),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn add_liquidity_ensure_assets_unique() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			DEX::add_liquidity(Origin::signed(LP), 0, ASSET_0, 0, ASSET_0, DEADLINE),
			Error::<Test>::IdenticalAssets
		);
	});
}

#[test]
fn add_liquidity_ensure_amount_0_valid() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			DEX::add_liquidity(Origin::signed(LP), 0, ASSET_0, 10 * UNITS, ASSET_1, DEADLINE),
			Error::<Test>::InvalidAmount
		);
	});
}

#[test]
fn add_liquidity_ensure_amount_1_valid() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			DEX::add_liquidity(Origin::signed(LP), 1 * UNITS, ASSET_0, 0, ASSET_1, DEADLINE),
			Error::<Test>::InvalidAmount
		);
	});
}

#[test]
fn add_liquidity_ensure_asset_0_valid() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_noop!(
			DEX::add_liquidity(
				Origin::signed(LP),
				10 * UNITS,
				INVALID_ASSET,
				20 * UNITS,
				ASSET_0,
				DEADLINE
			),
			Error::<Test>::InvalidAsset
		);
	});
}

#[test]
fn add_liquidity_ensure_asset_1_valid() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_noop!(
			DEX::add_liquidity(
				Origin::signed(LP),
				10 * UNITS,
				ASSET_0,
				20 * UNITS,
				INVALID_ASSET,
				DEADLINE
			),
			Error::<Test>::InvalidAsset
		);
	});
}

#[test]
fn add_liquidity_ensure_asset_0_balance_sufficient() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
		assert_noop!(
			DEX::add_liquidity(
				Origin::signed(LP),
				10 * UNITS,
				ASSET_1,
				20 * UNITS,
				ASSET_0,
				DEADLINE
			),
			Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn add_liquidity_ensure_asset_1_balance_sufficient() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, LP, 100 * UNITS));
		assert_noop!(
			DEX::add_liquidity(
				Origin::signed(LP),
				10 * UNITS,
				ASSET_0,
				20 * UNITS,
				ASSET_1,
				DEADLINE
			),
			Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn add_liquidity_ensure_within_deadline() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, LP, 100 * UNITS));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, LP, 100 * UNITS));

		let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
		Timestamp::set_timestamp(now);

		assert_noop!(
			DEX::add_liquidity(
				Origin::signed(LP),
				10 * UNITS,
				ASSET_0,
				20 * UNITS,
				ASSET_1,
				now - 10
			),
			Error::<Test>::DeadlinePassed
		);
	});
}

#[test]
fn add_liquidity_ensure_liquidity_pool_id() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, LP, 100 * UNITS));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, LP, 100 * UNITS));

		// Create liquidity pool asset in advance
		assert_ok!(Assets::create(Origin::signed(1), u32::MAX, ADMIN, MIN_BALANCE));
		assert_noop!(
			DEX::add_liquidity(
				Origin::signed(LP),
				10 * UNITS,
				ASSET_0,
				20 * UNITS,
				ASSET_1,
				DEADLINE
			),
			Error::<Test>::AssetAlreadyExists
		);
	});
}

#[test]
fn adds_liquidity() {
	new_test_ext().execute_with(|| {
		// Create assets and fund
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, LP, 100 * UNITS));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, LP, 100 * UNITS));

		// Add liquidity to pool
		assert_ok!(DEX::add_liquidity(
			Origin::signed(LP),
			10 * UNITS,
			ASSET_1,
			20 * UNITS,
			ASSET_0, // Intentionally placed lower id in second position to test ordering
			DEADLINE
		));

		// Ensure liquidity pool (and token) token created as asset
		let pool = LiquidityPools::<Test>::get((ASSET_0, ASSET_1)).unwrap();
		assert!(Assets::maybe_total_supply(pool.id).is_some());

		// Check resulting balances
		assert_eq!(Assets::balance(ASSET_0, &LP), 80 * UNITS);
		assert_eq!(Assets::balance(ASSET_1, &LP), 90 * UNITS);
		assert_eq!(Assets::balance(pool.id, &LP), 20 * UNITS);

		check_pool_balances((ASSET_0, ASSET_1), (20 * UNITS, 10 * UNITS, 20 * UNITS));
	});
}

#[test]
fn adds_more_liquidity() {
	new_test_ext().execute_with(|| {
		// Create assets and fund
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, LP, 100 * UNITS));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, LP, 100 * UNITS));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, ANOTHER_LP, 2 * UNITS));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, ANOTHER_LP, 1 * UNITS));

		// Add liquidity to pool
		assert_ok!(DEX::add_liquidity(
			Origin::signed(LP),
			10 * UNITS,
			ASSET_1,
			20 * UNITS,
			ASSET_0, // Intentionally placed lower id in second position to test ordering
			DEADLINE
		));

		// Ensure liquidity pool (and token) token created as asset
		let pool = LiquidityPools::<Test>::get((ASSET_0, ASSET_1)).unwrap();
		assert!(Assets::maybe_total_supply(pool.id).is_some());

		// Check resulting balances
		assert_eq!(Assets::balance(ASSET_0, &LP), 80 * UNITS);
		assert_eq!(Assets::balance(ASSET_1, &LP), 90 * UNITS);
		assert_eq!(Assets::balance(pool.id, &LP), 20 * UNITS);
		check_pool_balances((ASSET_0, ASSET_1), (20 * UNITS, 10 * UNITS, 20 * UNITS));

		// Add more liquidity to pool
		assert_ok!(DEX::add_liquidity(
			Origin::signed(ANOTHER_LP),
			2 * UNITS,
			ASSET_0,
			1 * UNITS,
			ASSET_1,
			DEADLINE
		));

		// Check resulting balances
		assert_eq!(Assets::balance(ASSET_0, &ANOTHER_LP), 0);
		assert_eq!(Assets::balance(ASSET_1, &ANOTHER_LP), 0);
		assert_eq!(Assets::balance(pool.id, &ANOTHER_LP), 2 * UNITS);
		check_pool_balances((ASSET_0, ASSET_1), (22 * UNITS, 11 * UNITS, 22 * UNITS));
	});
}

#[test]
fn remove_liquidity_ensures_signed() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			DEX::remove_liquidity(Origin::none(), 0, ASSET_0, ASSET_1, DEADLINE),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn remove_liquidity_ensure_assets_unique() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			DEX::remove_liquidity(Origin::signed(LP), 0, ASSET_0, ASSET_0, DEADLINE),
			Error::<Test>::IdenticalAssets
		);
	});
}

#[test]
fn remove_liquidity_ensure_amount_valid() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			DEX::remove_liquidity(Origin::signed(LP), 0, ASSET_0, ASSET_1, DEADLINE),
			Error::<Test>::InvalidAmount
		);
	});
}

#[test]
fn remove_liquidity_ensure_within_deadline() {
	new_test_ext().execute_with(|| {
		let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
		Timestamp::set_timestamp(now);

		assert_noop!(
			DEX::remove_liquidity(Origin::signed(LP), 10 * UNITS, ASSET_0, ASSET_1, now - 10),
			Error::<Test>::DeadlinePassed
		);
	});
}

#[test]
fn remove_liquidity_ensure_pool_exists() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			DEX::remove_liquidity(Origin::signed(LP), 10 * UNITS, ASSET_0, ASSET_1, DEADLINE),
			Error::<Test>::NoPool
		);
	});
}

#[test]
fn remove_liquidity_ensure_balance_sufficient() {
	new_test_ext().execute_with(|| {
		// Create assets and fund
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, LP, 100 * UNITS));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, LP, 100 * UNITS));
		assert_ok!(DEX::add_liquidity(
			Origin::signed(LP),
			10 * UNITS,
			ASSET_1,
			20 * UNITS,
			ASSET_0,
			DEADLINE
		));

		assert_noop!(
			DEX::remove_liquidity(Origin::signed(LP), (20 * UNITS) + 1, ASSET_1, ASSET_0, DEADLINE),
			Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn removes_liquidity() {
	new_test_ext().execute_with(|| {
		// Create assets and fund
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, LP, 100 * UNITS));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, LP, 100 * UNITS));
		assert_ok!(DEX::add_liquidity(
			Origin::signed(LP),
			10 * UNITS,
			ASSET_1,
			20 * UNITS,
			ASSET_0, // Intentionally placed lower id in second position to test ordering
			DEADLINE
		));

		// Check resulting balances
		assert_eq!(Assets::balance(ASSET_0, &LP), 80 * UNITS);
		assert_eq!(Assets::balance(ASSET_1, &LP), 90 * UNITS);
		let pool = LiquidityPools::<Test>::get((ASSET_0, ASSET_1)).unwrap();
		assert_eq!(Assets::balance(pool.id, &LP), 20 * UNITS);

		assert_ok!(DEX::remove_liquidity(
			Origin::signed(LP),
			20 * UNITS,
			ASSET_0,
			ASSET_1,
			DEADLINE
		));

		// Check resulting balances
		assert_eq!(Assets::balance(ASSET_0, &LP), 100 * UNITS);
		assert_eq!(Assets::balance(ASSET_1, &LP), 100 * UNITS);
		assert_eq!(Assets::balance(pool.id, &LP), 0);

		check_pool_balances((ASSET_0, ASSET_1), (0, 0, 0));
	});
}

#[test]
fn removes_some_liquidity() {
	new_test_ext().execute_with(|| {
		// Create assets and fund
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, LP, 100 * UNITS));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, LP, 100 * UNITS));
		assert_ok!(DEX::add_liquidity(
			Origin::signed(LP),
			10 * UNITS,
			ASSET_1,
			20 * UNITS,
			ASSET_0, // Intentionally placed lower id in second position to test ordering
			DEADLINE
		));

		// Check resulting balances
		assert_eq!(Assets::balance(ASSET_0, &LP), 80 * UNITS);
		assert_eq!(Assets::balance(ASSET_1, &LP), 90 * UNITS);
		let pool = LiquidityPools::<Test>::get((ASSET_0, ASSET_1)).unwrap();
		assert_eq!(Assets::balance(pool.id, &LP), 20 * UNITS);

		assert_ok!(DEX::remove_liquidity(
			Origin::signed(LP),
			10 * UNITS,
			ASSET_0,
			ASSET_1,
			DEADLINE
		));

		assert_eq!(Assets::balance(ASSET_0, &LP), 90 * UNITS);
		assert_eq!(Assets::balance(ASSET_1, &LP), 95 * UNITS);
		assert_eq!(Assets::balance(pool.id, &LP), 10 * UNITS);
		check_pool_balances((ASSET_0, ASSET_1), (10 * UNITS, 5 * UNITS, 10 * UNITS));
	});
}

#[test]
fn swap_ensures_signed() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			DEX::swap(Origin::none(), 0, ASSET_0, ASSET_1, DEADLINE),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn swap_ensure_within_deadline() {
	new_test_ext().execute_with(|| {
		let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
		Timestamp::set_timestamp(now);

		assert_noop!(
			DEX::swap(Origin::signed(BUYER), 10 * UNITS, ASSET_0, ASSET_1, now - 10),
			Error::<Test>::DeadlinePassed
		);
	});
}

#[test]
fn swap_ensure_amount_valid() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			DEX::swap(Origin::signed(BUYER), 0, ASSET_0, ASSET_1, DEADLINE),
			Error::<Test>::InvalidAmount
		);
	});
}

#[test]
fn swap_ensure_balance_sufficient() {
	new_test_ext().execute_with(|| {
		// Create assets and fund
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, LP, 100 * UNITS));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, LP, 100 * UNITS));

		assert_noop!(
			DEX::swap(Origin::signed(BUYER), (20 * UNITS) + 1, ASSET_1, ASSET_0, DEADLINE),
			Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn swap_ensure_pool_exists() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, BUYER, 100 * UNITS));
		assert_noop!(
			DEX::swap(Origin::signed(BUYER), 10 * UNITS, ASSET_0, ASSET_1, DEADLINE),
			Error::<Test>::NoPool
		);
	});
}

#[test]
fn swap_empty_pool_asset_0() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, LP, 100 * UNITS));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, LP, 100 * UNITS));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, BUYER, 10 * UNITS));
		assert_ok!(DEX::add_liquidity(
			Origin::signed(LP),
			10 * UNITS,
			ASSET_1,
			20 * UNITS,
			ASSET_0,
			DEADLINE
		));
		assert_ok!(DEX::remove_liquidity(
			Origin::signed(LP),
			20 * UNITS,
			ASSET_0,
			ASSET_1,
			DEADLINE
		));
		assert_noop!(
			DEX::swap(Origin::signed(BUYER), 10 * UNITS, ASSET_1, ASSET_0, DEADLINE),
			Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn swap_empty_pool_asset_1() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, LP, 100 * UNITS));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, LP, 100 * UNITS));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, BUYER, 10 * UNITS));
		assert_ok!(DEX::add_liquidity(
			Origin::signed(LP),
			10 * UNITS,
			ASSET_1,
			20 * UNITS,
			ASSET_0,
			DEADLINE
		));
		assert_ok!(DEX::remove_liquidity(
			Origin::signed(LP),
			20 * UNITS,
			ASSET_0,
			ASSET_1,
			DEADLINE
		));
		assert_noop!(
			DEX::swap(Origin::signed(BUYER), 10 * UNITS, ASSET_0, ASSET_1, DEADLINE),
			Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn swaps_asset_0() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, LP, 100 * UNITS));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, LP, 100 * UNITS));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, BUYER, 10 * UNITS));
		assert_ok!(DEX::add_liquidity(
			Origin::signed(LP),
			10 * UNITS,
			ASSET_1,
			20 * UNITS,
			ASSET_0,
			DEADLINE
		));
		assert_ok!(DEX::swap(Origin::signed(BUYER), 10 * UNITS, ASSET_0, ASSET_1, DEADLINE));

		// Check resulting balances
		assert_eq!(Assets::balance(ASSET_0, &BUYER), 0);
		assert_eq!(Assets::balance(ASSET_1, &BUYER), 4992);
		check_pool_balances((ASSET_0, ASSET_1), (30 * UNITS, 5008, 20 * UNITS));
	});
}

#[test]
fn swaps_asset_1() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, LP, 100 * UNITS));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, LP, 100 * UNITS));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, BUYER, 10 * UNITS));
		assert_ok!(DEX::add_liquidity(
			Origin::signed(LP),
			10 * UNITS,
			ASSET_1,
			20 * UNITS,
			ASSET_0,
			DEADLINE
		));
		assert_ok!(DEX::swap(Origin::signed(BUYER), 5 * UNITS, ASSET_1, ASSET_0, DEADLINE));

		// Check resulting balances
		assert_eq!(Assets::balance(ASSET_0, &BUYER), 9984);
		assert_eq!(Assets::balance(ASSET_1, &BUYER), 5 * UNITS);
		check_pool_balances((ASSET_0, ASSET_1), (10016, 15 * UNITS, 20 * UNITS));
	});
}

fn check_pool_balances(pool: (u32, u32), expected: (u128, u128, u128)) {
	let liquidity_pool = LiquidityPools::<Test>::get(pool).unwrap();
	assert_eq!(Assets::balance(pool.0, &liquidity_pool.account), expected.0);
	assert_eq!(Assets::balance(pool.1, &liquidity_pool.account), expected.1);
	assert_eq!(Assets::total_issuance(liquidity_pool.id), expected.2);
}

#[test]
fn gets_price() {
	new_test_ext().execute_with(|| {
		// Create assets and fund
		assert_ok!(Assets::force_create(Origin::root(), ASSET_0, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::force_create(Origin::root(), ASSET_1, ADMIN, true, MIN_BALANCE));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_0, LP, 100 * UNITS));
		assert_ok!(Assets::mint(Origin::signed(ADMIN), ASSET_1, LP, 100 * UNITS));

		// Add liquidity to pool
		assert_ok!(DEX::add_liquidity(
			Origin::signed(LP),
			10 * UNITS,
			ASSET_0,
			20 * UNITS,
			ASSET_1,
			DEADLINE
		));

		// Price a swap
		assert_eq!(DEX::price(5 * UNITS, ASSET_0, ASSET_1).unwrap(), 9984);
	});
}

#[test]
fn price_invalid_pool() {
	new_test_ext().execute_with(|| {
		assert_noop!(DEX::price(5 * UNITS, ASSET_0, ASSET_1), Error::<Test>::NoPool);
	});
}
