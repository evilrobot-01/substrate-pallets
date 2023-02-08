use crate::{mock::*, Event};
use frame_support::assert_ok;
use std::num::ParseIntError;

#[test]
fn submits_value() {
    new_test_ext().execute_with(|| {
        // Go past genesis block so events get deposited
        System::set_block_number(1);
        // Submit value
        let payload = decode_hex("4254430000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003d1e382100045544800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002e90edd00001812f2590c000000020000002c1296a449f5d353c8b04eb389f33a583ee79449cca6e366900042f19f2521e722a410929223231905839c00865af68738f1a202478d87dc33675ea5824f343901b4254430000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003d1e382100045544800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002e90edd00001812f2590c000000020000002dbbf8a0e6b1c9a56a4a0ef7089ef2a3f74fbd21fbd5c7c8192b70084004b4f6d37427507c4fff835f74fd4d000b6830ed296e207f49831b96f90a4f4e60820ee1c0002312e312e3223746573742d646174612d66656564000014000002ed57011e0000").unwrap().try_into().unwrap();
        assert_ok!(RedStoneExample::submit_value(RuntimeOrigin::signed(1), payload));
        // Read pallet storage and assert an expected result.
        let expected = 42_000 * 100_000_000;
        assert_eq!(RedStoneExample::value(), Some(expected));
        // Assert that the correct event was deposited
        System::assert_last_event(Event::ValueStored { value: expected, who: 1 }.into());
    });
}

// From https://github.com/redstone-finance/redstone-rust-sdk/blob/main/tests/integration_test.rs
fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}
