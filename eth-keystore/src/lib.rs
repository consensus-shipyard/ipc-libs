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
    type Key: Clone + Eq + Hash + Into<String> + TryFrom<KeyInfo>;

    /// Get the key info by address string
    fn get(&self, addr: &Self::Key) -> Result<Option<KeyInfo>>;
    /// List all addresses in the key store
    fn list_all(&self) -> Result<Vec<Self::Key>>;
    /// Put a new info to the addr
    fn put(&mut self, info: KeyInfo) -> Result<()>;
}

/// The struct that contains evm private key info
#[derive(Clone)]
pub struct KeyInfo {
    private_key: Vec<u8>,
}

impl KeyInfo {
    pub fn private_key(&self) -> &[u8] {
        &self.private_key
    }
}

impl Drop for KeyInfo {
    fn drop(&mut self) {
        self.private_key.zeroize();
    }
}
