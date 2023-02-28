//! The module contains the handlers implementation for the json rpc server.

pub mod create;
mod subnet;

pub enum HandlersBundler {
    CreateSubnet()
}