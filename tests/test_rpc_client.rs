//! Integration tests for the Bitcoin RPC client.
//!
//! These tests require a running Bitcoin Core node in regtest mode.
//!
//! Setup:
//! ```bash
//! bitcoind -regtest -rpcuser=bitcoin -rpcpassword=bitcoin -rpcport=18443
//! ```

// use bdk_bitcoind_client::{Auth, Client, Error};
// use corepc_types::bitcoin::BlockHash;
// use std::path::PathBuf;

// /// Helper to get the test RPC URL
// fn test_url() -> String {
//     std::env::var("BITCOIN_RPC_URL").unwrap_or_else(|_| "http://localhost:18443".to_string())
// }

// /// Helper to get test credentials
// fn test_auth() -> Auth {
//     let user = std::env::var("BITCOIN_RPC_USER").unwrap_or_else(|_| "bitcoin".to_string());
//     let pass = std::env::var("BITCOIN_RPC_PASS").unwrap_or_else(|_| "bitcoin".to_string());
//     Auth::UserPass(user, pass)
// }

// #[test]
// #[ignore]
// fn test_client_with_user_pass() {
//     let client = Client::with_auth(&test_url(), test_auth()).expect("failed to create client");

//     let result = client
//         .get_best_block_hash()
//         .expect("failed to call getblockchaininfo");

//     assert_eq!(
//         result.to_string().len(),
//         64,
//         "block hash should be 64 characters"
//     );
//     assert!(
//         result.to_string().chars().all(|c| c.is_ascii_hexdigit()),
//         "hash should only contain hex digits"
//     );
// }

// #[test]
// fn test_auth_none_returns_error() {
//     let result = Client::with_auth(&test_url(), Auth::None);

//     assert!(result.is_err());
//     match result {
//         Err(Error::MissingAuthentication) => (),
//         _ => panic!("expected MissingAuthentication error"),
//     }
// }

// #[test]
// #[ignore]
// fn test_invalid_credentials() {
//     let client = Client::with_auth(
//         &test_url(),
//         Auth::UserPass("wrong".to_string(), "credentials".to_string()),
//     )
//     .expect("client creation should succeed");

//     let result: Result<BlockHash, Error> = client.get_best_block_hash();

//     assert!(result.is_err());
// }

// #[test]
// fn test_invalid_cookie_file() {
//     let cookie_path = PathBuf::from("/nonexistent/path/to/cookie");
//     let result = Client::with_auth(&test_url(), Auth::CookieFile(cookie_path));

//     assert!(result.is_err());
//     match result {
//         Err(Error::InvalidCookieFile) => (),
//         Err(Error::Io(_)) => (),
//         _ => panic!("expected InvalidCookieFile or Io error"),
//     }
// }

// #[test]
// #[ignore]
// fn test_client_with_custom_transport() {
//     use jsonrpc::http::minreq_http::Builder;

//     let transport = Builder::new()
//         .url(&test_url())
//         .expect("invalid URL")
//         .timeout(std::time::Duration::from_secs(30))
//         .basic_auth("bitcoin".to_string(), Some("bitcoin".to_string()))
//         .build();

//     let client = Client::with_transport(transport);

//     let result = client
//         .get_best_block_hash()
//         .expect("failed to call getblockchaininfo");

//     assert_eq!(
//         result.to_string().len(),
//         64,
//         "block hash should be 64 characters"
//     );
// }
