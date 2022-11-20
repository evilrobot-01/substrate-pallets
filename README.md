License: Unlicense

# Tellor
A sample implementation of some of the more notable functionality available within the Tellor smart contracts. It is assumed that the Tributes TRB token is the native token for simplicity.

Note: Each sample pallet is currently implemented as a different module with feature flags.

## [Flex](src/flex)
Oracle handling staking, reporting and reading of data, as well as slashing and data removal as instructed via Governance.
- `traits::Governance` allows injecting governance functionality required by the pallet through runtime configuration. See the `Governance` type within the pallet `Config`.
- `traits::RuntimeApi` exposes 'getters', aligned to existing smart contract APIs.

## [Governance](src/governance)
'Controls data disputes and voting'.
- `traits::Oracle` allows injecting oracle functionality required by the pallet through runtime configuration. See the `Oracle` type within the pallet `Config`.
- `traits::RuntimeApi` exposes 'getters' aligned to existing smart contract APIs.

## [AutoPay](src/autopay)
'Keeps track of reporters data submissions and user tips for reporters and allows reporters to get their rewards/get paid’.
  - `traits::QueryDataStorage` allows injecting query data storage functionality required by the pallet through runtime configuration. See the `QueryData` type within the pallet `Config`.
  - `traits::RuntimeApi` exposes 'getters' aligned to existing smart contract APIs.