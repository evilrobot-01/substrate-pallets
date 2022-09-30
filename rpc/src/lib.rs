use codec::Codec;
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
	//types::error::{CallError, ErrorObject},
};
use sp_api::ProvideRuntimeApi;
use sp_runtime::traits::Block as BlockT;
use std::fmt::Display;
use std::sync::Arc;

pub use pallet_dex_runtime_api::DexApi as DexRuntimeApi;

#[rpc(client, server)]
pub trait DexApi<Balance, AssetId> {
	#[method(name = "price")]
	fn price(&self, amount: Balance, asset: AssetId, other: AssetId) -> RpcResult<Balance>;
}

pub struct Dex<C, P, B, A> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<P>,
	_balance: std::marker::PhantomData<B>,
	_asset_id: std::marker::PhantomData<A>,
}

impl<C, P, B, A> Dex<C, P, B, A> {
	/// Creates a new instance of the Dex Rpc helper.
	pub fn new(client: Arc<C>) -> Self {
		Self {
			client,
			_marker: Default::default(),
			_balance: Default::default(),
			_asset_id: Default::default(),
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
impl<C, Block, Balance, AssetId> DexApiServer<Balance, AssetId> for Dex<C, Block, Balance, AssetId>
where
	C: ProvideRuntimeApi<Block> + Send + Sync + 'static,
	C::Api: DexRuntimeApi<Block, Balance, AssetId>,
	Block: BlockT,
	Balance: Clone + Codec + Display + Send + Sync + 'static,
	AssetId: Clone + Codec + Display + Send + Sync + 'static,
{
	fn price(&self, _amount: Balance, _asset: AssetId, _other: AssetId) -> RpcResult<Balance> {
		let _api = self.client.runtime_api();
		todo!()
		// NOTE: the below is currently noy compiling unfortunately
		// let balance = api.price(amount, asset, other).map_err(|e| {
		// 	CallError::Custom(ErrorObject::owned(
		// 		Error::RuntimeError.into(),
		// 		"Unable to query nonce.",
		// 		Some(e.to_string()),
		// 	))
		// })?;
		// Ok(balance)
	}
}
