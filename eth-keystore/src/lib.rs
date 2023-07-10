#![feature(let_chains)]
// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

//! Ethereum wallet key store.

mod memory;
mod persistent;

use anyhow::Result;
use std::hash::Hash;
use zeroize::Zeroize;

pub use crate::persistent::PersistentKeyStore;

/// The key store trait for different evm key store
pub trait KeyStore {
    /// The type of the key that is stored
    type Key: Clone + Eq + Hash + TryFrom<KeyInfo>;

    /// Get the key info by address string
    fn get(&self, addr: &Self::Key) -> Result<Option<KeyInfo>>;
    /// List all addresses in the key store
    fn list(&self) -> Result<Vec<Self::Key>>;
    /// Put a new info to the addr
    fn put(&mut self, info: KeyInfo) -> Result<()>;
    /// Remove address from the key store
    fn remove(&mut self, addr: &Self::Key) -> Result<()>;
}

/// The struct that contains evm private key info
#[derive(Clone, PartialEq, Debug)]
pub struct KeyInfo {
    private_key: Vec<u8>,
}

impl Drop for KeyInfo {
    fn drop(&mut self) {
        self.private_key.zeroize();
    }
}

#[cfg(feature = "with-ethers")]
impl TryFrom<KeyInfo> for ethers::types::Address {
    type Error = anyhow::Error;

    fn try_from(value: KeyInfo) -> std::result::Result<Self, Self::Error> {
        use ethers::signers::Signer;
        let key = ethers::signers::Wallet::from_bytes(&value.private_key)?;
        Ok(key.address())
    }
}
