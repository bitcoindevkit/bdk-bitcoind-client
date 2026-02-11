//! Error types for the Bitcoin RPC client.

use bitcoin::{consensus::encode::FromHexError, hex::HexToArrayError};
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
    GetBlockVerboseOne(GetBlockVerboseOneError),

    /// Error modeling [`GetBlockHeaderVerbose`](corepc_types::model::GetBlockHeaderVerbose).
    GetBlockHeaderVerbose(GetBlockHeaderVerboseError),

    /// Error modeling [`GetBlockFilter`](corepc_types::model::GetBlockFilter)
    GetBlockFilter(GetBlockFilterError),

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
            Error::GetBlockVerboseOne(e) => write!(f, "block verbose error: {e}"),
            Error::GetBlockHeaderVerbose(e) => write!(f, "block header verbose error: {e}"),
            Error::GetBlockFilter(e) => write!(f, "block filter error: {e}"),
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

impl std::error::Error for Error {}

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

impl From<GetBlockVerboseOneError> for Error {
    fn from(e: GetBlockVerboseOneError) -> Self {
        Error::GetBlockVerboseOne(e)
    }
}

impl From<FromHexError> for Error {
    fn from(e: FromHexError) -> Self {
        Error::DecodeHex(e)
    }
}
