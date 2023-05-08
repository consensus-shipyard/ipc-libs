// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! wallet handlers and parameters

use crate::identity::json::KeyInfoJson;
use crate::identity::{KeyInfo, Wallet};
use crate::server::JsonRPCRequestHandler;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletImportParams {
    pub key_info: KeyInfoJson,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletImportResponse {
    pub address: String,
}

/// Send value between two addresses within a subnet
pub(crate) struct WalletImportHandler {
    wallet: Arc<RwLock<Wallet>>,
}

impl WalletImportHandler {
    pub(crate) fn new(wallet: Arc<RwLock<Wallet>>) -> Self {
        Self { wallet }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for WalletImportHandler {
    type Request = WalletImportParams;
    type Response = WalletImportResponse;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let mut wallet = self.wallet.write().unwrap();
        let key_info = KeyInfo::try_from(request.key_info)?;
        let address = wallet.import(key_info)?;

        Ok(WalletImportResponse {
            address: address.to_string(),
        })
    }
}
