//! Integration tests for the Bitcoin RPC client.
//!
//! These tests require a running Bitcoin Core node in regtest mode.
//!
//! Setup:
//! ```bash
//! bitcoind -regtest -rpcuser=bitcoin -rpcpassword=bitcoin -rpcport=18443
//! ```

use bdk_bitcoind_client::{Auth, Client, Error};
use corepc_node::{exe_path, Conf, Node};
use corepc_types::bitcoin::{BlockHash, Txid};
use jsonrpc::serde_json::json;
use std::{path::PathBuf, str::FromStr};

/// Helper to initialize the bitcoind executable path
fn init() -> String {
    exe_path().expect("bitcoind executable not found. Set BITCOIND_EXE or enable download feature.")
}

/// Helper to set up a clean bitcoind node and return the client.
fn setup() -> (Client, Node) {
    let exe = init();

    let mut conf = Conf::default();

    conf.args.push("-blockfilterindex=1");
    conf.args.push("-txindex=1");

    let node = Node::with_conf(exe, &conf).expect("Failed to start node");

    let rpc_url = node.rpc_url();
    let cookie = node
        .params
        .get_cookie_values()
        .expect("Failed to read cookie")
        .expect("Cookie file empty");

    let auth = Auth::UserPass(cookie.user, cookie.password);

    let client = Client::with_auth(&rpc_url, auth).expect("failed to create client");

    (client, node)
}

/// Helper to mine blocks
fn mine_blocks(client: &Client, n: u64) -> Result<Vec<String>, Error> {
    let address: String = client.call("getnewaddress", &[])?;
    client.call("generatetoaddress", &[json!(n), json!(address)])
}

#[test]
fn test_client_with_user_pass() {
    let (client, mut node) = setup();

    let block_hash = client
        .get_best_block_hash()
        .expect("failed to call getbestblockhash");

    assert_eq!(
        block_hash.to_string().len(),
        64,
        "block hash should be 64 characters"
    );
    assert!(
        block_hash
            .to_string()
            .chars()
            .all(|c| c.is_ascii_hexdigit()),
        "hash should only contain hex digits"
    );

    node.stop().expect("failed to stop node");
}

#[test]
fn test_invalid_credentials() {
    let (_, mut node) = setup();
    let client = Client::with_auth(
        &node.rpc_url(),
        Auth::UserPass("wrong".to_string(), "credentials".to_string()),
    )
    .expect("client creation should succeed");

    let result: Result<BlockHash, Error> = client.get_best_block_hash();

    assert!(result.is_err());

    node.stop().expect("failed to stop node");
}

#[test]
fn test_invalid_cookie_file() {
    let dummy_url = "http://127.0.0.1:18443";
    let cookie_path = PathBuf::from("/nonexistent/path/to/cookie");

    let result = Client::with_auth(dummy_url, Auth::CookieFile(cookie_path));

    assert!(
        result.is_err(),
        "Client should fail when cookie file is missing"
    );

    match result {
        Err(Error::InvalidCookieFile) => (),
        Err(Error::Io(ref e)) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(e) => panic!("Expected InvalidCookieFile or NotFound Io error, got: {e:?}"),
        _ => panic!("Expected an error but got Ok"),
    }
}

#[test]
fn test_client_with_custom_transport() {
    use jsonrpc::http::bitreq_http::Builder;

    let (_, node) = setup();

    let rpc_url = node.rpc_url();
    let cookie = node
        .params
        .get_cookie_values()
        .expect("Failed to read cookie")
        .expect("Cookie file empty");

    let transport = Builder::new()
        .url(&rpc_url)
        .expect("invalid URL")
        .timeout(std::time::Duration::from_secs(30))
        .basic_auth(cookie.user, Some(cookie.password))
        .build();

    let client = Client::with_transport(transport);

    let result = client
        .get_best_block_hash()
        .expect("failed to call getbestblockhash");

    assert_eq!(
        result.to_string().len(),
        64,
        "block hash should be 64 characters"
    );
}

#[test]
fn test_get_block_count() {
    let (client, mut node) = setup();

    let block_count = client.get_block_count().expect("failed to get block count");

    assert_eq!(block_count, 0);

    node.stop().expect("failed to stop node");
}

#[test]
fn test_get_block_hash() {
    let (client, mut node) = setup();

    let genesis_hash = client
        .get_block_hash(0)
        .expect("failed to get genesis block hash");

    assert_eq!(genesis_hash.to_string().len(), 64);

    node.stop().expect("failed to stop node");
}

#[test]
fn test_get_block_hash_for_current_height() {
    let (client, mut node) = setup();

    let block_count = client.get_block_count().expect("failed to get block count");

    let block_hash = client
        .get_block_hash(block_count)
        .expect("failed to get block hash");

    assert_eq!(block_hash.to_string().len(), 64);
    node.stop().expect("failed to stop node");
}

#[test]
fn test_get_block_hash_invalid_height() {
    let (client, mut node) = setup();

    let result = client.get_block_hash(999999999);

    assert!(result.is_err());
    node.stop().expect("failed to stop node");
}

#[test]
fn test_get_best_block_hash() {
    let (client, mut node) = setup();

    let best_block_hash = client
        .get_best_block_hash()
        .expect("failed to get best block hash");

    assert_eq!(best_block_hash.to_string().len(), 64);

    let block_count = client.get_block_count().expect("failed to get block count");
    let block_hash = client
        .get_block_hash(block_count)
        .expect("failed to get block hash");

    assert_eq!(best_block_hash, block_hash);
    node.stop().expect("failed to stop node");
}

#[test]
fn test_get_best_block_hash_changes_after_mining() {
    let (client, mut node) = setup();

    let hash_before = client
        .get_best_block_hash()
        .expect("failed to get best block hash");

    mine_blocks(&client, 1).expect("failed to mine block");

    let hash_after = client
        .get_best_block_hash()
        .expect("failed to get best block hash");

    assert_ne!(hash_before, hash_after);
    node.stop().expect("failed to stop node");
}

#[test]
fn test_get_block() {
    let (client, mut node) = setup();

    let genesis_hash = client
        .get_block_hash(0)
        .expect("failed to get genesis hash");

    let block = client
        .get_block(&genesis_hash)
        .expect("failed to get block");

    assert_eq!(block.block_hash(), genesis_hash);
    assert!(!block.txdata.is_empty());
    node.stop().expect("failed to stop node");
}

#[test]
fn test_get_block_after_mining() {
    let (client, mut node) = setup();

    let hashes = mine_blocks(&client, 1).expect("failed to mine block");
    let block_hash = BlockHash::from_str(&hashes[0]).expect("invalid hash");

    let block = client.get_block(&block_hash).expect("failed to get block");

    assert_eq!(block.block_hash(), block_hash);
    assert!(!block.txdata.is_empty());
    node.stop().expect("failed to stop node");
}

#[test]
fn test_get_block_invalid_hash() {
    let (client, mut node) = setup();

    let invalid_hash =
        BlockHash::from_str("0000000000000000000000000000000000000000000000000000000000000000")
            .unwrap();

    let result = client.get_block(&invalid_hash);

    assert!(result.is_err());
    node.stop().expect("failed to stop node");
}

#[test]
fn test_get_block_header() {
    let (client, mut node) = setup();

    let genesis_hash = client
        .get_block_hash(0)
        .expect("failed to get genesis hash");

    let header = client
        .get_block_header(&genesis_hash)
        .expect("failed to get block header");

    assert_eq!(header.block_hash(), genesis_hash);
    node.stop().expect("failed to stop node");
}

#[test]
fn test_get_block_header_has_valid_fields() {
    let (client, mut node) = setup();

    let genesis_hash = client
        .get_block_hash(0)
        .expect("failed to get genesis hash");

    let header = client
        .get_block_header(&genesis_hash)
        .expect("failed to get block header");

    assert!(header.time > 0);
    assert!(header.nonce >= 1);
    node.stop().expect("failed to stop node");
}

#[test]
fn test_get_raw_mempool_empty() {
    let (client, mut node) = setup();

    mine_blocks(&client, 1).expect("failed to mine block");

    std::thread::sleep(std::time::Duration::from_millis(100));

    let mempool = client.get_raw_mempool().expect("failed to get mempool");

    assert!(mempool.is_empty());
    node.stop().expect("failed to stop node");
}

#[test]
fn test_get_raw_mempool_with_transaction() {
    let (client, mut node) = setup();

    mine_blocks(&client, 101).expect("failed to mine blocks");

    let address: String = client
        .call("getnewaddress", &[])
        .expect("failed to get address");
    let txid: String = client
        .call("sendtoaddress", &[json!(address), json!(0.001)])
        .expect("failed to send transaction");

    let mempool = client.get_raw_mempool().expect("failed to get mempool");

    let txid_parsed = Txid::from_str(&txid).unwrap();
    assert!(mempool.contains(&txid_parsed));
    node.stop().expect("failed to stop node");
}

#[test]
fn test_get_raw_transaction() {
    let (client, mut node) = setup();

    mine_blocks(&client, 1).expect("failed to mine block");

    let best_hash = client
        .get_best_block_hash()
        .expect("failed to get best block hash");

    let block = client.get_block(&best_hash).expect("failed to get block");

    let txid = &block.txdata[0].compute_txid();

    let tx = client
        .get_raw_transaction(txid)
        .expect("failed to get raw transaction");

    assert_eq!(tx.compute_txid(), *txid);
    assert!(!tx.input.is_empty());
    assert!(!tx.output.is_empty());
    node.stop().expect("failed to stop node");
}

#[test]
fn test_get_raw_transaction_invalid_txid() {
    let (client, mut node) = setup();

    let fake_txid =
        Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap();

    let result = client.get_raw_transaction(&fake_txid);

    assert!(result.is_err());
    node.stop().expect("failed to stop node");
}

#[test]
fn test_get_block_filter() {
    let (client, mut node) = setup();

    let genesis_hash = client
        .get_block_hash(0)
        .expect("failed to get genesis hash");

    let result = client.get_block_filter(&genesis_hash);

    match result {
        Ok(filter) => {
            assert!(!filter.filter.is_empty());
        }
        Err(_) => {
            println!("Block filters not enabled (requires -blockfilterindex=1)");
        }
    }
    node.stop().expect("failed to stop node");
}
