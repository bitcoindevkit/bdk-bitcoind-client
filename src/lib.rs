//! Bitcoin Core RPC client library.
//!
//! This crate provides a Rust client for interacting with Bitcoin Core's JSON-RPC interface.
//! It supports multiple authentication methods and provides a type-safe interface for
//! making RPC calls to a Bitcoin Core daemon.

#[macro_use]
#[allow(unused_imports)]
extern crate alloc;

mod client;
mod error;
mod rpc;
pub use rpc::*;
#[cfg(feature = "bitreq")]
pub mod bitreq;

pub use client::*;
pub use error::*;

pub use jsonrpc;
