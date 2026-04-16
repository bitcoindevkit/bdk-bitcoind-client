//! Integration tests for the `bdk_bitcoind_client` [`Client`].
//!
//! These tests require a running Bitcoin Core node in regtest mode. To setup refer to [`corepc_node`].

use bdk_bitcoind_client::{Auth, Client, Error};
use corepc_node::Conf;
use corepc_types::bitcoin::{Amount, BlockHash, Txid};
use std::str::FromStr;

mod testenv;

use testenv::TestEnv;

#[test]
fn test_invalid_credentials() {
    let env = TestEnv::new();

    let client = Client::with_auth(
        &env.node.rpc_url(),
        Auth::UserPass("wrong".to_string(), "credentials".to_string()),
    )
    .expect("client creation should succeed");

    let result: Result<BlockHash, Error> = client.get_best_block_hash();

    assert!(result.is_err());
}

#[test]
fn test_client_with_custom_transport() {
    use jsonrpc::http::bitreq_http::Builder;

    let env = TestEnv::new();

    let rpc_url = env.node.rpc_url();
    let cookie = env
        .node
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

    let _result = client
        .get_best_block_hash()
        .expect("failed to call getbestblockhash");
}

#[test]
fn test_get_block_count() {
    let env = TestEnv::new();

    let block_count = env
        .client
        .get_block_count()
        .expect("failed to get block count");

    assert_eq!(block_count, 0);
}

#[test]
fn test_get_block_hash() {
    let env = TestEnv::new();

    let _genesis_hash = env
        .client
        .get_block_hash(0)
        .expect("failed to get genesis block hash");
}

#[test]
fn test_get_block_hash_for_current_height() {
    let env = TestEnv::new();

    let block_count = env
        .client
        .get_block_count()
        .expect("failed to get block count");

    let _block_hash = env
        .client
        .get_block_hash(block_count)
        .expect("failed to get block hash");
}

#[test]
fn test_get_block_hash_invalid_height() {
    let env = TestEnv::new();

    let result = env.client.get_block_hash(999_999_999);

    assert!(result.is_err());
}

#[test]
fn test_get_best_block_hash() {
    let env = TestEnv::new();

    let best_block_hash = env
        .client
        .get_best_block_hash()
        .expect("failed to get best block hash");

    let block_count = env
        .client
        .get_block_count()
        .expect("failed to get block count");
    let block_hash = env
        .client
        .get_block_hash(block_count)
        .expect("failed to get block hash");

    assert_eq!(best_block_hash, block_hash);
}

#[test]
fn test_get_block() {
    let env = TestEnv::new();

    let genesis_hash = env
        .client
        .get_block_hash(0)
        .expect("failed to get genesis hash");

    let block = env
        .client
        .get_block(&genesis_hash)
        .expect("failed to get block");

    assert_eq!(block.block_hash(), genesis_hash);
    assert!(!block.txdata.is_empty());
}

#[test]
fn test_get_block_after_mining() {
    let env = TestEnv::new();

    let hashes = env.mine_blocks(1, None).expect("failed to mine block");
    let block_hash = hashes[0];

    let block = env
        .client
        .get_block(&block_hash)
        .expect("failed to get block");

    assert_eq!(block.block_hash(), block_hash);
    assert!(!block.txdata.is_empty());
}

#[test]
fn test_get_block_verbose() {
    let env = TestEnv::new();

    let hashes = env.mine_blocks(1, None).expect("failed to mine block");
    let block_hash = hashes[0];

    let get_block_verbose_one = env
        .client
        .get_block_verbose(&block_hash)
        .expect("failed to get block verbose 1");

    assert_eq!(get_block_verbose_one.hash, block_hash);
    assert_eq!(get_block_verbose_one.confirmations, 1);
}

#[test]
fn test_get_block_invalid_hash() {
    let env = TestEnv::new();

    let invalid_hash =
        BlockHash::from_str("0000000000000000000000000000000000000000000000000000000000000000")
            .unwrap();

    let result = env.client.get_block(&invalid_hash);

    assert!(result.is_err());
}

#[test]
fn test_get_block_header() {
    let env = TestEnv::new();

    let genesis_hash = env
        .client
        .get_block_hash(0)
        .expect("failed to get genesis hash");

    let header = env
        .client
        .get_block_header(&genesis_hash)
        .expect("failed to get block header");

    assert_eq!(header.block_hash(), genesis_hash);
}

#[test]
fn test_get_block_header_verbose() {
    let env = TestEnv::new();

    let genesis_hash = env
        .client
        .get_block_hash(0)
        .expect("failed to get genesis hash");

    let header = env
        .client
        .get_block_header_verbose(&genesis_hash)
        .expect("failed to get block header verbose");

    assert_eq!(header.hash, genesis_hash);
}

#[test]
fn test_get_raw_mempool_empty() {
    let env = TestEnv::new();

    let _hashes = env.mine_blocks(1, None).expect("failed to mine block");

    std::thread::sleep(std::time::Duration::from_millis(100));

    let mempool = env.client.get_raw_mempool().expect("failed to get mempool");

    assert!(mempool.is_empty());
}

#[test]
fn test_get_raw_mempool_with_transaction() {
    let env = TestEnv::new();

    let _hashes = env.mine_blocks(101, None).expect("failed to mine block");

    let address = env.node.client.new_address().unwrap();
    let txid = env
        .node
        .client
        .send_to_address(&address, Amount::from_btc(0.001).unwrap())
        .expect("failed to send to address")
        .into_model()
        .unwrap()
        .txid;

    let mempool = env.client.get_raw_mempool().expect("failed to get mempool");
    assert!(mempool.contains(&txid));
}

#[test]
fn test_get_raw_transaction() {
    let env = TestEnv::new();

    let _hashes = env.mine_blocks(1, None).expect("failed to mine block");

    let best_hash = env
        .client
        .get_best_block_hash()
        .expect("failed to get best block hash");

    let block = env
        .client
        .get_block(&best_hash)
        .expect("failed to get block");

    let expected_tx = &block.txdata[0];
    let txid = expected_tx.compute_txid();

    let result_tx = env
        .client
        .get_raw_transaction(&txid)
        .expect("failed to get raw transaction");

    assert_eq!(result_tx, *expected_tx);
    assert_eq!(result_tx.compute_txid(), txid);
}

#[test]
fn test_get_raw_transaction_invalid_txid() {
    let env = TestEnv::new();

    let fake_txid =
        Txid::from_str("0000000000000000000000000000000000000000000000000000000000000000").unwrap();

    let result = env.client.get_raw_transaction(&fake_txid);

    assert!(result.is_err());
}

#[test]
fn test_get_block_filter() {
    let env = TestEnv::new();

    let genesis_hash = env
        .client
        .get_block_hash(0)
        .expect("failed to get genesis hash");

    let result = env
        .client
        .get_block_filter(&genesis_hash)
        .expect("failed to get block filter");

    assert!(!result.filter.is_empty());
}

#[test]
fn test_get_prune_height() {
    // Spawn an unpruned node.
    let env_unpruned = TestEnv::new();

    // Assert that `getblockchaininfo.pruned` is `None`.
    let unpruned_res = env_unpruned.client.get_prune_height().unwrap();
    assert_eq!(unpruned_res, None);

    // Spawn a node with manual pruning enabled.
    let mut node_config = Conf::default();
    node_config.args.push("-prune=1");
    node_config.args.push("-fastprune");
    let env_pruned = TestEnv::new_with_config(&node_config);

    // Mine 1000 blocks.
    let block_count = 1000;
    let _hashes = env_pruned.mine_blocks(block_count as usize, None);

    // Assert that `getblockchaininfo.pruned` is `Some(0)`.
    let pruned_res = env_pruned.client.get_prune_height().unwrap();
    assert_eq!(pruned_res, Some(0));

    // Prune the last 2 blocks.
    let _ = env_pruned.corepc_client.prune_blockchain(block_count - 2);

    // Assert that `getblockchaininfo.prunedheight` is > 0.
    // Note: it's not possible to assert on a specific block height since Bitcoin Core
    // prunes at the block file level (`blkXXXX.dat`), and not at block height level.
    let pruned_res = env_pruned.client.get_prune_height().unwrap();
    assert!(pruned_res > Some(0));
}
