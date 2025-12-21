//! Error types for the Bitcoin RPC client.

use bitcoin::{
    consensus::encode::FromHexError,
    hex::{HexToArrayError, HexToBytesError},
};
#[cfg(feature = "28_0")]
use corepc_types::v17::{GetBlockHeaderVerboseError, GetBlockVerboseOneError};
#[cfg(not(feature = "28_0"))]
use corepc_types::v30::{GetBlockHeaderVerboseError, GetBlockVerboseOneError};
use corepc_types::{bitcoin, v30::GetBlockFilterError};
use jsonrpc::serde_json;
use std::{fmt, io, num::TryFromIntError};

/// Result type alias for the RPC client.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when using the Bitcoin RPC client.
#[derive(Debug)]
pub enum Error {
    /// Hex deserialization error
    DecodeHex(FromHexError),

    /// Error converting `GetBlockVersboseOne` type into the model type
    GetBlockVerboseOneError(GetBlockVerboseOneError),

    /// Error modeling [`GetBlockHeaderVerbose`](corepc_types::model::GetBlockHeaderVerbose).
    GetBlockHeaderVerboseError(GetBlockHeaderVerboseError),

    /// Error modeling [`GetBlockFilter`](corepc_types::model::GetBlockFilter)
    GetBlockFilterError(GetBlockFilterError),

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

    /// Error when converting an integer type to a smaller type due to overflow.
    Overflow(TryFromIntError),
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
            Error::DecodeHex(e) => write!(f, "Hex deserialization error: {e}"),
            Error::GetBlockHeaderVerboseError(e) => write!(f, "{e}"),
            Error::GetBlockVerboseOneError(e) => write!(f, "{e}"),
            Error::Overflow(e) => write!(f, "Integer conversion overflow error: {e}"),
            Error::GetBlockFilterError(e) => write!(f, "{e}"),
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
            Error::DecodeHex(e) => Some(e),
            Error::GetBlockVerboseOneError(e) => Some(e),
            Error::Overflow(e) => Some(e),
            Error::GetBlockFilterError(e) => Some(e),
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

impl From<TryFromIntError> for Error {
    fn from(e: TryFromIntError) -> Self {
        Error::Overflow(e)
    }
}

impl From<GetBlockVerboseOneError> for Error {
    fn from(e: GetBlockVerboseOneError) -> Self {
        Error::GetBlockVerboseOneError(e)
    }
}

impl From<FromHexError> for Error {
    fn from(e: FromHexError) -> Self {
        Error::DecodeHex(e)
    }
}
