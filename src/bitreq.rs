// SPDX-License-Identifier: MIT OR Apache-2.0

//! Bitcoin Core RPC client backed by the [`bitreq`] HTTP transport.
//!
//! [`bitreq`]: https://docs.rs/jsonrpc/latest/jsonrpc/http/bitreq_http/index.html

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use corepc_types::{
    bitcoin::{
        Block, BlockHash, Transaction, Txid,
        block::Header,
        consensus::encode::{deserialize_hex, serialize_hex},
    },
    model, v30,
};
use jsonrpc::{
    Transport, bitreq_http, serde,
    serde_json::{Value, json},
};

use crate::{Error, Rpc};

#[cfg(all(feature = "28_0", not(feature = "29_0")))]
pub mod v28;

/// Authentication methods for the Bitcoin Core JSON-RPC server.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum Auth {
    /// Username and password authentication (RPC user/pass).
    UserPass(String, String),
    /// Authentication via a cookie file.
    CookieFile(PathBuf),
}

impl Auth {
    /// Converts this `Auth` into an optional username and password pair.
    ///
    /// # Errors
    ///
    /// Returns an error if the `CookieFile` cannot be read or is invalid.
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

/// Bitcoin Core RPC client backed by the `bitreq` HTTP transport.
pub struct Client {
    inner: crate::Client,
    transport: Box<dyn Transport>,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("inner", &self.inner)
            .finish_non_exhaustive()
    }
}

impl Client {
    /// Creates a client connected to a Bitcoin Core RPC server with authentication and timeout.
    ///
    /// # Errors
    ///
    /// Returns an error if the URL is invalid or the cookie file cannot be read.
    pub fn with_auth_timeout(
        url: &str,
        auth: Auth,
        timeout: core::time::Duration,
    ) -> Result<Self, Error> {
        let mut builder = bitreq_http::Builder::new()
            .url(url)
            .map_err(|e| Error::InvalidUrl(format!("{e}")))?
            .timeout(timeout);

        let (user, pass) = auth.get_user_pass()?;
        if let Some(username) = user {
            builder = builder.basic_auth(username, pass);
        }

        Ok(Self {
            inner: crate::Client::new(),
            transport: Box::new(builder.build()),
        })
    }

    /// Creates a client using a custom transport.
    ///
    /// Useful when you need manual control over TLS, proxies, or timeouts beyond
    /// what [`with_auth_timeout`](Self::with_auth_timeout) provides.
    pub fn with_transport<T>(transport: T) -> Self
    where
        T: Transport + 'static,
    {
        Self {
            inner: crate::Client::new(),
            transport: Box::new(transport),
        }
    }

    /// Executes an RPC call through the configured transport.
    fn call<T>(&self, rpc: Rpc, params: &[Value]) -> Result<T, Error>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let method = rpc.to_string();
        self.inner
            .call(&method, params, |req| self.transport.send_request(req))
    }
}

/// `bitcoind` RPC methods.
impl Client {
    /// Retrieves the raw block data for a given block hash (verbosity 0).
    ///
    /// # Arguments
    ///
    /// * `block_hash`: The hash of the block to retrieve.
    ///
    /// # Returns
    ///
    /// The deserialized `Block` struct.
    pub fn get_block(&self, block_hash: &BlockHash) -> Result<Block, Error> {
        self.call::<String>(Rpc::GetBlock, &[json!(block_hash), json!(0)])
            .and_then(|block_hex| deserialize_hex(&block_hex).map_err(Error::DecodeHex))
    }

    /// Retrieves the hash of the best chain's block.
    ///
    /// # Returns
    ///
    /// The `BlockHash` of the chain tip.
    pub fn get_best_block_hash(&self) -> Result<BlockHash, Error> {
        self.call::<String>(Rpc::GetBestBlockHash, &[])
            .and_then(|blockhash_hex| blockhash_hex.parse().map_err(Error::HexToArray))
    }

    /// Retrieves the number of blocks in the longest chain.
    ///
    /// # Returns
    ///
    /// The block count as a `u32`.
    pub fn get_block_count(&self) -> Result<u32, Error> {
        self.call::<v30::GetBlockCount>(Rpc::GetBlockCount, &[])?
            .0
            .try_into()
            .map_err(Error::TryFromInt)
    }

    /// Retrieves the [`BlockHash`] of the block at `height`.
    ///
    /// # Arguments
    ///
    /// * `height`: The block height.
    ///
    /// # Returns
    ///
    /// The [`BlockHash`] of the block at `height`.
    pub fn get_block_hash(&self, height: u32) -> Result<BlockHash, Error> {
        self.call::<String>(Rpc::GetBlockHash, &[json!(height)])
            .and_then(|blockhash_hex| blockhash_hex.parse().map_err(Error::HexToArray))
    }

    /// Retrieves the Compact Block Filter (BIP-0158) with type `basic` for a block.
    ///
    /// # Arguments
    ///
    /// * `block_hash`: The hash of the block whose filter is requested.
    ///
    /// # Returns
    ///
    /// The `GetBlockFilter` structure containing the filter data for the block.
    pub fn get_block_filter(&self, block_hash: &BlockHash) -> Result<model::GetBlockFilter, Error> {
        let block_filter: v30::GetBlockFilter =
            self.call(Rpc::GetBlockFilter, &[json!(block_hash)])?;
        block_filter.into_model().map_err(Error::model)
    }

    /// Retrieves the `Header` for a block given its `BlockHash`.
    ///
    /// # Arguments
    ///
    /// * `block_hash`: The hash of the block whose header is requested.
    ///
    /// # Returns
    ///
    /// The deserialized `Header` struct.
    pub fn get_block_header(&self, block_hash: &BlockHash) -> Result<Header, Error> {
        self.call::<String>(Rpc::GetBlockHeader, &[json!(block_hash), json!(false)])
            .and_then(|header_hex: String| deserialize_hex(&header_hex).map_err(Error::DecodeHex))
    }

    /// Retrieves the `Txid`s for all transactions in the mempool.
    ///
    /// # Returns
    ///
    /// A vector of `Txid`s in the raw mempool.
    pub fn get_raw_mempool(&self) -> Result<Vec<Txid>, Error> {
        self.call::<model::GetRawMempool>(Rpc::GetRawMempool, &[])
            .map(|txids| txids.0)
    }

    /// Retrieves the raw transaction data for a given transaction ID.
    ///
    /// # Arguments
    ///
    /// * `txid`: The transaction ID to retrieve.
    ///
    /// # Returns
    ///
    /// The deserialized `Transaction` struct.
    pub fn get_raw_transaction(&self, txid: &Txid) -> Result<Transaction, Error> {
        self.call::<String>(Rpc::GetRawTransaction, &[json!(txid)])
            .and_then(|tx_hex| deserialize_hex(&tx_hex).map_err(Error::DecodeHex))
    }

    /// Submits a raw transaction to the network.
    ///
    /// # Arguments
    ///
    /// * `tx`: The transaction to broadcast.
    ///
    /// # Returns
    ///
    /// The transaction ID (`Txid`) of the broadcasted transaction.
    pub fn send_raw_transaction(&self, tx: &Transaction) -> Result<Txid, Error> {
        let hex_tx = serialize_hex(tx);
        let txid: Txid = self.call(Rpc::SendRawTransaction, &[json!(hex_tx)])?;
        Ok(txid)
    }
}

#[cfg(feature = "29_0")]
use corepc_types::model::{GetBlockHeaderVerbose, GetBlockVerboseOne};

#[cfg(feature = "29_0")]
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
            self.call(Rpc::GetBlockHeader, &[json!(block_hash)])?;
        header_info.into_model().map_err(Error::model)
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
            self.call(Rpc::GetBlock, &[json!(block_hash), json!(1)])?;
        block_info.into_model().map_err(Error::model)
    }

    /// Retrieves information about the blockchain state.
    ///
    /// # Returns
    ///
    /// State information as a `GetBlockchainInfo` struct.
    pub fn get_blockchain_info(&self) -> Result<model::GetBlockchainInfo, Error> {
        let info: v30::GetBlockchainInfo = self.call(Rpc::GetBlockchainInfo, &[])?;
        info.into_model().map_err(Error::model)
    }
}

#[cfg(test)]
mod tests {
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
        let cookie_path = PathBuf::from("/nonexistent/path/to/cookie");
        let auth = Auth::CookieFile(cookie_path);
        let result = auth.get_user_pass();
        assert!(matches!(result, Err(Error::Io(_))));
    }
}
