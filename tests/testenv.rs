use bdk_bitcoind_client::{Auth, Client};
use bitcoin::{Address, BlockHash};
use corepc_node::client::client_sync::Auth as CorepcAuth;
use corepc_node::{exe_path, Conf, Node};
use corepc_types::bitcoin;

/// Test Environment for running integration tests.
///
/// [`TestEnv`] exposes the [`Client`] API defined by this crate to be tested against
/// a running [`corepc_node::Node`] instance.
#[derive(Debug)]
pub struct TestEnv {
    /// [`bdk_bitcoind_client::Client`]
    pub client: Client,
    /// [`corepc_node::Node`]
    pub node: Node,
    /// [`corepc_node::Client]
    pub corepc_client: corepc_node::Client,
}

impl Default for TestEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl TestEnv {
    /// Create new [`TestEnv`] with a default [configuration](Config).
    ///
    /// This will first look for the path of the `bitcoind` executable using [`corepc_node::exe_path`]
    /// before returning a new [`TestEnv`] with [`Client`] connected to it.
    ///
    /// Note that [`Node`] also exposes its own RPC [`client`](Node::client) which may help with
    /// creating different test cases, but be aware that this is different from the client we're
    /// actually testing.
    pub fn new() -> Self {
        // Enable `txindex` and `blockfilterindex` by default.
        let mut bitcoind_config = Conf::default();
        bitcoind_config.args.push("-txindex=1");
        bitcoind_config.args.push("-blockfilterindex=1");

        TestEnv::new_with_config(&bitcoind_config)
    }

    /// Create new [`TestEnv`] with a custom [configuration](Config) for the [node](Node).
    ///
    /// This will first look for the path of the `bitcoind` executable using [`corepc_node::exe_path`]
    /// before returning a new [`TestEnv`] with [`Client`] connected to it.
    ///
    /// Note that [`Node`] also exposes its own RPC [`client`](Node::client) which may help with
    /// creating different test cases, but be aware that this is different from the client we're
    /// actually testing.
    pub fn new_with_config(config: &Conf) -> Self {
        // Try to get `BITCOIN_EXE` from the environment.
        let bitcoind_exe = exe_path().unwrap();

        // Spawn a `corepc::Node` with the default configuration.
        let node = Node::with_conf(bitcoind_exe, config).unwrap();

        // Get the URL for the RPC server.
        let rpc_url = node.rpc_url();
        // Get the location for the cookie file.
        let cookie_file = &node.params.cookie_file;

        // Setup authentication and create the `bdk_client`.
        let auth = Auth::CookieFile(cookie_file.clone());
        let client = Client::with_auth(&rpc_url, auth).unwrap();

        // Setup authentication and create the `corepc_client`.
        let corepc_auth = CorepcAuth::CookieFile(cookie_file.clone());
        let corepc_client = corepc_node::Client::new_with_auth(&rpc_url, corepc_auth).unwrap();

        Self {
            client,
            node,
            corepc_client,
        }
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
            None => self.node.client.new_address()?,
        };
        Ok(self
            .node
            .client
            .generate_to_address(nblocks, &address)?
            .into_model()?
            .0)
    }
}
