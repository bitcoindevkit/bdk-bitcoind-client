// SPDX-License-Identifier: MIT OR Apache-2.0

//! Error types for the Bitcoin RPC client.

use core::fmt;
#[cfg(feature = "bitreq")]
use std::io;

#[cfg(feature = "bitreq")]
use corepc_types::bitcoin::{consensus::encode::FromHexError, hex::HexToArrayError};
use jsonrpc::serde_json;

/// Errors that can occur when using the Bitcoin RPC client.
#[derive(Debug)]
pub enum Error {
    /// JSON-RPC error from the server.
    JsonRpc(jsonrpc::Error),

    /// JSON serialization/deserialization error.
    Json(serde_json::Error),

    /// Hex deserialization error.
    #[cfg(feature = "bitreq")]
    DecodeHex(FromHexError),

    /// Error converting a version-specific RPC type into the model type.
    #[cfg(feature = "bitreq")]
    Model(Box<dyn core::error::Error + Send + Sync + 'static>),

    /// Invalid or corrupted cookie file.
    #[cfg(feature = "bitreq")]
    InvalidCookieFile,

    /// The provided URL is syntactically incorrect.
    #[cfg(feature = "bitreq")]
    InvalidUrl(String),

    /// Hash parsing error.
    #[cfg(feature = "bitreq")]
    HexToArray(HexToArrayError),

    /// I/O error (e.g., reading cookie file, network issues).
    #[cfg(feature = "bitreq")]
    Io(io::Error),

    /// Error when converting an integer type to a smaller type due to overflow.
    #[cfg(feature = "bitreq")]
    TryFromInt(core::num::TryFromIntError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::JsonRpc(e) => write!(f, "JSON-RPC error: {e}"),
            Error::Json(e) => write!(f, "JSON error: {e}"),
            #[cfg(feature = "bitreq")]
            Error::DecodeHex(e) => write!(f, "hex deserialization error: {e}"),
            #[cfg(feature = "bitreq")]
            Error::Model(e) => write!(f, "model conversion error: {e}"),
            #[cfg(feature = "bitreq")]
            Error::InvalidCookieFile => write!(f, "invalid or missing cookie file"),
            #[cfg(feature = "bitreq")]
            Error::InvalidUrl(e) => write!(f, "invalid RPC URL: {e}"),
            #[cfg(feature = "bitreq")]
            Error::HexToArray(e) => write!(f, "hash parsing error: {e}"),
            #[cfg(feature = "bitreq")]
            Error::Io(e) => write!(f, "I/O error: {e}"),
            #[cfg(feature = "bitreq")]
            Error::TryFromInt(e) => write!(f, "integer conversion overflow: {e}"),
        }
    }
}

impl core::error::Error for Error {}

impl Error {
    /// Converts `e` to an [`Error::Model`] error.
    #[cfg(feature = "bitreq")]
    pub(crate) fn model<E>(e: E) -> Self
    where
        E: core::error::Error + Send + Sync + 'static,
    {
        Self::Model(Box::new(e))
    }

    /// Wraps `e` as a [`jsonrpc::Error::Transport`] inside [`Error::JsonRpc`].
    pub(crate) fn transport<E>(e: E) -> Self
    where
        E: core::error::Error + Send + Sync + 'static,
    {
        Self::JsonRpc(jsonrpc::Error::Transport(Box::new(e)))
    }
}

// Conversions from other error types.

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

#[cfg(feature = "bitreq")]
impl From<HexToArrayError> for Error {
    fn from(e: HexToArrayError) -> Self {
        Error::HexToArray(e)
    }
}

#[cfg(feature = "bitreq")]
impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

#[cfg(feature = "bitreq")]
impl From<core::num::TryFromIntError> for Error {
    fn from(e: core::num::TryFromIntError) -> Self {
        Error::TryFromInt(e)
    }
}

#[cfg(feature = "bitreq")]
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
