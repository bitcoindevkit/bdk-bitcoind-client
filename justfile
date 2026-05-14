alias b := build
alias c := check
alias cf := check-features
alias cs := check-sigs
alias d := doc
alias do := doc-open
alias f := fmt
alias l := lock
alias m := msrv
alias p := pre-push
alias t := test
alias tv := test-version

_default:
    @echo "> bitcoind-client"
    @echo "> An experimental \`bitcoind\` RPC client for BDK"
    @echo ""
    @just --list

[doc: "Build the `bitcoind-client`"]
build:
    cargo build

[doc: "Check code formatting, compilation, and linting"]
check:
    cargo rbmt fmt --check
    cargo rbmt lint
    cargo rbmt docs

[doc: "Check that all feature combinations compile"]
check-features:
    cargo rbmt test --toolchain stable --lock-file recent

[doc: "Check if all commits in this branch are PGP-signed"]
check-sigs:
    bash contrib/check-signatures.sh

[doc: "Generate documentation"]
doc:
    cargo rbmt docs

[doc: "Generate and open documetation"]
doc-open:
    cargo rbmt docs --open

[doc: "Format code"]
fmt:
    cargo rbmt fmt

[doc: "Regenerate `Cargo-recent.lock` and `Cargo-minimal.lock`"]
lock:
    cargo rbmt lock

[doc: "Verify the library builds with the MSRV toolchain (1.85.0)"]
msrv:
    cargo rbmt test --toolchain msrv --lock-file minimal

[doc: "Run all tests on the workspace with all features"]
test:
    cargo rbmt test --toolchain stable --lock-file recent

# TODO: update this when https://github.com/rust-bitcoin/rust-bitcoin-maintainer-tools/issues/113 is fixed
[doc: "Run tests against a specific Bitcoin Core version: 30_0, 29_0, 28_0"]
test-version VERSION:
    cargo test --no-default-features --features {{VERSION}}

[doc: "Run pre-push suite: lock, check-sigs, fmt, check, test, and msrv"]
pre-push: lock check-sigs fmt check test msrv
