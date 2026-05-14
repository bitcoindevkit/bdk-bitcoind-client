// SPDX-License-Identifier: MIT OR Apache-2.0

//! Bitcoin Core RPC client library.
//!
//! The top-level [`Client`] is transport-agnostic (sans-io): callers supply
//! the transport at each call site via a `send_fn` closure.
//!
//! For a batteries-included HTTP client backed by the `bitreq` transport,
//! enable the `bitreq` feature and use [`bitreq::Client`].

mod client;
mod error;
mod rpc;

pub use client::*;
pub use error::*;
pub use rpc::*;

#[cfg(feature = "bitreq")]
pub mod bitreq;

pub use jsonrpc;

#[cfg(feature = "bitreq")]
pub use corepc_types;
