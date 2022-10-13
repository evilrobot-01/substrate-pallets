use crate as pallet_marketplace;
use frame_support::dispatch::DispatchResult;
use frame_support::traits::{AsEnsureOriginWithArg, ConstU16, ConstU64, GenesisBuild};
use frame_support::{parameter_types, PalletId};
use frame_system as system;
use frame_system::{EnsureRoot, EnsureSigned};
use sp_core::H256;
use sp_runtime::traits::{ConstU128, ConstU32};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    DispatchError,
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
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        Assets: pallet_assets::{Pallet, Call, Storage, Config<T>, Event<T>},
        DEX: pallet_dex::{Pallet, Call, Storage, Event<T>},
        Uniques: pallet_uniques::{Pallet, Call, Storage, Event<T>},
        Marketplace: pallet_marketplace::{Pallet, Call, Storage, Event<T>},
    }
);

pub type AssetId = u32;
pub type AccountId = u64;
pub type Balance = u128;
pub type CollectionId = u64;
pub type ItemId = u32;

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
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

impl pallet_assets::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type AssetId = AssetId;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<AccountId>;
    type AssetDeposit = ();
    type AssetAccountDeposit = ();
    type MetadataDepositBase = ();
    type MetadataDepositPerByte = ();
    type ApprovalDeposit = ();
    type StringLimit = ConstU32<25>;
    type Freezer = ();
    type Extra = ();
    type WeightInfo = ();
}

impl pallet_balances::Config for Test {
    type Balance = Balance;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ConstU128<1>;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
}

parameter_types! {
    pub const DEXPallet: PalletId = PalletId(*b"py/de-ex");
    pub const LiquidityPoolTokenMinimumBalance: u32 = 1;
}

impl pallet_dex::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AssetId = AssetId;
    type Assets = Assets;
    type LiquidityPoolTokenMinimumBalance = LiquidityPoolTokenMinimumBalance;
    type LiquidityPoolTokenDecimals = ();
    type MinimumLiquidity = ();
    type NativeCurrency = Balances;
    type NativeAssetId = ();
    type PalletId = DEXPallet;
    type SwapFeeUnits = ConstU128<1000>;
    type SwapFeeValue = ConstU128<997>;
    type Time = Timestamp;

    fn exists(id: Self::AssetId) -> bool {
        Assets::maybe_total_supply(id).is_some()
    }
}

impl pallet_marketplace::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AssetId = AssetId;
    type Assets = Assets;
    type CollectionId = CollectionId;
    type DEX = proxies::Dex;
    type ItemId = ItemId;
    type NativeAssetId = ();
    type NativeCurrency = Balances;
    type Uniques = Uniques;

    fn exists(id: Self::AssetId) -> bool {
        Assets::maybe_total_supply(id).is_some()
    }
}

impl pallet_timestamp::Config for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ConstU64<5>;
    type WeightInfo = ();
}

impl pallet_uniques::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type CollectionId = CollectionId;
    type ItemId = ItemId;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<AccountId>;
    type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
    type Locker = Marketplace;
    type CollectionDeposit = ();
    type ItemDeposit = ();
    type MetadataDepositBase = ();
    type AttributeDepositBase = ();
    type DepositPerByte = ();
    type StringLimit = ConstU32<20>;
    type KeyLimit = ();
    type ValueLimit = ();
    type WeightInfo = ();
}

pub(crate) const NATIVE_TOKEN: u32 = 0;
pub(crate) const ASSET_1: u32 = 1;
pub(crate) const ASSET_2: u32 = 2;
pub(crate) const BUYER: u64 = 3;
pub(crate) const OWNER: u64 = 2;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into();

    const ADMIN: u64 = 0;
    const MIN_BALANCE: u128 = 1;
    const LIQUIDITY_PROVIDER: u64 = 123;

    // Set balances of native token
    let config: pallet_balances::GenesisConfig<Test> = pallet_balances::GenesisConfig {
        balances: vec![(OWNER, 100), (LIQUIDITY_PROVIDER, 100), (BUYER, 10)],
    };
    config.assimilate_storage(&mut storage).unwrap();

    // Configure assets
    let config: pallet_assets::GenesisConfig<Test> = pallet_assets::GenesisConfig {
        assets: vec![
            // id, owner, is_sufficient, min_balance
            (NATIVE_TOKEN, ADMIN, true, MIN_BALANCE), // Proxy for native token
            (ASSET_1, ADMIN, true, MIN_BALANCE),
            (ASSET_2, ADMIN, true, MIN_BALANCE),
        ],
        metadata: vec![
            // id, name, symbol, decimals
            (NATIVE_TOKEN, "Native (Proxy)".into(), "UNIT".into(), 10),
            (ASSET_1, "Token 1".into(), "TOK1".into(), 10),
            (ASSET_2, "Token 2".into(), "TOK2".into(), 10),
        ],
        accounts: vec![
            // id, account_id, balance
            (ASSET_1, LIQUIDITY_PROVIDER, 1000),
            (ASSET_1, BUYER, 150),
        ],
    };
    config.assimilate_storage(&mut storage).unwrap();

    // Configure DEX liquidity pools
    let config: pallet_dex::GenesisConfig<Test> = pallet_dex::GenesisConfig {
        liquidity_pools: vec![((100, NATIVE_TOKEN), (1000, ASSET_1), LIQUIDITY_PROVIDER)],
    };
    config.assimilate_storage(&mut storage).unwrap();

    let mut ext: sp_io::TestExternalities = storage.into();
    ext.execute_with(|| System::set_block_number(1));
    ext
}

mod proxies {
    use crate::mock::*;

    pub struct Dex;
    impl crate::traits::Price for Dex {
        type AssetId = AssetId;
        type Balance = Balance;

        fn price(
            amount: Self::Balance,
            asset: Self::AssetId,
            other: Self::AssetId,
        ) -> Result<Self::Balance, DispatchError> {
            DEX::price(amount, asset, other)
        }
    }
    impl crate::traits::Swap<<Test as frame_system::Config>::AccountId> for Dex {
        type AssetId = AssetId;
        type Balance = Balance;

        fn swap(
            amount: Self::Balance,
            asset: Self::AssetId,
            other: Self::AssetId,
            buyer: &<Test as frame_system::Config>::AccountId,
        ) -> DispatchResult {
            <DEX as pallet_dex::traits::Swap<<Test as frame_system::Config>::AccountId>>::swap(
                amount, asset, other, buyer,
            )
        }
    }
}
