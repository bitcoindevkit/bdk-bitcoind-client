use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use crate::error::Error;
use crate::jsonrpc::bitreq_http::Builder;
use corepc_types::{
    bitcoin::{
        block::Header, consensus::encode::deserialize_hex, Block, BlockHash, Transaction, Txid,
    },
    model::{GetBlockCount, GetBlockFilter, GetRawMempool},
    v30,
};
use jsonrpc::{
    serde,
    serde_json::{self, json},
    Transport,
};

#[cfg(feature = "28_0")]
pub mod v28;

/// Client authentication methods for the Bitcoin Core JSON-RPC server
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Auth {
    /// Username and password authentication (RPC user/pass)
    UserPass(String, String),
    /// Authentication via a cookie file
    CookieFile(PathBuf),
}

impl Auth {
    /// Converts `Auth` enum into the optional username and password strings
    /// required by JSON-RPC client transport.
    ///
    /// # Errors
    ///
    /// Returns an error if the `CookieFile` cannot be read or invalid
    pub fn get_user_pass(self) -> Result<(Option<String>, Option<String>), Error> {
        match self {
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

/// Bitcoin Core JSON-RPC Client.
///
/// A wrapper for JSON-RPC client for interacting with the `bitcoind` RPC interface.
#[derive(Debug)]
pub struct Client {
    /// The inner JSON-RPC client.
    inner: jsonrpc::Client,
}

impl Client {
    /// Creates a client connection to a bitcoind JSON-RPC server with authentication
    ///
    /// Requires authentication via username/password or cookie file.
    /// For connections without authentication, use `with_transport` instead.
    ///
    /// # Arguments
    ///
    /// * `url` - URL of the RPC server
    /// * `auth` - authentication method (`UserPass` or `CookieFile`).
    ///
    /// # Errors
    ///
    /// * Returns `Error::InvalidUrl` if the URL is invalid.
    /// * Returns errors related to reading the cookie file.
    pub fn with_auth(url: &str, auth: Auth) -> Result<Self, Error> {
        let mut builder = Builder::new()
            .url(url)
            .map_err(|e| Error::InvalidUrl(format!("{e}")))?
            .timeout(std::time::Duration::from_secs(60));

        builder = match auth {
            Auth::UserPass(user, pass) => builder.basic_auth(user, Some(pass)),
            Auth::CookieFile(path) => {
                let cookie = std::fs::read_to_string(path)
                    .map_err(|_| Error::InvalidCookieFile)?
                    .trim()
                    .to_string();
                builder.cookie_auth(cookie)
            }
        };

        Ok(Self {
            inner: jsonrpc::Client::with_transport(builder.build()),
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

    /// Calls the underlying RPC `method` with given `args` list
    ///
    /// This is the generic function used by all specific RPC methods.
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

/// `Bitcoind` RPC methods implementation for `Client`
impl Client {
    /// Retrieves the raw block data for a given block hash (verbosity 0)
    ///
    /// # Arguments
    ///
    /// * `block_hash`: The hash of the block to retrieve.
    ///
    /// # Returns
    ///
    /// The deserialized `Block` struct.
    pub fn get_block(&self, block_hash: &BlockHash) -> Result<Block, Error> {
        self.call::<String>("getblock", &[json!(block_hash), json!(0)])
            .and_then(|block_hex| deserialize_hex(&block_hex).map_err(Error::DecodeHex))
    }

    /// Retrieves the hash of the tip of the best block chain.
    ///
    /// # Returns
    ///
    /// The `BlockHash` of the chain tip.
    pub fn get_best_block_hash(&self) -> Result<BlockHash, Error> {
        self.call::<String>("getbestblockhash", &[])
            .and_then(|blockhash_hex| blockhash_hex.parse().map_err(Error::HexToArray))
    }

    /// Retrieves the number of blocks in the longest chain
    ///
    /// # Returns
    ///
    /// The block count as a `u32`
    pub fn get_block_count(&self) -> Result<u32, Error> {
        self.call::<GetBlockCount>("getblockcount", &[])?
            .0
            .try_into()
            .map_err(Error::TryFromInt)
    }

    /// Retrieves the block hash at a given height
    ///
    /// # Arguments
    ///
    /// * `height`: The block height
    ///
    /// # Returns
    ///
    /// The `BlockHash` for the given height
    pub fn get_block_hash(&self, height: u32) -> Result<BlockHash, Error> {
        self.call::<String>("getblockhash", &[json!(height)])
            .and_then(|blockhash_hex| blockhash_hex.parse().map_err(Error::HexToArray))
    }

    /// Retrieve the `basic` BIP 157 content filter for a particular block
    ///
    /// # Arguments
    ///
    /// * `block_hash`: The hash of the block whose filter is requested
    ///
    /// # Returns
    ///
    /// The `GetBlockFilter` structure containing the filter data
    pub fn get_block_filter(&self, block_hash: &BlockHash) -> Result<GetBlockFilter, Error> {
        let block_filter: v30::GetBlockFilter =
            self.call("getblockfilter", &[json!(block_hash)])?;
        block_filter.into_model().map_err(Error::GetBlockFilter)
    }

    /// Retrieves the raw block header for a given block hash.
    ///
    /// # Arguments
    ///
    /// * `block_hash`: The hash of the block whose header is requested.
    ///
    /// # Returns
    ///
    /// The deserialized `Header` struct
    pub fn get_block_header(&self, block_hash: &BlockHash) -> Result<Header, Error> {
        self.call::<String>("getblockheader", &[json!(block_hash), json!(false)])
            .and_then(|header_hex: String| deserialize_hex(&header_hex).map_err(Error::DecodeHex))
    }

    /// Retrieves the transaction IDs of all transactions currently in the mempool
    ///
    /// # Returns
    ///
    /// A vector of `Txid`s in the raw mempool
    pub fn get_raw_mempool(&self) -> Result<Vec<Txid>, Error> {
        self.call::<GetRawMempool>("getrawmempool", &[])
            .map(|txids| txids.0)
    }

    /// Retrieves the raw transaction data for a given transaction ID
    ///
    /// # Arguments
    ///
    /// * `txid`: The transaction ID to retrieve.
    ///
    /// # Returns
    ///
    /// The deserialized `Transaction` struct
    pub fn get_raw_transaction(&self, txid: &Txid) -> Result<Transaction, Error> {
        self.call::<String>("getrawtransaction", &[json!(txid)])
            .and_then(|tx_hex| deserialize_hex(&tx_hex).map_err(Error::DecodeHex))
    }
}

#[cfg(not(feature = "28_0"))]
use corepc_types::model::{GetBlockHeaderVerbose, GetBlockVerboseOne};

#[cfg(not(feature = "28_0"))]
impl Client {
    /// Retrieves the verbose JSON representation of a block header (verbosity 1).
    ///
    /// # Arguments
    ///
    /// * `block_hash`: The hash of the block to retrieve.
    ///
    /// # Returns
    ///
    /// The verbose header as a `GetBlockHeaderVerbose` struct.
    pub fn get_block_header_verbose(
        &self,
        block_hash: &BlockHash,
    ) -> Result<GetBlockHeaderVerbose, Error> {
        let header_info: v30::GetBlockHeaderVerbose =
            self.call("getblockheader", &[json!(block_hash)])?;
        header_info
            .into_model()
            .map_err(Error::GetBlockHeaderVerbose)
    }

    /// Retrieves the verbose JSON representation of a block (verbosity 1).
    ///
    /// # Arguments
    ///
    /// * `block_hash`: The hash of the block to retrieve.
    ///
    /// # Returns
    ///
    /// The verbose block data as a `GetBlockVerboseOne` struct.
    pub fn get_block_verbose(&self, block_hash: &BlockHash) -> Result<GetBlockVerboseOne, Error> {
        let block_info: v30::GetBlockVerboseOne =
            self.call("getblock", &[json!(block_hash), json!(1)])?;
        block_info.into_model().map_err(Error::GetBlockVerboseOne)
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
    #[ignore = "modifies the local filesystem"]
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

    #[test]
    fn test_auth_invalid_cookie_file() {
        let dummy_url = "http://127.0.0.1:18443";
        let cookie_path = PathBuf::from("/nonexistent/path/to/cookie");

        let result = Client::with_auth(dummy_url, Auth::CookieFile(cookie_path));

        assert!(matches!(result, Err(Error::InvalidCookieFile)));
    }
}
