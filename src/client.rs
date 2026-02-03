use core::sync::atomic::{AtomicUsize, Ordering};

use jsonrpc::serde_json;
use jsonrpc::{Request, Response};
use serde::Deserialize;
use serde_json::{
    json,
    value::{RawValue, Value},
};

use crate::error::Error;
use crate::RpcMethod;

/// JSONRPC protocol version.
const JSONRPC: &str = "2.0";

/// Client
#[derive(Debug)]
pub struct Client {
    nonce: AtomicUsize,
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    /// New.
    pub fn new() -> Self {
        Self {
            nonce: AtomicUsize::new(0),
        }
    }

    /// Execute the RPC.
    pub fn call<T, E>(
        &self,
        rpc: impl RpcMethod,
        params: &[Value],
        send_fn: impl Fn(Request) -> Result<Response, E>,
    ) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de>,
        E: std::error::Error + Send + Sync + 'static,
    {
        let method = rpc.method();
        let raw_value = if params.is_empty() {
            None
        } else {
            Some(serde_json::value::to_raw_value(params)?)
        };
        let request = self.request(&method, raw_value.as_deref());
        let request_id = request.id.clone();
        let response = send_fn(request).map_err(Error::transport)?;
        if response.id != request_id {
            return Err(Error::NonceMismatch);
        }
        Ok(response.result()?)
    }

    /// Execute the RPC asynchronously.
    pub async fn call_async<T, E>(
        &self,
        rpc: impl RpcMethod,
        params: &[Value],
        send_fn: impl AsyncFn(Request) -> Result<Response, E>,
    ) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de>,
        E: std::error::Error + Send + Sync + 'static,
    {
        let method = rpc.method();
        let raw_value = if params.is_empty() {
            None
        } else {
            Some(serde_json::value::to_raw_value(params)?)
        };
        let request = self.request(&method, raw_value.as_deref());
        let request_id = request.id.clone();
        let response = send_fn(request).await.map_err(Error::transport)?;
        if response.id != request_id {
            return Err(Error::NonceMismatch);
        }
        Ok(response.result()?)
    }

    /// Forms the [`Request`].
    fn request<'a>(&self, method: &'a str, params: Option<&'a RawValue>) -> Request<'a> {
        let nonce = self.nonce.fetch_add(1, Ordering::Relaxed);
        Request {
            method,
            params,
            id: json!(nonce),
            jsonrpc: Some(JSONRPC),
        }
    }
}

// #[cfg(test)]
// mod test_auth {
//     use super::*;

//     #[test]
//     fn test_auth_user_pass_get_user_pass() {
//         let auth = Auth::UserPass("user".to_string(), "pass".to_string());
//         let result = auth.get_user_pass().expect("failed to get user pass");

//         assert_eq!(result, (Some("user".to_string()), Some("pass".to_string())));
//     }

//     #[test]
//     fn test_auth_none_get_user_pass() {
//         let auth = Auth::None;
//         let result = auth.get_user_pass().expect("failed to get user pass");

//         assert_eq!(result, (None, None));
//     }

//     #[test]
//     fn test_auth_cookie_file_get_user_pass() {
//         let temp_dir = std::env::temp_dir();
//         let cookie_path = temp_dir.join("test_auth_cookie");
//         std::fs::write(&cookie_path, "testuser:testpass").expect("failed to write cookie");

//         let auth = Auth::CookieFile(cookie_path.clone());
//         let result = auth.get_user_pass().expect("failed to get user pass");

//         assert_eq!(
//             result,
//             (Some("testuser".to_string()), Some("testpass".to_string()))
//         );

//         std::fs::remove_file(cookie_path).ok();
//     }
// }
