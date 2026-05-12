// SPDX-License-Identifier: MIT OR Apache-2.0

use core::sync::atomic::{AtomicUsize, Ordering};

use jsonrpc::serde;
use jsonrpc::serde_json::value::RawValue;
use jsonrpc::serde_json::{self, Value, json};
use jsonrpc::{Request, Response};

use crate::Error;

/// JSON-RPC protocol version.
const JSONRPC: &str = "2.0";

/// Bitcoin Core JSON-RPC client (sans-io).
///
/// Manages request IDs and handles JSON-RPC request building and response
/// deserialization. Does not perform any I/O — callers supply the transport
/// via `send_fn` at each call site.
///
/// This type is the low-level building block used by transport-specific clients
/// such as [`bitreq::Client`](crate::bitreq::Client). It can also be used
/// directly when you need to supply your own transport.
#[derive(Debug, Default)]
pub struct Client {
    id: AtomicUsize,
}

impl Client {
    /// Creates a new [`Client`].
    pub fn new() -> Self {
        Self {
            id: AtomicUsize::new(0),
        }
    }

    /// Calls an RPC method using the provided `send_fn`.
    ///
    /// Builds a JSON-RPC [`Request`], passes it to `send_fn`, and deserializes
    /// the [`Response`] into `T`.
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails, `send_fn` returns an error
    /// (wrapped as [`Error::JsonRpc`] via a transport error), or the response
    /// contains a JSON-RPC error (also wrapped as [`Error::JsonRpc`]).
    pub fn call<T, E>(
        &self,
        method: &str,
        params: &[Value],
        send_fn: impl Fn(Request) -> Result<Response, E>,
    ) -> Result<T, Error>
    where
        T: for<'de> serde::Deserialize<'de>,
        E: core::error::Error + Send + Sync + 'static,
    {
        let raw = if params.is_empty() {
            None
        } else {
            Some(serde_json::value::to_raw_value(params)?)
        };
        let request = self.build_request(method, raw.as_deref());
        let request_id = request.id.clone();
        let response = send_fn(request).map_err(Error::transport)?;
        if response.id != request_id {
            return Err(Error::JsonRpc(jsonrpc::Error::NonceMismatch));
        }
        Ok(response.result()?)
    }

    /// Builds a JSON-RPC [`Request`] with an auto-incremented ID.
    fn build_request<'a>(&self, method: &'a str, params: Option<&'a RawValue>) -> Request<'a> {
        let id = self.id.fetch_add(1, Ordering::Relaxed);
        Request {
            method,
            params,
            id: json!(id),
            jsonrpc: Some(JSONRPC),
        }
    }
}
