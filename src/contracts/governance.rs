use super::*;

pub(crate) fn begin_parachain_dispute(
	query_id: &[u8],
	timestamp: impl Into<U256>,
	value: &[u8],
	disputed_reporter: Address,
	dispute_initiator: Address,
	dispute_fee: impl Into<U256>,
	slash_amount: impl Into<U256>,
) -> Vec<u8> {
	const FUNCTION: [u8; 4] = [186, 124, 60, 48];

	Call::new(&FUNCTION)
		.fixed_bytes(query_id)
		.uint(timestamp)
		.bytes(value)
		.address(disputed_reporter)
		.address(dispute_initiator)
		.uint(dispute_fee)
		.uint(slash_amount)
		.encode()
}

pub(crate) fn vote(
	dispute_id: &[u8],
	total_tips_for: impl Into<U256>,
	total_tips_against: impl Into<U256>,
	total_tips_invalid: impl Into<U256>,
	total_reports_for: impl Into<U256>,
	total_reports_against: impl Into<U256>,
	total_reports_invalid: impl Into<U256>,
) -> Vec<u8> {
	const FUNCTION: [u8; 4] = [61, 181, 167, 166];

	Call::new(&FUNCTION)
		.fixed_bytes(dispute_id)
		.uint(total_tips_for)
		.uint(total_tips_against)
		.uint(total_tips_invalid)
		.uint(total_reports_for)
		.uint(total_reports_against)
		.uint(total_reports_invalid)
		.encode()
}

#[cfg(test)]
mod tests {
	use super::{super::tests::*, *};
	use ethabi::{Function, ParamType, Token};
	use sp_core::keccak_256;

	#[allow(deprecated)]
	fn begin_parachain_dispute() -> Function {
		// beginParachainDispute(bytes32,uint256,bytes,address,address,uint256,uint256)
		Function {
			name: "beginParachainDispute".to_string(),
			inputs: vec![
				param("_queryId", ParamType::FixedBytes(32)),
				param("_timestamp", ParamType::Uint(256)),
				param("_value", ParamType::Bytes),
				param("_disputedReporter", ParamType::Address),
				param("_disputeInitiator", ParamType::Address),
				param("_disputeFee", ParamType::Uint(256)),
				param("_slashAmount", ParamType::Uint(256)),
			],
			outputs: vec![],
			constant: None,
			state_mutability: Default::default(),
		}
	}

	#[test]
	#[ignore]
	fn begin_parachain_dispute_function_selector() {
		// Short signature bytes used for FUNCTION const
		let function = begin_parachain_dispute();
		println!("{} {:?}", function.signature(), function.short_signature());
	}

	#[test]
	fn encodes_begin_parachain_dispute_call() {
		let query_id = keccak_256("my_query".as_bytes());
		let timestamp = 1675711956967u64;
		let value = [
			0u8, 65, 242, 124, 97, 37, 67, 41, 189, 109, 132, 185, 252, 136, 215, 37, 101, 25, 113,
			126, 143, 68, 226, 21, 52, 30, 20, 190, 109, 250, 166, 10, 71, 121, 118, 208, 186, 68,
			115, 103, 116, 24, 76, 18, 145, 31, 14, 132, 213, 146, 98, 184, 227, 250, 43, 5, 1, 73,
			97, 130, 5,
		];
		let disputed_reporter = Address::random();
		let dispute_initiator = Address::random();
		let dispute_fee = 12345;
		let slash_amount = 54321;

		assert_eq!(
			begin_parachain_dispute()
				.encode_input(&vec![
					Token::FixedBytes(query_id.into()),
					Token::Uint(timestamp.into()),
					Token::Bytes(value.into()),
					Token::Address(disputed_reporter),
					Token::Address(dispute_initiator),
					Token::Uint(dispute_fee.into()),
					Token::Uint(slash_amount.into()),
				])
				.unwrap()[..],
			super::begin_parachain_dispute(
				&query_id,
				timestamp,
				&value,
				disputed_reporter,
				dispute_initiator,
				dispute_fee,
				slash_amount
			)[..]
		)
	}

	#[allow(deprecated)]
	fn vote() -> Function {
		// voteParachain(bytes32,uint256,uint256,uint256,uint256,uint256,uint256)
		Function {
			name: "voteParachain".to_string(),
			inputs: vec![
				param("_disputeId", ParamType::FixedBytes(32)),
				param("_totalTipsFor", ParamType::Uint(256)),
				param("_totalTipsAgainst", ParamType::Uint(256)),
				param("_totalTipsInvalid", ParamType::Uint(256)),
				param("_totalReportsFor", ParamType::Uint(256)),
				param("_totalReportsAgainst", ParamType::Uint(256)),
				param("_totalReportsInvalid", ParamType::Uint(256)),
			],
			outputs: vec![],
			constant: None,
			state_mutability: Default::default(),
		}
	}

	#[test]
	#[ignore]
	fn vote_function_selector() {
		// Short signature bytes used for FUNCTION const
		let function = vote();
		println!("{} {:?}", function.signature(), function.short_signature());
	}

	#[test]
	fn encodes_vote_call() {
		let para_id = 3000;
		let query_id = keccak_256("my_query".as_bytes());
		let timestamp = 1675711956967u64;
		let dispute_id = keccak_256(&ethabi::encode(&vec![
			Token::Uint(para_id.into()),
			Token::FixedBytes(query_id.into()).into(),
			Token::Uint(timestamp.into()),
		]));

		assert_eq!(
			vote()
				.encode_input(&vec![
					Token::FixedBytes(dispute_id.into()),
					Token::Uint(1.into()),
					Token::Uint(2.into()),
					Token::Uint(3.into()),
					Token::Uint(4.into()),
					Token::Uint(5.into()),
					Token::Uint(6.into()),
				])
				.unwrap()[..],
			super::vote(&dispute_id, 1, 2, 3, 4, 5, 6)[..]
		)
	}
}
