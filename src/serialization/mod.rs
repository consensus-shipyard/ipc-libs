// Copyright 2022-2023 Protocol Labs
//! Handles the serialization of different types between actor cbor tuple serialization and json rpc
//! json serialization.

mod checkpoint;

/// The trait to implement is we want to serialize directly to json. Most of the types should have no
/// need to implement this trait. But some types that are shared between actor using cbor tuple serialization
/// and json rpc response, we are using `AsJson` wrapper to handle convert to json instead.
#[derive(Debug)]
pub struct AsJson<T>(pub T);
