# bdk-bitcoind-client

A minimal, experimental Bitcoin Core RPC client designed specifically for the Bitcoin Dev Kit (BDK). Unlike generic RPC wrappers, 
`bdk-bitcoind-client` focuses on high-performance data emission and strict type safety, with built-in support for multiple Bitcoin 
Core versions (v28.0 through v30.0+).

### Features 

- _Version Pinning_: Explicit support for different Bitcoin Core RPC schemas via feature flags (28_0, 29_0, 30_0).
- _Minimal Dependencies_: Uses bitreq_http for a lightweight HTTP transport by default.
- _Wallet-Agnostic_: Focused on blockchain data emission (blocks, headers, mempool) rather than wallet management.
- _Robust Error Handling_: Specific error variants for RPC failures, deserialization issues, and transport timeouts.

### Installation

Add this to your `Cargo.toml`:
```
// For the latest Bitcoin Core (v30.0+) 
bdk-bitcoind-client = { version = "0.1.0" }

// OR for older nodes (e.g., v28.x) 
bdk-bitcoind-client = { version = "0.1.0", default-features = false, features = ["28_0"] }
```

### Quick Start

```
use bdk_bitcoind_client::{Auth, Client};
use std::path::PathBuf;
fn main() -> anyhow::Result<()> {
    // 1. Setup authentication (Cookie file is recommended for security)
    let auth = Auth::CookieFile(PathBuf::from("/path/to/regtest/.cookie"));

    // 2. Initialize the client
    let client = Client::with_auth("http://127.0.0.1:18443", auth)?;

    // 3. Query the blockchain
    let block_count = client.get_block_count()?;
    let best_hash = client.get_block_hash(block_count)?;
    
    // 4. Get verbose headers (handles schema differences automatically)
    let header = client.get_block_header_verbose(&best_hash)?;
    
    println!("Chain tip: {} at height {}", header.hash, header.height);

    Ok(())
}
```

### Version Compatibility

Bitcoin Core often changes its JSON-RPC response fields (e.g., adding the target field in `v29/v30`). This client manages these differences through compile-time features.

| Feature           | Bitcoin Core Version  | Notes                                        |
| ----------------- | --------------------- | -------------------------------------------- | 
| 30_0 (default)    | v30.x and newer       | Supports latest target and difficulty fields.|
| 29_0              | v29.x                 | Aligned with v29 schema. |
| 28_0v             | 28.x and older        | Omits newer fields |


### Development and Testing
To run tests against a specific Bitcoin Core version, use the corresponding feature flag:

```
cargo test --no-default-features --features 28_0
```

### Minimum Supported Rust Version (MSRV)

The library maintain a MSRV of 1.85.0.

## Just

This project has a [`justfile`](/justfile) for easy command running. You must have [`just`](https://github.com/casey/just) installed.

To see a list of available recipes: `just`

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
