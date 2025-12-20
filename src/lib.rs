//! Bitcoin Core RPC client library.
//!
//! This crate provides a Rust client for interacting with Bitcoin Core's JSON-RPC interface.
//! It supports multiple authentication methods and provides a type-safe interface for
//! making RPC calls to a Bitcoin Core daemon.

mod client;
mod error;

pub use client::{Auth, Client};
pub use error::{Error, Result};

pub use jsonrpc;
// Re-export corepc_types
pub use corepc_types;
