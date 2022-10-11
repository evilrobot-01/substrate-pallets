use codec::Codec;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
    types::error::CallError,
    types::ErrorObject,
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_rpc::number::NumberOrHex;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

pub use pallet_dex_runtime_api::DexApi as DexRuntimeApi;

// DEX RPC methods
#[rpc(client, server)]
pub trait DexApi<Balance, AssetId>
where
    Balance: Copy + TryFrom<NumberOrHex> + Into<NumberOrHex>,
{
    #[method(name = "dex_price")]
    fn price(&self, amount: Balance, asset: AssetId, other: AssetId) -> RpcResult<Balance>;
}

pub struct Dex<Client, Block> {
    client: Arc<Client>,
    _marker: std::marker::PhantomData<Block>,
}

impl<Client, Block> Dex<Client, Block> {
    /// Creates a new instance of the Dex Rpc helper.
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

/// Error type of this RPC api.
pub enum Error {
    /// The transaction was not decodable.
    DecodeError,
    /// The call to runtime failed.
    RuntimeError,
}

impl From<Error> for i32 {
    fn from(e: Error) -> i32 {
        match e {
            Error::RuntimeError => 1,
            Error::DecodeError => 2,
        }
    }
}

#[async_trait]
impl<Client, Block, Balance, AssetId> DexApiServer<Balance, AssetId> for Dex<Client, Block>
where
    Client: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    Client::Api: DexRuntimeApi<Block, Balance, AssetId>,
    Block: BlockT,
    Balance: Codec + Copy + TryFrom<NumberOrHex> + Into<NumberOrHex>,
    AssetId: Codec,
{
    fn price(&self, amount: Balance, asset: AssetId, other: AssetId) -> RpcResult<Balance> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(self.client.info().best_hash);
        let balance = api.price(&at, amount, asset, other).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Runtime error",
                Some(e.to_string()),
            ))
        })?;
        Ok(balance)
    }
}
