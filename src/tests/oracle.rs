use super::*;
use crate::{
	types::{Nonce, QueryIdOf, TimestampOf},
	Config, DAY_IN_MILLISECONDS,
};
use frame_support::{assert_noop, assert_ok};
use sp_core::{bounded::BoundedBTreeMap, bounded_btree_map, bounded_vec, Get, U256};
use sp_runtime::traits::BadOrigin;

type ReportingLock = <Test as Config>::ReportingLock;
type WithdrawalPeriod = <Test as Config>::WithdrawalPeriod;
type BoundedReportsSubmittedByQueryId =
	BoundedBTreeMap<QueryIdOf<Test>, u128, <Test as Config>::MaxQueriesPerReporter>;

#[test]
fn deposit_stake() {
	let reporter = 1;
	let address = Address::random();
	let amount = token(100);
	let another_reporter = 2;
	let mut ext = new_test_ext();

	// Prerequisites
	ext.execute_with(|| with_block(|| register_parachain(STAKE_AMOUNT)));

	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L86
	ext.execute_with(|| {
		with_block(|| {
			assert_noop!(
				Tellor::report_stake_deposited(
					RuntimeOrigin::signed(another_reporter),
					reporter,
					amount.into(),
					address
				),
				BadOrigin
			);
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				amount.into(),
				address
			));
			System::assert_last_event(
				Event::NewStakerReported { staker: reporter, amount, address }.into(),
			);

			assert_eq!(Tellor::get_total_stakers(), 1);
			let staker_details = Tellor::get_staker_info(reporter).unwrap();
			assert_eq!(staker_details.address, address);
			assert_eq!(staker_details.start_date, Timestamp::get());
			assert_eq!(staker_details.staked_balance, amount);
			assert_eq!(staker_details.locked_balance, 0);
			assert_eq!(staker_details.reward_debt, 0);
			assert_eq!(staker_details.reporter_last_timestamp, 0);
			assert_eq!(staker_details.reports_submitted, 0);
			assert_eq!(staker_details.start_vote_count, 0);
			assert_eq!(staker_details.start_vote_tally, 0);
			assert_eq!(staker_details.staked, true);
			assert!(staker_details.reports_submitted_by_query_id.is_empty());
			//assert_eq!(Tellor::total_reward_debt(), 0); // todo: total reward debt?
			assert_eq!(Tellor::get_total_stake_amount(), amount);

			// Test min value for amount argument
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				another_reporter,
				0.into(),
				Address::random()
			));
			assert_eq!(Tellor::get_total_stakers(), 1);

			assert_ok!(Tellor::report_staking_withdraw_request(
				Origin::Staking.into(),
				reporter,
				token(5).into(),
				address
			));
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				token(10).into(),
				address
			));
			assert_eq!(Tellor::get_total_stakers(), 1); // Ensure only unique addresses add to total stakers
			let staker_details = Tellor::get_staker_info(reporter).unwrap();
			assert_eq!(staker_details.staked_balance, token(105));
			assert_eq!(staker_details.locked_balance, token(0));
			assert_eq!(Tellor::get_total_stake_amount(), token(105));
		})
	});
}

#[test]
fn remove_value() {
	let query_data: QueryDataOf<Test> = spot_price("dot", "usd").try_into().unwrap();
	let query_id = keccak_256(query_data.as_ref()).into();
	let reporter = 1;
	let another_reporter = 2;
	let address = Address::random();
	let mut ext = new_test_ext();

	// Prerequisites
	ext.execute_with(|| {
		with_block(|| {
			register_parachain(STAKE_AMOUNT);
			super::deposit_stake(another_reporter, STAKE_AMOUNT, Address::random());
		})
	});

	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L127
	ext.execute_with(|| {
		with_block(|| {
			let timestamp = Timestamp::get();

			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				STAKE_AMOUNT.into(),
				address
			));
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(100),
				0,
				query_data.clone(),
			));

			assert_eq!(Tellor::get_new_value_count_by_query_id(query_id), 1);
			assert_noop!(Tellor::remove_value(query_id, 500), Error::InvalidTimestamp);
			assert_eq!(Tellor::retrieve_data(query_id, timestamp).unwrap(), uint_value(100));
			assert!(!Tellor::is_in_dispute(query_id, timestamp));

			// Value can only be removed via dispute
			assert_ok!(Tellor::begin_dispute(
				RuntimeOrigin::signed(another_reporter),
				query_id,
				timestamp
			));
			assert_eq!(Tellor::get_new_value_count_by_query_id(query_id), 1);
			assert_eq!(Tellor::retrieve_data(query_id, timestamp), None);
			assert!(Tellor::is_in_dispute(query_id, timestamp));
			assert_noop!(Tellor::remove_value(query_id, timestamp), Error::ValueDisputed);

			// Test min/max values for timestamp argument
			assert_noop!(Tellor::remove_value(query_id, 0), Error::InvalidTimestamp);
			assert_noop!(Tellor::remove_value(query_id, u64::MAX), Error::InvalidTimestamp);
		});
	});
}

#[test]
fn request_stake_withdraw() {
	let reporter = 1;
	let amount = token(1_000);
	let address = Address::random();
	let mut ext = new_test_ext();

	// Prerequisites
	ext.execute_with(|| with_block(|| register_parachain(STAKE_AMOUNT)));

	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L151
	ext.execute_with(|| {
		with_block(|| {
			assert_noop!(
				Tellor::report_staking_withdraw_request(
					RuntimeOrigin::signed(reporter),
					reporter,
					token(10).into(),
					address
				),
				BadOrigin
			);
			assert_noop!(
				Tellor::report_staking_withdraw_request(
					Origin::Staking.into(),
					reporter,
					token(5).into(),
					address
				),
				Error::InsufficientStake
			);
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				amount.into(),
				address
			));

			let staker_details = Tellor::get_staker_info(reporter).unwrap();
			assert_eq!(staker_details.start_date, Timestamp::get());
			assert_eq!(staker_details.staked_balance, amount);
			assert_eq!(staker_details.locked_balance, 0);
			assert_eq!(staker_details.staked, true);
			assert_eq!(Tellor::get_total_stake_amount(), amount);
			// expect(await tellor.totalRewardDebt()).to.equal(0) // todo:
			assert_noop!(
				Tellor::report_staking_withdraw_request(
					Origin::Staking.into(),
					reporter,
					(amount + 1).into(),
					address
				),
				Error::InsufficientStake
			);

			assert_ok!(Tellor::report_staking_withdraw_request(
				Origin::Staking.into(),
				reporter,
				token(10).into(),
				address
			));
			let staker_details = Tellor::get_staker_info(reporter).unwrap();
			assert_eq!(staker_details.start_date, Timestamp::get());
			assert_eq!(staker_details.reward_debt, 0);
			assert_eq!(staker_details.staked_balance, token(990));
			assert_eq!(staker_details.locked_balance, token(10));
			assert_eq!(staker_details.staked, true);
			assert_eq!(Tellor::get_total_stake_amount(), token(990));
			// expect(await tellor.totalRewardDebt()).to.equal(0) // todo:

			// Test max/min for amount arg
			assert_noop!(
				Tellor::report_staking_withdraw_request(
					Origin::Staking.into(),
					reporter,
					U256::max_value(),
					address
				),
				Error::InsufficientStake
			);
			assert_ok!(Tellor::report_staking_withdraw_request(
				Origin::Staking.into(),
				reporter,
				U256::zero(),
				address
			));
			let staker_details = Tellor::get_staker_info(reporter).unwrap();
			assert_eq!(staker_details.start_date, Timestamp::get());
			assert_eq!(staker_details.reward_debt, 0);
			assert_eq!(staker_details.staked_balance, token(990));
			assert_eq!(staker_details.locked_balance, token(10));
			assert_eq!(staker_details.staked, true);
			assert_eq!(Tellor::get_total_stake_amount(), token(990));
			// expect(await tellor.totalRewardDebt()).to.equal(0) // todo:

			assert_eq!(Tellor::get_total_stakers(), 1);
			assert_ok!(Tellor::report_staking_withdraw_request(
				Origin::Staking.into(),
				reporter,
				token(990).into(),
				address
			));
			assert_eq!(Tellor::get_total_stakers(), 0);
		});
	});
}

#[test]
fn slash_reporter() {
	let reporter = 1;
	let recipient = 2;
	let amount = token(1_000);
	let address = Address::random();
	let mut ext = new_test_ext();

	// Prerequisites
	ext.execute_with(|| with_block(|| register_parachain(STAKE_AMOUNT)));

	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L195
	ext.execute_with(|| {
		with_block(|| {
			assert_noop!(Tellor::report_slash(RuntimeOrigin::signed(reporter), 0, 0, 0), BadOrigin);
			assert_noop!(
				Tellor::report_slash(Origin::Governance.into(), 0, 0, 0),
				Error::InsufficientStake
			);

			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				amount.into(),
				address
			));

			// Slash when locked balance = 0
			let staker_details = Tellor::get_staker_info(reporter).unwrap();
			assert_eq!(staker_details.staked_balance, amount);
			assert_eq!(staker_details.locked_balance, 0);
			assert_eq!(Tellor::get_total_stake_amount(), amount);
			assert_ok!(Tellor::report_slash(
				Origin::Governance.into(),
				reporter,
				recipient,
				STAKE_AMOUNT
			));
			// todo?
			// blocky0 = await h.getBlock()
			// expect(await tellor.timeOfLastAllocation()).to.equal(blocky0.timestamp)
			// expect(await tellor.accumulatedRewardPerShare()).to.equal(0)
			let staker_details = Tellor::get_staker_info(reporter).unwrap();
			assert_eq!(staker_details.staked_balance, token(900));
			assert_eq!(staker_details.locked_balance, 0);
			assert!(staker_details.staked);
			assert_eq!(Tellor::get_total_stakers(), 1); // Still one staker as reporter has 900 staked & stake amount is 100
			assert_eq!(Tellor::get_total_stake_amount(), token(900));

			// Slash when lockedBalance >= stakeAmount
			assert_ok!(Tellor::report_staking_withdraw_request(
				Origin::Staking.into(),
				reporter,
				token(100).into(),
				address
			));
			let staker_details = Tellor::get_staker_info(reporter).unwrap();
			assert_eq!(staker_details.staked_balance, token(800));
			assert_eq!(staker_details.locked_balance, token(100));
			assert!(staker_details.staked);
			assert_ok!(Tellor::report_slash(
				Origin::Governance.into(),
				reporter,
				recipient,
				STAKE_AMOUNT
			));
			// todo?
			// expect(await tellor.timeOfLastAllocation()).to.equal(blocky1.timestamp)
			// expect(await tellor.accumulatedRewardPerShare()).to.equal(0)
			let staker_details = Tellor::get_staker_info(reporter).unwrap();
			assert_eq!(staker_details.staked_balance, token(800));
			assert_eq!(staker_details.locked_balance, 0);
			assert!(staker_details.staked);
			assert_eq!(Tellor::get_total_stake_amount(), token(800));

			// Slash when 0 < locked balance < stake amount
			assert_ok!(Tellor::report_staking_withdraw_request(
				Origin::Staking.into(),
				reporter,
				token(5).into(),
				address
			));
			let staker_details = Tellor::get_staker_info(reporter).unwrap();
			assert_eq!(staker_details.staked_balance, token(795));
			assert_eq!(staker_details.locked_balance, token(5));
			assert_eq!(Tellor::get_total_stake_amount(), token(795));
			assert_ok!(Tellor::report_slash(
				Origin::Governance.into(),
				reporter,
				recipient,
				STAKE_AMOUNT
			));
			// todo?
			// expect(await tellor.timeOfLastAllocation()).to.equal(blocky2.timestamp)
			// expect(await tellor.accumulatedRewardPerShare()).to.equal(0)
			let staker_details = Tellor::get_staker_info(reporter).unwrap();
			assert_eq!(staker_details.staked_balance, token(700));
			assert_eq!(staker_details.locked_balance, 0);
			assert_eq!(Tellor::get_total_stake_amount(), token(700));

			// Slash when locked balance + staked balance < stake amount
			assert_ok!(Tellor::report_staking_withdraw_request(
				Origin::Staking.into(),
				reporter,
				token(625).into(),
				address
			));
			let staker_details = Tellor::get_staker_info(reporter).unwrap();
			assert_eq!(staker_details.staked_balance, token(75));
			assert_eq!(staker_details.locked_balance, token(625));
			assert_eq!(Tellor::get_total_stake_amount(), token(75));
		});

		with_block_after(WithdrawalPeriod::get(), || {
			assert_ok!(Tellor::report_stake_withdrawal(
				Origin::Staking.into(),
				reporter,
				token(625).into(),
				address
			));
			let staker_details = Tellor::get_staker_info(reporter).unwrap();
			assert_eq!(staker_details.staked_balance, token(75));
			assert_eq!(staker_details.locked_balance, token(0));
			assert_ok!(Tellor::report_slash(
				Origin::Governance.into(),
				reporter,
				recipient,
				STAKE_AMOUNT
			));
			// todo?
			// expect(await tellor.timeOfLastAllocation()).to.equal(blocky.timestamp)
			// expect(await tellor.accumulatedRewardPerShare()).to.equal(0)
			let staker_details = Tellor::get_staker_info(reporter).unwrap();
			assert_eq!(staker_details.staked_balance, 0);
			assert_eq!(staker_details.locked_balance, 0);
			assert_eq!(Tellor::get_total_stakers(), 0);
			assert_eq!(Tellor::get_total_stake_amount(), 0);
		})
	});
}

#[test]
fn submit_value() {
	let query_data: QueryDataOf<Test> = spot_price("dot", "usd").try_into().unwrap();
	let query_id = keccak_256(query_data.as_ref()).into();
	let reporter = 1;
	let another_reporter = 2;
	let address = Address::random();
	let mut ext = new_test_ext();

	// Prerequisites
	ext.execute_with(|| with_block(|| register_parachain(STAKE_AMOUNT)));

	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L277
	ext.execute_with(|| {
		with_block(|| {
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				token(1_200).into(),
				address
			));
			assert_noop!(
				Tellor::submit_value(
					RuntimeOrigin::signed(reporter),
					query_id,
					bounded_vec![],
					0,
					query_data.clone()
				),
				Error::InvalidValue
			);
			assert_noop!(
				Tellor::submit_value(
					RuntimeOrigin::signed(reporter),
					query_id,
					uint_value(4_000),
					1,
					query_data.clone()
				),
				Error::InvalidNonce
			);
			assert_noop!(
				Tellor::submit_value(
					RuntimeOrigin::signed(another_reporter),
					query_id,
					uint_value(4_000),
					0,
					query_data.clone()
				),
				Error::InsufficientStake
			);
			assert_noop!(
				Tellor::submit_value(
					RuntimeOrigin::signed(reporter),
					H256::random(),
					uint_value(4_000),
					0,
					query_data.clone()
				),
				Error::InvalidQueryId
			);
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(4_000),
				0,
				query_data.clone()
			));
			assert_noop!(
				Tellor::submit_value(
					RuntimeOrigin::signed(reporter),
					query_id,
					uint_value(4_000),
					1,
					query_data.clone()
				),
				Error::ReporterTimeLocked
			);
		});

		with_block_after(WithdrawalPeriod::get(), || {
			let timestamp = Timestamp::get();
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(4_001),
				1,
				query_data.clone()
			));
			assert_eq!(Tellor::get_timestamp_index_by_timestamp(query_id, timestamp).unwrap(), 1);
			assert_eq!(
				Tellor::get_timestamp_by_query_id_and_index(query_id, 1).unwrap(),
				timestamp
			);
			assert_eq!(
				Tellor::get_block_number_by_timestamp(query_id, timestamp).unwrap(),
				System::block_number()
			);
			assert_eq!(Tellor::retrieve_data(query_id, timestamp).unwrap(), uint_value(4_001));
			assert_eq!(Tellor::get_reporter_by_timestamp(query_id, timestamp).unwrap(), reporter);
			assert_eq!(Tellor::time_of_last_new_value().unwrap(), timestamp);
			assert_eq!(Tellor::get_reports_submitted_by_address(&reporter), 2);
			assert_eq!(
				Tellor::get_reports_submitted_by_address_and_query_id(reporter, query_id),
				2
			);
		});

		// Test submit multiple identical values w/ min nonce
		with_block(|| {
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				another_reporter,
				token(120).into(),
				address
			));
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(another_reporter),
				query_id,
				uint_value(4_001),
				0,
				query_data.clone()
			));
		});
		with_block_after(ReportingLock::get(), || {
			let timestamp = Timestamp::get();
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(4_001),
				0,
				query_data.clone()
			));

			assert_eq!(Tellor::get_timestamp_index_by_timestamp(query_id, timestamp).unwrap(), 3);
			assert_eq!(
				Tellor::get_timestamp_by_query_id_and_index(query_id, 3).unwrap(),
				timestamp
			);
			assert_eq!(
				Tellor::get_block_number_by_timestamp(query_id, timestamp).unwrap(),
				System::block_number()
			);
			assert_eq!(Tellor::retrieve_data(query_id, timestamp).unwrap(), uint_value(4001));
			assert_eq!(Tellor::get_reporter_by_timestamp(query_id, timestamp).unwrap(), reporter);
			assert_eq!(Tellor::time_of_last_new_value().unwrap(), timestamp);
			assert_eq!(Tellor::get_reports_submitted_by_address(&reporter), 3);
			assert_eq!(
				Tellor::get_reports_submitted_by_address_and_query_id(reporter, query_id),
				3
			);

			// Test max val for nonce
			assert_noop!(
				Tellor::submit_value(
					RuntimeOrigin::signed(reporter),
					query_id,
					uint_value(4_001),
					Nonce::MAX,
					query_data
				),
				Error::InvalidNonce
			);
		})
	});
}

#[test]
fn withdraw_stake() {
	let reporter = 1;
	let address = Address::random();
	let mut ext = new_test_ext();

	// Prerequisites
	ext.execute_with(|| with_block(|| register_parachain(STAKE_AMOUNT)));

	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L323
	ext.execute_with(|| {
		with_block(|| {
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				STAKE_AMOUNT.into(),
				address
			));
			assert_eq!(Tellor::get_total_stakers(), 1);
			assert_noop!(
				Tellor::report_stake_withdrawal(
					Origin::Staking.into(),
					reporter,
					STAKE_AMOUNT.into(),
					address
				),
				Error::NoWithdrawalRequested
			);
			assert_ok!(Tellor::report_staking_withdraw_request(
				Origin::Staking.into(),
				reporter,
				token(10).into(),
				address
			));
			assert_noop!(
				Tellor::report_stake_withdrawal(
					Origin::Staking.into(),
					reporter,
					STAKE_AMOUNT.into(),
					address
				),
				Error::WithdrawalPeriodPending
			);
			let staker_details = Tellor::get_staker_info(reporter).unwrap();
			assert_eq!(staker_details.staked_balance, token(90));
			assert_eq!(staker_details.locked_balance, token(10));
		});

		with_block_after(WithdrawalPeriod::get(), || {
			assert_ok!(Tellor::report_stake_withdrawal(
				Origin::Staking.into(),
				reporter,
				token(10).into(),
				address
			));
			let staker_details = Tellor::get_staker_info(reporter).unwrap();
			assert_eq!(staker_details.staked_balance, token(90));
			assert_eq!(staker_details.locked_balance, 0);
			assert_noop!(
				Tellor::report_stake_withdrawal(
					Origin::Staking.into(),
					reporter,
					token(10).into(),
					address
				),
				Error::NoWithdrawalRequested
			);
		});
	});
}

#[test]
fn get_block_number_by_timestamp() {
	let reporter = 1;
	let address = Address::random();
	let query_data: QueryDataOf<Test> = spot_price("dot", "usd").try_into().unwrap();
	let query_id = keccak_256(query_data.as_ref()).into();
	let mut ext = new_test_ext();

	// Prerequisites
	ext.execute_with(|| with_block(|| register_parachain(STAKE_AMOUNT)));

	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L345
	ext.execute_with(|| {
		with_block(|| {
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				STAKE_AMOUNT.into(),
				address
			));
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(4000),
				0,
				query_data.clone(),
			));
			assert_eq!(
				Tellor::get_block_number_by_timestamp(query_id, Timestamp::get()).unwrap(),
				System::block_number()
			)
		});
	});
}

#[test]
fn get_current_value() {
	let reporter = 1;
	let query_data: QueryDataOf<Test> = spot_price("dot", "usd").try_into().unwrap();
	let query_id = keccak_256(query_data.as_ref()).into();
	let mut ext = new_test_ext();

	// Prerequisites
	ext.execute_with(|| with_block(|| register_parachain(STAKE_AMOUNT)));

	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L352
	ext.execute_with(|| {
		with_block(|| {
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				STAKE_AMOUNT.into(),
				Address::random()
			));
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(4000),
				0,
				query_data.clone(),
			));
			assert_eq!(Tellor::get_current_value(query_id).unwrap(), uint_value(4000))
		})
	});
}

#[test]
fn get_new_value_count_by_query_id() {
	let reporter = 1;
	let query_data: QueryDataOf<Test> = spot_price("dot", "usd").try_into().unwrap();
	let query_id = keccak_256(query_data.as_ref()).into();
	let mut ext = new_test_ext();

	// Prerequisites
	ext.execute_with(|| with_block(|| register_parachain(STAKE_AMOUNT)));

	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L363
	ext.execute_with(|| {
		with_block(|| {
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				STAKE_AMOUNT.into(),
				Address::random()
			));
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(4000),
				0,
				query_data.clone(),
			));
		});

		with_block_after(ReportingLock::get(), || {
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(4000),
				0,
				query_data.clone(),
			));
			assert_eq!(Tellor::get_new_value_count_by_query_id(query_id), 2)
		});
	});
}

#[test]
fn get_report_details() {
	let reporter = 1;
	let query_data: QueryDataOf<Test> = spot_price("dot", "usd").try_into().unwrap();
	let query_id = keccak_256(query_data.as_ref()).into();
	let mut ext = new_test_ext();

	// Prerequisites
	ext.execute_with(|| with_block(|| register_parachain(STAKE_AMOUNT)));

	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L372
	ext.execute_with(|| {
		let (timestamp_1, _) = with_block(|| {
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				STAKE_AMOUNT.into(),
				Address::random()
			));
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(4000),
				0,
				query_data.clone(),
			));
		});

		let (timestamp_2, _) = with_block_after(ReportingLock::get(), || {
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(4001),
				0,
				query_data.clone(),
			));
		});

		let (timestamp_3, _) = with_block_after(ReportingLock::get(), || {
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(4002),
				0,
				query_data.clone(),
			));
			assert_ok!(Tellor::remove_value(query_id, Timestamp::get()));
		});

		assert_eq!(Tellor::get_report_details(query_id, timestamp_1).unwrap(), (reporter, false));
		assert_eq!(Tellor::get_report_details(query_id, timestamp_2).unwrap(), (reporter, false));
		assert_eq!(Tellor::get_report_details(query_id, timestamp_3).unwrap(), (reporter, true));
		assert_eq!(Tellor::get_report_details(H256::zero(), timestamp_1), None);
	});
}

#[test]
fn get_reporting_lock() {
	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L398
	let reporting_lock: TimestampOf<Test> = ReportingLock::get();
	assert_eq!(Tellor::get_reporting_lock(), reporting_lock)
}

#[test]
fn get_reporter_by_timestamp() {
	let reporter = 1;
	let query_data: QueryDataOf<Test> = spot_price("dot", "usd").try_into().unwrap();
	let query_id = keccak_256(query_data.as_ref()).into();
	let mut ext = new_test_ext();

	// Prerequisites
	ext.execute_with(|| with_block(|| register_parachain(STAKE_AMOUNT)));

	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L402
	ext.execute_with(|| {
		with_block(|| {
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				STAKE_AMOUNT.into(),
				Address::random()
			));
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(4000),
				0,
				query_data.clone(),
			));
			assert_eq!(
				Tellor::get_reporter_by_timestamp(query_id, Timestamp::get()).unwrap(),
				reporter
			)
		});
	});
}

#[test]
fn get_reporter_last_timestamp() {
	let reporter = 1;
	let query_data: QueryDataOf<Test> = spot_price("dot", "usd").try_into().unwrap();
	let query_id = keccak_256(query_data.as_ref()).into();
	let mut ext = new_test_ext();

	// Prerequisites
	ext.execute_with(|| with_block(|| register_parachain(STAKE_AMOUNT)));

	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L409
	ext.execute_with(|| {
		with_block(|| {
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				STAKE_AMOUNT.into(),
				Address::random()
			));
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(4000),
				0,
				query_data.clone(),
			));
			assert_eq!(Tellor::get_reporter_last_timestamp(reporter).unwrap(), Timestamp::get())
		});
	});
}

#[test]
fn get_reports_submitted_by_address() {
	let reporter = 1;
	let query_data: QueryDataOf<Test> = spot_price("dot", "usd").try_into().unwrap();
	let query_id = keccak_256(query_data.as_ref()).into();
	let mut ext = new_test_ext();

	// Prerequisites
	ext.execute_with(|| with_block(|| register_parachain(STAKE_AMOUNT)));

	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L419
	ext.execute_with(|| {
		with_block(|| {
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				STAKE_AMOUNT.into(),
				Address::random()
			));
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(4000),
				0,
				query_data.clone(),
			));
		});

		with_block_after(ReportingLock::get(), || {
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(4000),
				0,
				query_data.clone(),
			));
			assert_eq!(Tellor::get_reports_submitted_by_address(&reporter), 2)
		})
	});
}

#[test]
fn get_reports_submitted_by_address_and_query_id() {
	let reporter = 1;
	let query_data: QueryDataOf<Test> = spot_price("dot", "usd").try_into().unwrap();
	let query_id = keccak_256(query_data.as_ref()).into();
	let mut ext = new_test_ext();

	// Prerequisites
	ext.execute_with(|| with_block(|| register_parachain(STAKE_AMOUNT)));

	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L429
	ext.execute_with(|| {
		with_block(|| {
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				STAKE_AMOUNT.into(),
				Address::random()
			));
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(4000),
				0,
				query_data.clone(),
			));
		});

		with_block_after(ReportingLock::get(), || {
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(4000),
				0,
				query_data.clone(),
			));
			assert_eq!(Tellor::get_reports_submitted_by_address_and_query_id(reporter, query_id), 2)
		})
	});
}

#[test]
fn get_stake_amount() {
	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L439
	new_test_ext().execute_with(|| {
		with_block(|| {
			assert_eq!(Tellor::get_stake_amount(), 0);
			register_parachain(STAKE_AMOUNT);
			assert_eq!(Tellor::get_stake_amount(), STAKE_AMOUNT);
		})
	});
}

#[test]
fn get_staker_info() {
	let reporter = 1;
	let address = Address::random();
	let query_data: QueryDataOf<Test> = spot_price("dot", "usd").try_into().unwrap();
	let query_id = keccak_256(query_data.as_ref()).into();
	let mut ext = new_test_ext();

	// Prerequisites
	ext.execute_with(|| with_block(|| register_parachain(STAKE_AMOUNT)));

	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L443
	ext.execute_with(|| {
		with_block(|| {
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				token(1_000).into(),
				address
			));
			assert_ok!(Tellor::report_staking_withdraw_request(
				Origin::Staking.into(),
				reporter,
				token(100).into(),
				address
			));
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(4000),
				0,
				query_data.clone(),
			));
			let staker_details = Tellor::get_staker_info(reporter).unwrap();
			assert_eq!(staker_details.address, address);
			assert_eq!(staker_details.start_date, Timestamp::get());
			assert_eq!(staker_details.staked_balance, token(900));
			assert_eq!(staker_details.locked_balance, token(100));
			assert_eq!(staker_details.reward_debt, 0);
			assert_eq!(staker_details.reporter_last_timestamp, Timestamp::get());
			assert_eq!(staker_details.reports_submitted, 1);
			assert_eq!(staker_details.start_vote_count, 0);
			assert_eq!(staker_details.start_vote_tally, 0);
			assert_eq!(staker_details.staked, true);
			let reports_submitted_by_query_id: BoundedReportsSubmittedByQueryId =
				bounded_btree_map!(query_id => 1u128);
			assert_eq!(staker_details.reports_submitted_by_query_id, reports_submitted_by_query_id);
		});
	});
}

#[test]
fn get_time_of_last_new_value() {
	let reporter = 1;
	let query_data: QueryDataOf<Test> = spot_price("dot", "usd").try_into().unwrap();
	let query_id = keccak_256(query_data.as_ref()).into();
	let mut ext = new_test_ext();

	// Prerequisites
	ext.execute_with(|| with_block(|| register_parachain(STAKE_AMOUNT)));

	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L461
	ext.execute_with(|| {
		with_block(|| {
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				STAKE_AMOUNT.into(),
				Address::random()
			));
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(4000),
				0,
				query_data.clone(),
			));
			assert_eq!(Tellor::get_time_of_last_new_value().unwrap(), Timestamp::get())
		});
	});
}

#[test]
fn get_timestamp_by_query_and_index() {
	let reporter = 1;
	let query_data: QueryDataOf<Test> = spot_price("dot", "usd").try_into().unwrap();
	let query_id = keccak_256(query_data.as_ref()).into();
	let mut ext = new_test_ext();

	// Prerequisites
	ext.execute_with(|| with_block(|| register_parachain(STAKE_AMOUNT)));

	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L471
	ext.execute_with(|| {
		with_block(|| {
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				STAKE_AMOUNT.into(),
				Address::random()
			));
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(4000),
				0,
				query_data.clone(),
			));
		});

		with_block_after(ReportingLock::get(), || {
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(4000),
				0,
				query_data.clone(),
			));
			assert_eq!(
				Tellor::get_timestamp_by_query_id_and_index(query_id, 1).unwrap(),
				Timestamp::get()
			)
		})
	});
}

#[test]
fn get_timestamp_index_by_timestamp() {
	let reporter = 1;
	let query_data: QueryDataOf<Test> = spot_price("dot", "usd").try_into().unwrap();
	let query_id = keccak_256(query_data.as_ref()).into();
	let mut ext = new_test_ext();

	// Prerequisites
	ext.execute_with(|| with_block(|| register_parachain(STAKE_AMOUNT)));

	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L481
	ext.execute_with(|| {
		with_block(|| {
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				STAKE_AMOUNT.into(),
				Address::random()
			));
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(4000),
				0,
				query_data.clone(),
			));
		});

		with_block_after(ReportingLock::get(), || {
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(4000),
				0,
				query_data.clone(),
			));
			assert_eq!(
				Tellor::get_timestamp_index_by_timestamp(query_id, Timestamp::get()).unwrap(),
				1
			)
		})
	});
}

#[test]
fn get_total_stake_amount() {
	let reporter = 1;
	let address = Address::random();
	let mut ext = new_test_ext();

	// Prerequisites
	ext.execute_with(|| with_block(|| register_parachain(STAKE_AMOUNT)));

	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L491
	ext.execute_with(|| {
		with_block(|| {
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				STAKE_AMOUNT.into(),
				address
			));
			assert_ok!(Tellor::report_staking_withdraw_request(
				Origin::Staking.into(),
				reporter,
				token(10).into(),
				address
			));
			assert_eq!(Tellor::get_total_stake_amount(), token(90))
		});
	});
}

#[test]
fn get_total_stakers() {
	let reporter = 1;
	let address = Address::random();
	let mut ext = new_test_ext();

	// Prerequisites
	ext.execute_with(|| with_block(|| register_parachain(STAKE_AMOUNT)));

	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L502
	ext.execute_with(|| {
		with_block(|| {
			// Only count unique stakers
			assert_eq!(Tellor::get_total_stakers(), 0);
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				STAKE_AMOUNT.into(),
				address
			));
			assert_eq!(Tellor::get_total_stakers(), 1);
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				STAKE_AMOUNT.into(),
				address
			));
			assert_eq!(Tellor::get_total_stakers(), 1);

			// Unstake, restake
			assert_ok!(Tellor::report_staking_withdraw_request(
				Origin::Staking.into(),
				reporter,
				token(200).into(),
				address
			));
			assert_eq!(Tellor::get_total_stakers(), 0);
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				STAKE_AMOUNT.into(),
				address
			));
			assert_eq!(Tellor::get_total_stakers(), 1);
		});
	});
}

#[test]
fn retrieve_data() {
	let reporter = 1;
	let query_data: QueryDataOf<Test> = spot_price("dot", "usd").try_into().unwrap();
	let query_id = keccak_256(query_data.as_ref()).into();
	let mut ext = new_test_ext();

	// Prerequisites
	ext.execute_with(|| with_block(|| register_parachain(STAKE_AMOUNT)));

	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L519
	ext.execute_with(|| {
		with_block(|| {
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				STAKE_AMOUNT.into(),
				Address::random()
			));
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(4000),
				0,
				query_data.clone(),
			));
		});

		with_block_after(ReportingLock::get(), || {
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(4001),
				0,
				query_data.clone(),
			));
			assert_eq!(
				Tellor::retrieve_data(query_id, Timestamp::get()).unwrap(),
				uint_value(4001)
			);

			// Test max/min values for _timestamp arg
			assert_eq!(Tellor::retrieve_data(query_id, 0), None);
			assert_eq!(Tellor::retrieve_data(query_id, <TimestampOf<Test>>::MAX), None);
		})
	});
}

#[test]
#[ignore]
fn get_total_time_based_rewards_balance() {
	// https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L533
	todo!()
}

#[test]
#[ignore]
fn add_staking_rewards() {
	// https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L539
	todo!()
}

#[test]
#[ignore]
fn get_pending_reward_by_staker() {
	// https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L561
	todo!()
}

#[test]
fn get_index_for_data_before() {
	let reporter = 1;
	let query_data: QueryDataOf<Test> = spot_price("dot", "usd").try_into().unwrap();
	let query_id = keccak_256(query_data.as_ref()).into();
	let mut ext = new_test_ext();

	// Prerequisites
	ext.execute_with(|| with_block(|| register_parachain(STAKE_AMOUNT)));

	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L519
	ext.execute_with(|| {
		let (timestamp_0, _) = with_block(|| {
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				token(1_000).into(),
				Address::random()
			));
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(100),
				0,
				query_data.clone(),
			));
		});
		let (timestamp_1, _) = with_block_after(ReportingLock::get(), || {
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(100),
				1,
				query_data.clone(),
			));
		});
		let (timestamp_2, _) = with_block_after(ReportingLock::get(), || {
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(100),
				2,
				query_data.clone(),
			));
		});

		assert_eq!(Tellor::get_index_for_data_before(query_id, timestamp_2), Some(1));

		// advance time and test
		for year in 1..2 {
			with_block_after(year * 365 * DAY_IN_MILLISECONDS, || {
				assert_eq!(Tellor::get_index_for_data_before(query_id, timestamp_2), Some(1));
			});
		}

		for i in 0..50 {
			with_block_after(ReportingLock::get(), || {
				assert_ok!(Tellor::submit_value(
					RuntimeOrigin::signed(reporter),
					query_id,
					uint_value(100 + i),
					0,
					query_data.clone(),
				));
			});
		}
		let timestamp_52 = Timestamp::get();

		// test last value disputed
		with_block(|| {
			assert_ok!(Tellor::remove_value(query_id, timestamp_52));
			assert_eq!(Tellor::get_index_for_data_before(query_id, timestamp_52), Some(51));
			assert_eq!(Tellor::get_index_for_data_before(query_id, timestamp_2), Some(1));
			assert_eq!(Tellor::get_index_for_data_before(query_id, timestamp_2 + 1), Some(2));

			// remove value at index 2
			assert_ok!(Tellor::remove_value(query_id, timestamp_2));
			assert_eq!(Tellor::get_index_for_data_before(query_id, timestamp_2), Some(1));
			assert_eq!(Tellor::get_index_for_data_before(query_id, timestamp_2 + 1), Some(1));
			assert_eq!(Tellor::get_index_for_data_before(query_id, timestamp_1 + 1), Some(1));

			assert_ok!(Tellor::remove_value(query_id, timestamp_1));
			assert_eq!(Tellor::get_index_for_data_before(query_id, timestamp_2 - 1), Some(0));

			assert_ok!(Tellor::remove_value(query_id, timestamp_0));
			assert_eq!(Tellor::get_index_for_data_before(query_id, timestamp_2 - 1), None);
		});

		let query_data: QueryDataOf<Test> = spot_price("ksm", "usd").try_into().unwrap();
		let query_id = keccak_256(query_data.as_ref()).into();

		let (timestamp_0, _) = with_block_after(ReportingLock::get(), || {
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(100),
				0,
				query_data.clone(),
			));
		});
		let (timestamp_1, _) = with_block_after(ReportingLock::get(), || {
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(100),
				0,
				query_data.clone(),
			));

			assert_ok!(Tellor::remove_value(query_id, timestamp_0));
			assert_ok!(Tellor::remove_value(query_id, Timestamp::get()));
		});

		assert_eq!(Tellor::get_index_for_data_before(query_id, timestamp_1 + 1), None);
		assert_eq!(Tellor::get_index_for_data_before(query_id, timestamp_0 + 1), None);

		let (timestamp_2, _) = with_block_after(ReportingLock::get(), || {
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(100),
				0,
				query_data.clone(),
			));
		});

		with_block_after(ReportingLock::get(), || {
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(100),
				0,
				query_data.clone(),
			));

			assert_ok!(Tellor::remove_value(query_id, timestamp_2));
			assert_eq!(Tellor::get_index_for_data_before(query_id, timestamp_2 + 1), None);
		});
	});
}

#[test]
fn get_data_before() {
	let reporter = 1;
	let query_data: QueryDataOf<Test> = spot_price("dot", "usd").try_into().unwrap();
	let query_id = keccak_256(query_data.as_ref()).into();
	let mut ext = new_test_ext();

	// Prerequisites
	ext.execute_with(|| with_block(|| register_parachain(STAKE_AMOUNT)));

	// Based on https://github.com/tellor-io/tellorFlex/blob/3b3820f2111ec2813cb51455ef68cf0955c51674/test/functionTests-TellorFlex.js#L697
	ext.execute_with(|| {
		let (timestamp_1, _) = with_block(|| {
			assert_ok!(Tellor::report_stake_deposited(
				Origin::Staking.into(),
				reporter,
				token(1_000).into(),
				Address::random()
			));
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(150),
				0,
				query_data.clone(),
			));
		});
		let (timestamp_2, _) = with_block_after(ReportingLock::get(), || {
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(160),
				1,
				query_data.clone(),
			));
		});
		let (timestamp_3, _) = with_block_after(ReportingLock::get(), || {
			assert_ok!(Tellor::submit_value(
				RuntimeOrigin::signed(reporter),
				query_id,
				uint_value(170),
				2,
				query_data.clone(),
			));
		});

		assert_eq!(
			Tellor::get_data_before(query_id, timestamp_3 + 1),
			Some((uint_value(170), timestamp_3))
		);
		assert_eq!(
			Tellor::get_data_before(query_id, timestamp_2),
			Some((uint_value(150), timestamp_1))
		);

		// advance time one year and test
		for year in 1..2 {
			with_block_after(year * 365 * DAY_IN_MILLISECONDS, || {
				assert_eq!(
					Tellor::get_data_before(query_id, timestamp_3 + 1),
					Some((uint_value(170), timestamp_3))
				);
				assert_eq!(
					Tellor::get_data_before(query_id, timestamp_2),
					Some((uint_value(150), timestamp_1))
				);
			});
		}

		// submit 50 values and test
		for i in 0..50 {
			with_block_after(ReportingLock::get(), || {
				assert_ok!(Tellor::submit_value(
					RuntimeOrigin::signed(reporter),
					query_id,
					uint_value(100 + i),
					0,
					query_data.clone(),
				));
			});
		}

		assert_eq!(
			Tellor::get_data_before(query_id, timestamp_3 + 1),
			Some((uint_value(170), timestamp_3))
		);
		assert_eq!(
			Tellor::get_data_before(query_id, timestamp_2),
			Some((uint_value(150), timestamp_1))
		);
	});
}

#[test]
#[ignore]
fn update_stake_amount() {
	todo!()
}

#[test]
#[ignore]
fn update_rewards() {
	todo!()
}

#[test]
#[ignore]
fn update_stake_and_pay_rewards() {
	todo!()
}

#[test]
#[ignore]
fn get_real_staking_rewards_balance() {
	todo!()
}
