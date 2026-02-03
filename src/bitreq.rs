//! [`crate::bitreq::Client`]

use alloc::string::{String, ToString};
use core::time::Duration;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use bitcoin::BlockHash;
use corepc_types::bitcoin;
use corepc_types::model::GetBlockchainInfo;
use corepc_types::v29;
use jsonrpc::bitreq_http::BitreqHttpTransport;
use jsonrpc::Transport;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{Error, Rpc};

/// Auth
#[derive(Clone, Debug)]
pub enum Auth {
    /// user pass
    UserPass(String, String),
    /// cookie file
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
            Auth::UserPass(user, pass) => Ok((Some(user), Some(pass))),
            Auth::CookieFile(path) => {
                let line = BufReader::new(File::open(path)?)
                    .lines()
                    .next()
                    .ok_or(Error::InvalidCookieFile)??;
                let colon = line.find(':').ok_or(Error::InvalidCookieFile)?;
                let user = line[..colon].to_string();
                let pass = line[colon + 1..].to_string();

                Ok((Some(user), Some(pass)))
            }
        }
    }
}

/// Client
#[derive(Debug)]
pub struct Client {
    inner: crate::Client,
    tp: BitreqHttpTransport,
}

impl Client {
    /// Create a new blocking client using `bitreq`.
    pub fn new(url: &str, timeout: Duration, auth: Auth) -> Result<Self, Error> {
        let (user, pass) = auth.get_user_pass()?;
        let user = user.unwrap_or_default();
        Ok(Self {
            inner: crate::Client::new(),
            tp: BitreqHttpTransport::builder()
                .url(url)
                .unwrap()
                .timeout(timeout)
                .basic_auth(user, pass)
                .build(),
        })
    }

    /// Execute the RPC
    fn call<T>(&self, rpc: Rpc, params: &[Value]) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.inner
            .call(rpc, params, |request| self.tp.send_request(request))
    }

    /// `getblockcount`
    pub fn get_block_count(&self) -> Result<u32, Error> {
        Ok(self
            .call::<v29::GetBlockCount>(Rpc::GetBlockCount, &[])?
            .into_model()
            .0
            .try_into()?)
    }

    /// `getblockchaininfo`
    pub fn get_blockchain_info(&self) -> Result<GetBlockchainInfo, Error> {
        self.call::<v29::GetBlockchainInfo>(Rpc::GetBlockchainInfo, &[])?
            .into_model()
            .map_err(Error::model)
    }

    /// `getblockhash`
    pub fn get_block_hash(&self, height: u32) -> Result<BlockHash, Error> {
        Ok(self
            .call::<v29::GetBlockHash>(Rpc::GetBlockHash, &[json!(height)])?
            .block_hash()?)
    }
}
