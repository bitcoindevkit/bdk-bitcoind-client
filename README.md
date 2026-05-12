# bitcoind-client

<p>
    <!-- <a href="https://crates.io/crates/bitcoind-client"><img src="https://img.shields.io/crates/v/bitcoind-client.svg"/></a> -->
    <!-- <a href="https://docs.rs/bitcoind-client"><img src="https://img.shields.io/badge/docs.rs-bitcoind-client-orange"/></a> -->
    <a href="https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/"><img src="https://img.shields.io/badge/rustc-1.85.0%2B-orange.svg"/></a>
    <a href="https://github.com/bitcoindevkit/bitcoind-client/blob/master/LICENSE"><img src="https://img.shields.io/badge/License-MIT%2FApache--2.0-red.svg"/></a>
    <a href="https://github.com/bitcoindevkit/bitcoind-client/actions/workflows/cont_integration.yml"><img src="https://github.com/bitcoindevkit/bitcoind-client/actions/workflows/cont_integration.yml/badge.svg"></a>
</p>

A minimal `bitcoind` RPC client custom built for [BDK](https://github.com/bitcoindevkit/bdk).

## Features

- **Multiple Bitcoin Core Version Support**: implements support for multiple Bitcoin Core versions in the backend:
  - Bitcoin Core v30.0
  - Bitcoin Core v29.0
  - Bitcoin Core v28.0

- **Minimal Dependencies**: by default, the minimal `bitreq_http` is used as the HTTP transport.

- **Separation of Concerns**: focused on emitting generic data structures, such as blocks,
headers and mempool. Interpreting this data is left to wallets that use this crate as a chain-source.

- **Robust Error Handling**: implements specifc error variants for RPC, deserialization and transport errors.

## Usage

Add this to your `Cargo.toml` manifest to use this crate with
the latest Bitcoin Core version (currently v30.0) as the backend:

```toml
bitcoind-client = { version = "0.1.0" }
```

Alternatively, add this to your `Cargo.toml` manifest to use this crate
with a specific Bitcoin Core version as the backend (v28.0 or v29.0):

```toml
# Bitcoin Core v29.0
bitcoind-client = { version = "0.1.0", default-features = false, features = ["29_0"] }

# Bitcoin Core v28.0
bitcoind-client = { version = "0.1.0", default-features = false, features = ["28_0"] }
```

## Quick Start

```rust
use bitcoind_client::{Auth, Client};
use std::path::PathBuf;
fn main() -> anyhow::Result<()> {
    // Define how to authenticate with `bitcoind` (Cookie File or User/Pass)
    let auth = Auth::CookieFile(PathBuf::from("/path/to/regtest/.cookie"));
    let auth = Auth::UserPass("user".to_string(), "pass".to_string());

    // Instantiate a JSON-RPC `Client`
    let client = Client::with_auth("http://127.0.0.1:18443", auth)?;

    // Perform blockchain queries to `bitcoind` using the `Client`
    let block_count = client.get_block_count()?;
    let best_hash = client.get_block_hash(block_count)?;
    let best_header = client.get_block_header_verbose(&best_hash)?;

    println!("Block Count: {}", block_count);
    println!("Best Block Hash: {}", best_hash);
    println!("Chain Tip: {} at height {}", best_header.hash, best_header.height);

    Ok(())
}
```

## Bitcoin Core Version Compatibility

Bitcoin Core often changes its JSON-RPC schema, such as the addition of the `target`
and `difficulty` fields in the `getmininginfo` RPC on Bitcoin Core v29.0 and newer.

This crate manages this via compile-time feature flags:

| Feature Flag     | Bitcoin Core Version | Notes                                                        |
| ---------------- | -------------------- | ------------------------------------------------------------ |
| `30_0` (default) | v30.x                |                                                              |
| `29_0`           | v29.x                | Supports `target` and `difficulty` fields on `getmininginfo` |
| `28_0`           | v28.x and older      | Omits newer fields                                           |


## Developing

This project uses [`cargo-rbmt`](https://github.com/rust-bitcoin/rust-bitcoin-maintainer-tools/tree/master/cargo-rbmt)
to manage everything related to `cargo`, such as formatting, linting, testing and CI. To install it, run:

```console
~$ cargo install cargo-rbmt
```

A `justfile` is provided for convenient command-running. You must have
[`just`](https://github.com/casey/just?tab=readme-ov-file#installation) installed.

Run `just` to see available commands:

```console
~$ just
> bitcoind-client
> An experimental `bitcoind` RPC client for BDK

Available recipes:
    build                # Build the `bitcoind-client` [alias: b]
    check                # Check code formatting, compilation, and linting [alias: c]
    check-features       # Check that all feature combinations compile [alias: cf]
    check-sigs           # Checks whether all commits in this branch are PGP-signed [alias: cs]
    doc                  # Generate documentation [alias: d]
    doc-open             # Generate and open documetation [alias: do]
    fmt                  # Format code [alias: f]
    lock                 # Regenerate `Cargo-recent.lock` and `Cargo-minimal.lock` [alias: l]
    msrv                 # Verify the library builds with the MSRV toolchain (1.85.0) [alias: m]
    pre-push             # Run pre-push suite: lock, check-sigs, fmt, check, test, and msrv [alias: p]
    test                 # Run all tests on the workspace with all features [alias: t]
    test-version VERSION # Run tests against a specific Bitcoin Core version: 30_0, 29_0, 28_0 [alias: tv]
```

## Minimum Supported Rust Version (MSRV)

This library should compile with any combination of features on Rust 1.85.0.

To build with the MSRV toolchain, copy `Cargo-minimal.lock` to `Cargo.lock`.

## License

Licensed under either of

* Apache License, Version 2.0 ([Apache 2.0](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
* MIT License ([MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
