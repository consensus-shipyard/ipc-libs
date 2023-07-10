// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

//! Persistent file key store

use crate::memory::MemoryKeyStore;
use crate::{KeyInfo, KeyStore};
use anyhow::anyhow;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use zeroize::Zeroize;

#[derive(Default)]
pub struct PersistentKeyStore<T> {
    memory: MemoryKeyStore<T>,
    file_path: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct PersistentKeyInfo {
    /// The address associated with the private key. We can derive this from the private key
    /// but for the ease of debugging, we keep this field
    address: String,
    /// Hex encoded private key
    private_key: String,
}

impl Drop for PersistentKeyInfo {
    fn drop(&mut self) {
        self.private_key.zeroize();
    }
}

impl<T: Clone + Eq + Hash + AsRef<[u8]> + TryFrom<KeyInfo>> KeyStore for PersistentKeyStore<T> {
    type Key = T;

    fn get(&self, addr: &Self::Key) -> Result<Option<KeyInfo>> {
        self.memory.get(addr)
    }

    fn list_all(&self) -> Result<Vec<Self::Key>> {
        self.memory.list_all()
    }

    fn put(&mut self, info: KeyInfo) -> Result<()> {
        self.memory.put(info)?;
        self.write_all_no_encryption()
    }
}

impl<T: Clone + Eq + Hash + AsRef<[u8]> + TryFrom<KeyInfo>> PersistentKeyStore<T> {
    pub fn new(path: PathBuf) -> Result<Self> {
        let reader = BufReader::new(File::open(&path)?);

        let persisted_key_info: Vec<PersistentKeyInfo> =
            serde_json::from_reader(reader).map_err(|e| {
                anyhow!(
                    "failed to deserialize keyfile, initializing new keystore at: {:?} due to: {e:}",
                    path
                )
            })?;

        let mut key_infos = HashMap::new();
        for info in persisted_key_info.iter() {
            let key_info = KeyInfo {
                private_key: hex::decode(&info.private_key)?,
            };
            let addr = T::try_from(key_info.clone())
                .map_err(|_| anyhow!("cannot convert private key to address"))?;

            key_infos.insert(addr, key_info);
        }

        Ok(Self {
            memory: MemoryKeyStore { data: key_infos },
            file_path: path,
        })
    }

    /// Write all keys to file without any encryption.
    fn write_all_no_encryption(&self) -> Result<()> {
        let dir = self
            .file_path
            .parent()
            .ok_or_else(|| anyhow!("Key store parent path not exists"))?;

        fs::create_dir_all(dir)?;

        let file = File::create(&self.file_path)?;

        // TODO: do we need to set path permission?

        let writer = BufWriter::new(file);

        let to_persist = self
            .memory
            .data
            .iter()
            .map(|(key, val)| {
                let address = hex::encode(key.as_ref());
                let private_key = hex::encode(&val.private_key);
                PersistentKeyInfo {
                    address,
                    private_key,
                }
            })
            .collect::<Vec<_>>();

        serde_json::to_writer_pretty(writer, &to_persist)
            .map_err(|e| anyhow!("failed to serialize and write key info: {e}"))?;

        Ok(())
    }
}
