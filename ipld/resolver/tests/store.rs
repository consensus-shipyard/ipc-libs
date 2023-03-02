use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use anyhow::Result;
use fvm_ipld_blockstore::Blockstore;
use libipld::Cid;
use libp2p_bitswap::BitswapStore;

#[derive(Debug, Clone, Default)]
pub struct TestBlockstore {
    blocks: Arc<RwLock<HashMap<Cid, Vec<u8>>>>,
}

impl Blockstore for TestBlockstore {
    fn has(&self, k: &Cid) -> Result<bool> {
        todo!()
    }

    fn get(&self, k: &Cid) -> Result<Option<Vec<u8>>> {
        todo!()
    }

    fn put_keyed(&self, k: &Cid, block: &[u8]) -> Result<()> {
        todo!()
    }
}

pub type TestStoreParams = libipld::DefaultParams;

impl BitswapStore for TestBlockstore {
    type Params = TestStoreParams;

    fn contains(&mut self, cid: &Cid) -> Result<bool> {
        todo!()
    }

    fn get(&mut self, cid: &Cid) -> Result<Option<Vec<u8>>> {
        todo!()
    }

    fn insert(&mut self, block: &libipld::Block<Self::Params>) -> Result<()> {
        todo!()
    }

    fn missing_blocks(&mut self, cid: &Cid) -> Result<Vec<Cid>> {
        todo!()
    }
}
