use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

const TOKEN_0_ID: u32 = 1;
const TOKEN_1_ID: u32 = 2;

const ALICE: u64 = 1;
const BOB: u64 = 2;

const MIL: u128 = (10 as u128).pow(6);

#[test]
fn init_should_work_1() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc1155::init(
			Origin::signed(ALICE),
			vec![TOKEN_0_ID, TOKEN_1_ID],
			vec![1000, 1000]
		));
		assert_eq!(Erc1155::get_balance(ALICE, TOKEN_0_ID), 1000 * MIL);
		assert_eq!(Erc1155::get_total_supply(TOKEN_0_ID), 1000 * MIL);
		assert_eq!(Erc1155::get_balance(ALICE, TOKEN_1_ID), 1000 * MIL);
		assert_eq!(Erc1155::get_total_supply(TOKEN_1_ID), 1000 * MIL);
	});
}

#[test]
fn init_should_work_2() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc1155::init(Origin::signed(ALICE), vec![TOKEN_0_ID], vec![1000]));
		assert_ok!(Erc1155::init(Origin::signed(BOB), vec![TOKEN_1_ID], vec![1000]));
		assert_eq!(Erc1155::get_balance(ALICE, TOKEN_0_ID), 1000 * MIL);
		assert_eq!(Erc1155::get_total_supply(TOKEN_0_ID), 1000 * MIL);
		assert_eq!(Erc1155::get_balance(BOB, TOKEN_1_ID), 1000 * MIL);
		assert_eq!(Erc1155::get_total_supply(TOKEN_1_ID), 1000 * MIL);
	});
}

#[test]
fn init_should_fail_1() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc1155::init(
			Origin::signed(ALICE),
			vec![TOKEN_0_ID, TOKEN_1_ID],
			vec![1000, 1000]
		));
		assert_noop!(
			Erc1155::init(Origin::signed(ALICE), vec![TOKEN_0_ID], vec![1000]),
			Error::<Test>::AlreadyInitialized
		);
	});
}

#[test]
fn init_should_fail_2() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Erc1155::init(Origin::signed(ALICE), vec![TOKEN_0_ID, TOKEN_1_ID], vec![1000, 0]),
			Error::<Test>::ZeroSupplyProvided
		);
	});
}

#[test]
fn init_should_fail_3() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Erc1155::init(Origin::signed(ALICE), vec![TOKEN_0_ID, TOKEN_1_ID], vec![1000]),
			Error::<Test>::InsufficientDataProvided
		);
		assert_noop!(
			Erc1155::init(Origin::signed(ALICE), vec![TOKEN_1_ID], vec![1000, 100]),
			Error::<Test>::InsufficientDataProvided
		);
	});
}

#[test]
fn init_should_fail_4() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Erc1155::init(Origin::signed(ALICE), vec![TOKEN_0_ID], vec![u128::MAX]),
			Error::<Test>::Overflow
		);
	});
}

#[test]
fn set_approval_for_all_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc1155::init(Origin::signed(ALICE), vec![TOKEN_0_ID], vec![1000]));
		assert_ok!(Erc1155::set_approval_for_all(Origin::signed(ALICE), BOB, true));
		assert_eq!(Erc1155::get_approval(ALICE, BOB), true);
		assert_ok!(Erc1155::set_approval_for_all(Origin::signed(ALICE), BOB, false));
		assert_eq!(Erc1155::get_approval(ALICE, BOB), false);
	});
}

#[test]
fn transfer_from_single_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc1155::init(Origin::signed(ALICE), vec![TOKEN_0_ID], vec![1000]));
		assert_ok!(Erc1155::set_approval_for_all(Origin::signed(ALICE), BOB, true));
		assert_ok!(Erc1155::transfer_from_single(
			Origin::signed(ALICE),
			ALICE,
			BOB,
			TOKEN_0_ID,
			100 * MIL
		));
		assert_eq!(Erc1155::get_balance(ALICE, TOKEN_0_ID), 900 * MIL);
		assert_eq!(Erc1155::get_balance(BOB, TOKEN_0_ID), 100 * MIL);
	});
}

#[test]
fn transfer_from_single_should_fail_1() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc1155::init(Origin::signed(ALICE), vec![TOKEN_0_ID], vec![1000]));
		assert_noop!(
			Erc1155::transfer_from_single(Origin::signed(ALICE), ALICE, BOB, TOKEN_0_ID, 100 * MIL),
			Error::<Test>::TransferNotApproved
		);
	});
}

#[test]
fn transfer_from_single_should_fail_2() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc1155::init(Origin::signed(ALICE), vec![TOKEN_0_ID], vec![1000]));
		assert_noop!(
			Erc1155::transfer_from_single(Origin::signed(ALICE), ALICE, BOB, TOKEN_0_ID, 0 * MIL),
			Error::<Test>::ZeroAmountTransfer
		);
	});
}

#[test]
fn transfer_from_single_should_fail_3() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc1155::init(Origin::signed(ALICE), vec![TOKEN_0_ID], vec![1000]));
		assert_noop!(
			Erc1155::transfer_from_single(Origin::signed(ALICE), ALICE, ALICE, TOKEN_0_ID, 1 * MIL),
			Error::<Test>::SelfTransfer
		);
	});
}

#[test]
fn transfer_from_single_should_fail_4() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc1155::init(Origin::signed(ALICE), vec![TOKEN_0_ID], vec![1000]));
		assert_ok!(Erc1155::set_approval_for_all(Origin::signed(ALICE), 0, true));
		assert_noop!(
			Erc1155::transfer_from_single(Origin::signed(ALICE), ALICE, 0, TOKEN_0_ID, 1 * MIL),
			Error::<Test>::ZeroAdressTransfer
		);
	});
}

#[test]
fn transfer_from_single_should_fail_5() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc1155::init(Origin::signed(ALICE), vec![TOKEN_0_ID], vec![1000]));
		assert_ok!(Erc1155::set_approval_for_all(Origin::signed(ALICE), BOB, true));
		assert_noop!(
			Erc1155::transfer_from_single(
				Origin::signed(ALICE),
				ALICE,
				BOB,
				TOKEN_0_ID,
				1001 * MIL
			),
			Error::<Test>::InsufficientFunds
		);
	});
}

#[test]
fn transfer_from_single_should_fail_6() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc1155::init(Origin::signed(ALICE), vec![TOKEN_0_ID], vec![1000]));
		assert_ok!(Erc1155::set_approval_for_all(Origin::signed(ALICE), BOB, true));
		assert_noop!(
			Erc1155::transfer_from_single(Origin::signed(ALICE), ALICE, BOB, TOKEN_1_ID, 50 * MIL),
			Error::<Test>::Uninitilized
		);
	});
}

#[test]
fn transfer_from_batch_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc1155::init(
			Origin::signed(ALICE),
			vec![TOKEN_0_ID, TOKEN_1_ID],
			vec![1000, 1000]
		));
		assert_ok!(Erc1155::set_approval_for_all(Origin::signed(ALICE), BOB, true));
		assert_ok!(Erc1155::transfer_from_batch(
			Origin::signed(ALICE),
			ALICE,
			BOB,
			vec![TOKEN_0_ID, TOKEN_1_ID],
			vec![900 * MIL, 50 * MIL]
		));
		assert_eq!(Erc1155::get_balance(ALICE, TOKEN_0_ID), 100 * MIL);
		assert_eq!(Erc1155::get_balance(ALICE, TOKEN_1_ID), 950 * MIL);
		assert_eq!(Erc1155::get_balance(BOB, TOKEN_0_ID), 900 * MIL);
		assert_eq!(Erc1155::get_balance(BOB, TOKEN_1_ID), 50 * MIL);
	});
}

#[test]
fn transfer_from_batch_should_fail_1() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc1155::init(
			Origin::signed(ALICE),
			vec![TOKEN_0_ID, TOKEN_1_ID],
			vec![1000, 1000]
		));
		assert_noop!(
			Erc1155::transfer_from_batch(
				Origin::signed(ALICE),
				ALICE,
				BOB,
				vec![TOKEN_0_ID, TOKEN_1_ID],
				vec![900 * MIL, 50 * MIL]
			),
			Error::<Test>::TransferNotApproved
		);
	});
}

#[test]
fn transfer_from_batch_should_fail_2() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc1155::init(
			Origin::signed(ALICE),
			vec![TOKEN_0_ID, TOKEN_1_ID],
			vec![1000, 1000]
		));
		assert_ok!(Erc1155::set_approval_for_all(Origin::signed(ALICE), BOB, true));
		assert_noop!(
			Erc1155::transfer_from_batch(
				Origin::signed(ALICE),
				ALICE,
				BOB,
				vec![TOKEN_0_ID, TOKEN_1_ID],
				vec![900 * MIL, 0]
			),
			Error::<Test>::ZeroAmountTransfer
		);
	});
}

#[test]
fn transfer_from_batch_should_fail_3() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc1155::init(
			Origin::signed(ALICE),
			vec![TOKEN_0_ID, TOKEN_1_ID],
			vec![1000, 1000]
		));
		assert_ok!(Erc1155::set_approval_for_all(Origin::signed(ALICE), BOB, true));
		assert_noop!(
			Erc1155::transfer_from_batch(
				Origin::signed(ALICE),
				ALICE,
				ALICE,
				vec![TOKEN_0_ID, TOKEN_1_ID],
				vec![900 * MIL, 50 * MIL]
			),
			Error::<Test>::SelfTransfer
		);
	});
}

#[test]
fn transfer_from_batch_should_fail_4() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc1155::init(
			Origin::signed(ALICE),
			vec![TOKEN_0_ID, TOKEN_1_ID],
			vec![1000, 1000]
		));
		assert_ok!(Erc1155::set_approval_for_all(Origin::signed(ALICE), 0, true));
		assert_noop!(
			Erc1155::transfer_from_batch(
				Origin::signed(ALICE),
				ALICE,
				0,
				vec![TOKEN_0_ID, TOKEN_1_ID],
				vec![900 * MIL, 50 * MIL]
			),
			Error::<Test>::ZeroAdressTransfer
		);
	});
}

#[test]
fn transfer_from_batch_should_fail_5() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc1155::init(
			Origin::signed(ALICE),
			vec![TOKEN_0_ID, TOKEN_1_ID],
			vec![1000, 1000]
		));
		assert_ok!(Erc1155::set_approval_for_all(Origin::signed(ALICE), BOB, true));
		assert_noop!(
			Erc1155::transfer_from_batch(
				Origin::signed(ALICE),
				ALICE,
				BOB,
				vec![TOKEN_0_ID, TOKEN_1_ID],
				vec![1001 * MIL, 50 * MIL]
			),
			Error::<Test>::InsufficientFunds
		);
	});
}

#[test]
fn transfer_from_batch_should_fail_6() {
	new_test_ext().execute_with(|| {
		assert_ok!(Erc1155::init(Origin::signed(ALICE), vec![TOKEN_0_ID], vec![1000]));
		assert_ok!(Erc1155::set_approval_for_all(Origin::signed(ALICE), BOB, true));
		assert_noop!(
			Erc1155::transfer_from_batch(
				Origin::signed(ALICE),
				ALICE,
				BOB,
				vec![TOKEN_0_ID, TOKEN_1_ID],
				vec![100 * MIL, 50 * MIL]
			),
			Error::<Test>::Uninitilized
		);
	});
}
