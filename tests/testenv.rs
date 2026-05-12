// SPDX-License-Identifier: MIT OR Apache-2.0

use bdk_bitcoind_client::bitreq::{Auth, Client};
use bitcoin::{Address, BlockHash};
use bitcoind::{BitcoinD, Conf, exe_path};
use corepc_types::bitcoin;

/// Test environment for running integration tests.
///
/// [`TestEnv`] exposes the [`Client`] API defined by this crate to be tested against
/// a running [`bitcoind::BitcoinD`] instance.
#[derive(Debug)]
pub struct TestEnv {
    /// [`bdk_bitcoind_client::bitreq::Client`]
    pub client: Client,
    /// [`bitcoind::BitcoinD`]
    pub bitcoind: BitcoinD,
}

impl TestEnv {
    /// Create new [`TestEnv`].
    ///
    /// This will first look for the path of the `bitcoind` executable using [`bitcoind::exe_path`]
    /// before returning a new [`TestEnv`] with [`Client`] connected to it.
    ///
    /// Note that [`Node`] also exposes its own RPC [`client`](Node::client) which may help with
    /// creating different test cases, but be aware that this is different from the client we're
    /// actually testing.
    pub fn setup() -> anyhow::Result<Self> {
        let exe = exe_path()?;

        let mut conf = Conf::default();
        conf.args.push("-blockfilterindex=1");
        conf.args.push("-txindex=1");

        let bitcoind = BitcoinD::with_conf(exe, &conf)?;

        let rpc_url = bitcoind.rpc_url();
        let cookie_file = &bitcoind.params.cookie_file;
        let auth = Auth::CookieFile(cookie_file.clone());
        let client = Client::with_auth_timeout(&rpc_url, auth, std::time::Duration::from_secs(15))?;

        Ok(Self { client, bitcoind })
    }

    /// Mines `nblocks` blocks to the given `address`, or an address controlled
    /// by the [`Node`] if not provided.
    pub fn mine_blocks(
        &self,
        nblocks: usize,
        address: Option<Address>,
    ) -> anyhow::Result<Vec<BlockHash>> {
        let address = match address {
            Some(addr) => addr,
            None => self.bitcoind.client.new_address()?,
        };
        Ok(self
            .bitcoind
            .client
            .generate_to_address(nblocks, &address)?
            .into_model()?
            .0)
    }
}
