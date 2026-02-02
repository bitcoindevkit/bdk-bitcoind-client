//! Error types for the Bitcoin RPC client.

use alloc::boxed::Box;
use core::num::TryFromIntError;
use std::{fmt, io};

use corepc_types::bitcoin::hex::HexToArrayError;
use jsonrpc::serde_json;

/// Errors that can occur when using the Bitcoin RPC client.
#[derive(Debug)]
pub enum Error {
    /// Hash parsing error.
    HexToArray(HexToArrayError),
    /// I/O error (e.g., reading cookie file, network issues).
    Io(io::Error),
    /// Invalid or corrupted cookie file.
    InvalidCookieFile,
    /// JSON serialization/deserialization error.
    Json(serde_json::Error),
    /// JSON-RPC error from the server.
    JsonRpc(jsonrpc::Error),
    /// model types error
    Model(Box<dyn core::error::Error + Send + Sync>),
    /// nonce mismatch
    NonceMismatch,
    /// integer conversion
    TryFromInt(TryFromIntError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::HexToArray(e) => write!(f, "{e}"),
            Self::Io(e) => write!(f, "{e}"),
            Self::InvalidCookieFile => write!(f, "invalid cookie file"),
            Self::Json(e) => write!(f, "{e}"),
            Self::JsonRpc(e) => write!(f, "{e}"),
            Self::Model(e) => write!(f, "{e}"),
            Self::NonceMismatch => write!(f, "mismatched nonce"),
            Self::TryFromInt(e) => write!(f, "{e}"),
        }
    }
}

impl core::error::Error for Error {}

impl Error {
    /// Convert `e` to a [`Error::Model`] error.
    #[cfg(feature = "bitreq")]
    pub(crate) fn model<E>(e: E) -> Self
    where
        E: core::error::Error + Send + Sync + 'static,
    {
        Self::Model(Box::new(e))
    }

    /// Convert `e` to a [`jsonrpc::Error::Transport`] error.
    pub(crate) fn transport<E>(e: E) -> Self
    where
        E: core::error::Error + Send + Sync + 'static,
    {
        Self::JsonRpc(jsonrpc::Error::Transport(Box::new(e)))
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
