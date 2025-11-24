use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use crate::error::Error;
use crate::jsonrpc::minreq_http::Builder;
use corepc_types::bitcoin::BlockHash;
use jsonrpc::{serde, serde_json, Transport};

/// client authentication methods
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Auth {
    None,
    UserPass(String, String),
    CookieFile(PathBuf),
}

impl Auth {
    /// Convert into the arguments that jsonrpc::Client needs.
    pub fn get_user_pass(self) -> Result<(Option<String>, Option<String>), Error> {
        match self {
            Auth::None => Ok((None, None)),
            Auth::UserPass(u, p) => Ok((Some(u), Some(p))),
            Auth::CookieFile(path) => {
                let line = BufReader::new(File::open(path)?)
                    .lines()
                    .next()
                    .ok_or(Error::InvalidCookieFile)??;
                let colon = line.find(':').ok_or(Error::InvalidCookieFile)?;
                Ok((Some(line[..colon].into()), Some(line[colon + 1..].into())))
            }
        }
    }
}

// RPC Client.
#[derive(Debug)]
pub struct Client {
    /// The inner JSON-RPC client.
    inner: jsonrpc::Client,
}

impl Client {
    /// Creates a client to a bitcoind JSON-RPC server.
    ///
    /// Requires authentication via username/password or cookie file.
    /// For connections without authentication, use `with_transport` instead.
    pub fn with_auth(url: &str, auth: Auth) -> Result<Self, Error> {
        if matches!(auth, Auth::None) {
            return Err(Error::MissingAuthentication);
        }

        let mut builder = Builder::new()
            .url(url)
            .map_err(|e| Error::InvalidResponse(format!("Invalid URL: {e}")))?
            .timeout(std::time::Duration::from_secs(60));

        builder = match auth {
            Auth::None => unreachable!(),
            Auth::UserPass(user, pass) => builder.basic_auth(user, Some(pass)),
            Auth::CookieFile(path) => {
                let cookie = std::fs::read_to_string(path)
                    .map_err(|_| Error::InvalidCookieFile)?
                    .trim()
                    .to_string();
                builder.cookie_auth(cookie)
            }
        };

        let transport = builder.build();

        Ok(Self {
            inner: jsonrpc::Client::with_transport(transport),
        })
    }

    /// Creates a client to a bitcoind JSON-RPC server with transport.
    pub fn with_transport<T>(transport: T) -> Self
    where
        T: Transport,
    {
        Self {
            inner: jsonrpc::Client::with_transport(transport),
        }
    }

    /// Calls the RPC `method` with a given `args` list.
    pub fn call<T>(&self, method: &str, args: &[serde_json::Value]) -> Result<T, Error>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let raw = serde_json::value::to_raw_value(args)?;
        let request = self.inner.build_request(method, Some(&*raw));
        let resp = self.inner.send_request(request)?;

        Ok(resp.result()?)
    }
}

// `bitcoind` RPC methods
impl Client {
    /// Get best block hash.
    pub fn get_best_block_hash(&self) -> Result<BlockHash, Error> {
        let res: String = self.call("getbestblockhash", &[])?;
        Ok(res.parse()?)
    }
}

#[cfg(test)]
mod test_auth {
    use super::*;

    #[test]
    fn test_auth_user_pass_get_user_pass() {
        let auth = Auth::UserPass("user".to_string(), "pass".to_string());
        let result = auth.get_user_pass().expect("failed to get user pass");

        assert_eq!(result, (Some("user".to_string()), Some("pass".to_string())));
    }

    #[test]
    fn test_auth_none_get_user_pass() {
        let auth = Auth::None;
        let result = auth.get_user_pass().expect("failed to get user pass");

        assert_eq!(result, (None, None));
    }

    #[test]
    fn test_auth_cookie_file_get_user_pass() {
        let temp_dir = std::env::temp_dir();
        let cookie_path = temp_dir.join("test_auth_cookie");
        std::fs::write(&cookie_path, "testuser:testpass").expect("failed to write cookie");

        let auth = Auth::CookieFile(cookie_path.clone());
        let result = auth.get_user_pass().expect("failed to get user pass");

        assert_eq!(
            result,
            (Some("testuser".to_string()), Some("testpass".to_string()))
        );

        std::fs::remove_file(cookie_path).ok();
    }
}
