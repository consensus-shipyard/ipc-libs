//! The ipc-agent json rpc node.

use std::fmt::Debug;
use anyhow::Result;
use serde::de::DeserializeOwned;
use async_trait::async_trait;

pub mod jsonrpc;
pub mod request;
pub mod response;

/// The JSON RPC server request handler trait.
#[async_trait]
pub trait JsonRPCRequestHandler {
    type Request: Debug;
    type Response: Debug + DeserializeOwned;

    /// Handles the request sent to the json rpc server. Returns a response back.
    async fn handle(&self, request: &Self::Request) -> Result<Self::Response>;
}