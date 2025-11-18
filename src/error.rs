//! Error types for the Bitcoin RPC client.

use std::{fmt, io};

use corepc_types::bitcoin::hex::{HexToArrayError, HexToBytesError};
use jsonrpc::serde_json;

/// Result type alias for the RPC client.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when using the Bitcoin RPC client.
#[derive(Debug)]
pub enum Error {
    /// Missing authentication credentials.
    MissingAuthentication,

    /// Invalid or corrupted cookie file.
    InvalidCookieFile,

    /// Invalid response from the RPC server.
    InvalidResponse(String),

    /// JSON-RPC error from the server.
    JsonRpc(jsonrpc::Error),

    /// Hex decoding error for byte vectors (used in get_block, etc.)
    HexToBytes(HexToBytesError),

    /// Hash parsing error.
    HexToArray(HexToArrayError),

    /// JSON serialization/deserialization error.
    Json(serde_json::Error),

    /// I/O error (e.g., reading cookie file, network issues).
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::MissingAuthentication => {
                write!(f, "authentication is required but none was provided")
            }
            Error::InvalidCookieFile => write!(f, "invalid cookie file"),
            Error::InvalidResponse(e) => write!(f, "invalid response: {e}"),
            Error::HexToBytes(e) => write!(f, "Hex to bytes error: {e}"),
            Error::HexToArray(e) => write!(f, "Hash parsing eror: {e}"),
            Error::JsonRpc(e) => write!(f, "JSON-RPC error: {e}"),
            Error::Json(e) => write!(f, "JSON error: {e}"),
            Error::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::JsonRpc(e) => Some(e),
            Error::Json(e) => Some(e),
            Error::Io(e) => Some(e),
            Error::HexToBytes(e) => Some(e),
            Error::HexToArray(e) => Some(e),
            _ => None,
        }
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
