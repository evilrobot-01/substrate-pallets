use crate as pallet_redstone;
use frame_support::parameter_types;
use frame_support::traits::{ConstU16, ConstU64};
use frame_system as system;
use sp_core::{ConstU32, ConstU8, H256};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        RedStoneExample: pallet_redstone,
    }
);

impl system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

type Moment = u64;

impl pallet_timestamp::Config for Test {
    type Moment = Moment;
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<1>;
    type WeightInfo = ();
}

const SIGNER_1_PUB_KEY: [u8; 33] = [
    3, 79, 53, 91, 220, 183, 204, 10, 247, 40, 239, 60, 206, 185, 97, 93, 144, 104, 75, 181, 178,
    202, 95, 133, 154, 176, 240, 183, 4, 7, 88, 113, 170,
];
const SIGNER_2_PUB_KEY: [u8; 33] = [
    2, 70, 109, 127, 202, 229, 99, 229, 203, 9, 160, 209, 135, 11, 181, 128, 52, 72, 4, 97, 120,
    121, 161, 73, 73, 207, 34, 40, 95, 27, 174, 63, 39,
];

parameter_types! {
    pub const AuthorisedSigners: [[u8;33];2] = [SIGNER_1_PUB_KEY, SIGNER_2_PUB_KEY];
    pub const DataFeedId: [u8;32] = [66, 84, 67, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
}

impl pallet_redstone::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AuthorisedSigners = AuthorisedSigners;
    type DataFeedId = DataFeedId;
    type MaxPayloadLen = ConstU32<1000>;
    type UniqueSignersThreshold = ConstU8<1>;
    type Moment = Moment;
    type Time = Timestamp;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
