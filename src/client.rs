use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    str::FromStr,
};

use crate::error::Error;
use crate::jsonrpc::minreq_http::Builder;
use corepc_types::{
    bitcoin::{
        Block, BlockHash, Transaction, Txid, block::Header, consensus::deserialize, hex::FromHex,
    },
    model::{GetBlockCount, GetBlockFilter, GetBlockVerboseOne, GetRawMempool},
};
use jsonrpc::{
    Transport, serde,
    serde_json::{self, json},
};

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
    /// Get block
    pub fn get_block(&self, block_hash: &BlockHash) -> Result<Block, Error> {
        let hex_string: String = self.call("getblock", &[json!(block_hash), json!(0)])?;

        let bytes: Vec<u8> = Vec::<u8>::from_hex(&hex_string).map_err(Error::HexToBytes)?;

        let block: Block = deserialize(&bytes)
            .map_err(|e| Error::InvalidResponse(format!("failed to deserialize block: {e}")))?;

        Ok(block)
    }

    /// Get block verboseone
    pub fn get_block_verbose(&self, block_hash: &BlockHash) -> Result<GetBlockVerboseOne, Error> {
        let res: GetBlockVerboseOne = self.call("getblock", &[json!(block_hash), json!(1)])?;
        Ok(res)
    }

    /// Get best block hash
    pub fn get_best_block_hash(&self) -> Result<BlockHash, Error> {
        let res: String = self.call("getbestblockhash", &[])?;
        Ok(res.parse()?)
    }

    /// Get block count
    pub fn get_block_count(&self) -> Result<u64, Error> {
        let res: GetBlockCount = self.call("getblockcount", &[])?;
        Ok(res.0)
    }

    /// Get block hash
    pub fn get_block_hash(&self, height: u32) -> Result<BlockHash, Error> {
        let raw: serde_json::Value = self.call("getblockhash", &[json!(height)])?;

        let hash_str = match raw {
            serde_json::Value::String(s) => s,
            serde_json::Value::Object(obj) => obj
                .get("hash")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::InvalidResponse("getblockhash: missing 'hash' field".into()))?
                .to_string(),
            _ => {
                return Err(Error::InvalidResponse(
                    "getblockhash: unexpected response type".into(),
                ));
            }
        };

        BlockHash::from_str(&hash_str).map_err(Error::HexToArray)
    }

    /// Get block filter
    pub fn get_block_filter(&self, block_hash: BlockHash) -> Result<GetBlockFilter, Error> {
        let res: GetBlockFilter = self.call("getblockfilter", &[json!(block_hash)])?;
        Ok(res)
    }

    /// Get block header
    pub fn get_block_header(&self, block_hash: &BlockHash) -> Result<Header, Error> {
        let hex_string: String = self.call("getblockheader", &[json!(block_hash), json!(false)])?;

        let bytes = Vec::<u8>::from_hex(&hex_string).map_err(Error::HexToBytes)?;

        let header = deserialize(&bytes).map_err(|e| {
            Error::InvalidResponse(format!("failed to deserialize block header: {e}"))
        })?;

        Ok(header)
    }

    /// Get raw mempool
    pub fn get_raw_mempool(&self) -> Result<Vec<Txid>, Error> {
        let res: GetRawMempool = self.call("getrawmempool", &[])?;
        Ok(res.0)
    }

    /// Get raw transaction
    pub fn get_raw_transaction(&self, txid: &Txid) -> Result<Transaction, Error> {
        let hex_string: String = self.call("getrawtransaction", &[json!(txid)])?;

        let bytes = Vec::<u8>::from_hex(&hex_string).map_err(Error::HexToBytes)?;

        let transaction = deserialize(&bytes).map_err(|e| {
            Error::InvalidResponse(format!("transaction deserialization failed: {e}"))
        })?;

        Ok(transaction)
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
