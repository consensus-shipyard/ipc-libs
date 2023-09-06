// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

//! Memory key store

use crate::evm::{KeyInfo, KeyStore};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::hash::Hash;

use super::Defaultable;

#[derive(Default)]
pub struct MemoryKeyStore<T: Defaultable> {
    pub(crate) data: HashMap<T, KeyInfo>,
    pub(crate) default: Option<T>,
}

impl<T: Clone + Eq + Hash + TryFrom<KeyInfo> + Defaultable> KeyStore for MemoryKeyStore<T> {
    type Key = T;

    fn get(&self, addr: &Self::Key) -> Result<Option<KeyInfo>> {
        Ok(self.data.get(addr).cloned())
    }

    fn list(&self) -> Result<Vec<Self::Key>> {
        Ok(self.data.keys().cloned().collect())
    }

    fn put(&mut self, info: KeyInfo) -> Result<Self::Key> {
        let addr = Self::Key::try_from(info.clone())
            .map_err(|_| anyhow!("cannot convert private key to public key"))?;
        self.data.insert(addr.clone(), info);
        Ok(addr)
    }

    fn remove(&mut self, addr: &Self::Key) -> Result<()> {
        // if the address is the default, remove also from the
        // default key
        if self.default == Some(*addr) {
            self.default = None;
            self.remove(&Self::Key::default_key())?;
        }
        self.data.remove(addr);
        Ok(())
    }

    fn set_default(&mut self, addr: &Self::Key) -> Result<()> {
        let info = self.get(addr)?;
        match info {
            Some(i) => self.data.insert(Self::Key::default_key(), i),
            None => return Err(anyhow!("can't set default key: not found in keystore")),
        };

        self.default = Some(addr.clone());
        Ok(())
    }

    fn get_default(&mut self) -> Result<Self::Key> {
        // check the map if it doesn't exists
        if self.default.is_none() {
            if let Some(info) = self.get(&Self::Key::default_key())? {
                self.default = Some(
                    Self::Key::try_from(info)
                        .map_err(|_| Err(anyhow!("couldn't get address from key info"))),
                );
                return Ok(self.default.unwrap());
            }
        }

        // if it exists return it directly
        Ok(self.default.unwrap())
    }
}