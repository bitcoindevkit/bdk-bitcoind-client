// SPDX-License-Identifier: MIT OR Apache-2.0

//! Error types for the Bitcoin RPC client.

use core::fmt;
use core::num::TryFromIntError;
use std::io;

use bitcoin::{consensus::encode::FromHexError, hex::HexToArrayError};
use corepc_types::bitcoin;
use jsonrpc::serde_json;

/// Errors that can occur when using the Bitcoin RPC client.
#[derive(Debug)]
pub enum Error {
    /// Hex deserialization error
    DecodeHex(FromHexError),

    /// Error converting a version-specific RPC type into the model type.
    Model(Box<dyn core::error::Error + Send + Sync + 'static>),

    /// Invalid or corrupted cookie file.
    InvalidCookieFile,

    /// The provided URL is syntactically incorrect
    InvalidUrl(String),

    /// JSON-RPC error from the server.
    JsonRpc(jsonrpc::Error),

    /// Hash parsing error.
    HexToArray(HexToArrayError),

    /// JSON serialization/deserialization error.
    Json(serde_json::Error),

    /// I/O error (e.g., reading cookie file, network issues).
    Io(io::Error),

    /// Error when converting an integer type to a smaller type due to overflow.
    TryFromInt(TryFromIntError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::DecodeHex(e) => write!(f, "hex deserialization error: {e}"),
            Error::Model(e) => write!(f, "model conversion error: {e}"),
            Error::InvalidCookieFile => write!(f, "invalid or missing cookie file"),
            Error::InvalidUrl(e) => write!(f, "invalid RPC URL: {e}"),
            Error::HexToArray(e) => write!(f, "hash parsing error: {e}"),
            Error::JsonRpc(e) => write!(f, "JSON-RPC error: {e}"),
            Error::Json(e) => write!(f, "JSON error: {e}"),
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::TryFromInt(e) => write!(f, "integer conversion overflow: {e}"),
        }
    }
}

impl core::error::Error for Error {}

impl Error {
    /// Converts `e` to a [`Error::Model`] error.
    pub(crate) fn model<E>(e: E) -> Self
    where
        E: core::error::Error + Send + Sync + 'static,
    {
        Self::Model(Box::new(e))
    }
}

// Conversions from other error types
impl From<jsonrpc::Error> for Error {
    fn from(e: jsonrpc::Error) -> Self {
        Error::JsonRpc(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}

impl From<HexToArrayError> for Error {
    fn from(e: HexToArrayError) -> Self {
        Error::HexToArray(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<TryFromIntError> for Error {
    fn from(e: TryFromIntError) -> Self {
        Error::TryFromInt(e)
    }
}

impl From<FromHexError> for Error {
    fn from(e: FromHexError) -> Self {
        Error::DecodeHex(e)
    }
}

/// Extension methods for the client error type.
impl Error {
    /// Returns `true` if this is a "not found" error returned by `bitcoind`.
    ///
    /// `bitcoind` returns error code `-5` (`RPC_INVALID_ADDRESS_OR_KEY`)
    /// whenever a requested block hash, transaction ID, address, or similar object
    /// does not exist on the node.
    pub fn is_not_found_error(&self) -> bool {
        if let Error::JsonRpc(jsonrpc::Error::Rpc(rpc_err)) = self {
            rpc_err.code == -5
        } else {
            false
        }
    }
}
