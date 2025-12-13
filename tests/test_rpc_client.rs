//! Integration tests for the Bitcoin RPC client.
//!
//! These tests require a running Bitcoin Core node in regtest mode.
//!
//! Setup:
//! ```bash
//! bitcoind -regtest -rpcuser=bitcoin -rpcpassword=bitcoin -rpcport=18443
//! ```

use bdk_bitcoind_client::{Auth, Client, Error};
use corepc_types::bitcoin::{BlockHash, Txid};
use jsonrpc::serde_json::json;
use std::{path::PathBuf, str::FromStr};

/// Helper to get the test RPC URL
fn test_url() -> String {
    std::env::var("BITCOIN_RPC_URL").unwrap_or_else(|_| "http://localhost:18443".to_string())
}

/// Helper to get test credentials
fn test_auth() -> Auth {
    let user = std::env::var("BITCOIN_RPC_USER").unwrap_or_else(|_| "bitcoin".to_string());
    let pass = std::env::var("BITCOIN_RPC_PASS").unwrap_or_else(|_| "bitcoin".to_string());
    Auth::UserPass(user, pass)
}

/// Helper to create a test client
fn test_client() -> Client {
    Client::with_auth(&test_url(), test_auth()).expect("failed to create client")
}

/// Helper to mine blocks
fn mine_blocks(client: &Client, n: u64) -> Result<Vec<String>, Error> {
    let address: String = client.call("getnewaddress", &[])?;
    client.call("generatetoaddress", &[json!(n), json!(address)])
}

#[test]
#[ignore]
fn test_client_with_user_pass() {
    let client = Client::with_auth(&test_url(), test_auth()).expect("failed to create client");

    let result = client
        .get_best_block_hash()
        .expect("failed to call getblockchaininfo");

    assert_eq!(
        result.to_string().len(),
        64,
        "block hash should be 64 characters"
    );
    assert!(
        result.to_string().chars().all(|c| c.is_ascii_hexdigit()),
        "hash should only contain hex digits"
    );
}

#[test]
fn test_auth_none_returns_error() {
    let result = Client::with_auth(&test_url(), Auth::None);

    assert!(result.is_err());
    match result {
        Err(Error::MissingAuthentication) => (),
        _ => panic!("expected MissingAuthentication error"),
    }
}

#[test]
#[ignore]
fn test_invalid_credentials() {
    let client = Client::with_auth(
        &test_url(),
        Auth::UserPass("wrong".to_string(), "credentials".to_string()),
    )
    .expect("client creation should succeed");

    let result: Result<BlockHash, Error> = client.get_best_block_hash();

    assert!(result.is_err());
}

#[test]
fn test_invalid_cookie_file() {
    let cookie_path = PathBuf::from("/nonexistent/path/to/cookie");
    let result = Client::with_auth(&test_url(), Auth::CookieFile(cookie_path));

    assert!(result.is_err());
    match result {
        Err(Error::InvalidCookieFile) => (),
        Err(Error::Io(_)) => (),
        _ => panic!("expected InvalidCookieFile or Io error"),
    }
}

#[test]
#[ignore]
fn test_client_with_custom_transport() {
    use jsonrpc::http::minreq_http::Builder;

    let transport = Builder::new()
        .url(&test_url())
        .expect("invalid URL")
        .timeout(std::time::Duration::from_secs(30))
        .basic_auth("bitcoin".to_string(), Some("bitcoin".to_string()))
        .build();

    let client = Client::with_transport(transport);

    let result = client
        .get_best_block_hash()
        .expect("failed to call getblockchaininfo");

    assert_eq!(
        result.to_string().len(),
        64,
        "block hash should be 64 characters"
    );
}

#[test]
#[ignore]
fn test_get_block_count() {
    let client = test_client();

    let block_count = client.get_block_count().expect("failed to get block count");

    assert!(block_count >= 1);
}

#[test]
#[ignore]
fn test_get_block_hash() {
    let client = test_client();

    let genesis_hash = client
        .get_block_hash(0)
        .expect("failed to get genesis block hash");

    assert_eq!(genesis_hash.to_string().len(), 64);
}

#[test]
#[ignore]
fn test_get_block_hash_for_current_height() {
    let client = test_client();

    let block_count = client.get_block_count().expect("failed to get block count");

    let block_hash = client
        .get_block_hash(block_count)
        .expect("failed to get block hash");

    assert_eq!(block_hash.to_string().len(), 64);
}

#[test]
#[ignore]
fn test_get_block_hash_invalid_height() {
    let client = test_client();

    let result = client.get_block_hash(999999999);

    assert!(result.is_err());
}

#[test]
#[ignore]
fn test_get_best_block_hash() {
    let client = test_client();

    let best_hash = client
        .get_best_block_hash()
        .expect("failed to get best block hash");

    assert_eq!(best_hash.to_string().len(), 64);
}

#[test]
#[ignore]
fn test_get_best_block_hash_changes_after_mining() {
    let client = test_client();

    let hash_before = client
        .get_best_block_hash()
        .expect("failed to get best block hash");

    mine_blocks(&client, 1).expect("failed to mine block");

    let hash_after = client
        .get_best_block_hash()
        .expect("failed to get best block hash");

    assert_ne!(hash_before, hash_after);
}

#[test]
#[ignore]
fn test_get_block() {
    let client = test_client();

    let genesis_hash = client
        .get_block_hash(0)
        .expect("failed to get genesis hash");

    let block = client
        .get_block(&genesis_hash)
        .expect("failed to get block");

    assert_eq!(block.block_hash(), genesis_hash);
    assert!(!block.txdata.is_empty());
}

#[test]
#[ignore]
fn test_get_block_after_mining() {
    let client = test_client();

    let hashes = mine_blocks(&client, 1).expect("failed to mine block");
    let block_hash = BlockHash::from_str(&hashes[0]).expect("invalid hash");

    let block = client.get_block(&block_hash).expect("failed to get block");

    assert_eq!(block.block_hash(), block_hash);
    assert!(!block.txdata.is_empty());
}

#[test]
#[ignore]
fn test_get_block_invalid_hash() {
    let client = test_client();

    let invalid_hash =
        BlockHash::from_str("0000000000000000000000000000000000000000000000000000000000000000")
            .unwrap();

    let result = client.get_block(&invalid_hash);

    assert!(result.is_err());
}

#[test]
#[ignore]
fn test_get_block_header() {
    let client = test_client();

    let genesis_hash = client
        .get_block_hash(0)
        .expect("failed to get genesis hash");

    let header = client
        .get_block_header(&genesis_hash)
        .expect("failed to get block header");

    assert_eq!(header.block_hash(), genesis_hash);
}

#[test]
#[ignore]
fn test_get_block_header_has_valid_fields() {
    let client = test_client();

    let genesis_hash = client
        .get_block_hash(0)
        .expect("failed to get genesis hash");

    let header = client
        .get_block_header(&genesis_hash)
        .expect("failed to get block header");

    assert!(header.time > 0);
    assert!(header.nonce >= 1);
}

#[test]
#[ignore]
fn test_get_raw_mempool_empty() {
    let client = test_client();

    mine_blocks(&client, 1).expect("failed to mine block");

    std::thread::sleep(std::time::Duration::from_millis(100));

    let mempool = client.get_raw_mempool().expect("failed to get mempool");

    assert!(mempool.is_empty());
}

#[test]
#[ignore]
fn test_get_raw_mempool_with_transaction() {
    let client = test_client();

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
}

#[test]
#[ignore]
fn test_get_raw_transaction() {
    let client = test_client();

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
}

#[test]
#[ignore]
fn test_get_raw_transaction_invalid_txid() {
    let client = test_client();

    let fake_txid =
        Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap();

    let result = client.get_raw_transaction(&fake_txid);

    assert!(result.is_err());
}

#[test]
#[ignore]
fn test_get_block_filter() {
    let client = test_client();

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
}
