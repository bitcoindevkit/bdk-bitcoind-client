# Default recipe
_default:
   @just --list

# ======================== #
# Aliases                  #
# ======================== #

alias b := build
alias c := check
alias d := doc
alias f := fmt
alias m := msrv
alias p := pre-push
alias t := test

# ======================== #
# Toolchains               #
# ======================== #

# Nightly toolchain
nightly := 'nightly'

# Stable toolchain
stable := 'stable'

# MSRV toolchain
msrv := '1.75.0'

# ======================== #
# Recipes                  #
# ======================== #

# Build the project
build:
   cargo +{{stable}} build --all-targets

# Check MSRV
msrv:
   cargo +{{msrv}} build --lib --no-default-features
   cargo +{{msrv}} build --lib --no-default-features --features "28_0"
   cargo +{{msrv}} build --lib --no-default-features --features "29_0"
   cargo +{{msrv}} build --lib --no-default-features --features "30_0"

# Format all code
fmt:
   cargo +{{nightly}} fmt

# Check code: formatting, compilation, linting, and commit signature
check:
   @just _verify-head
   cargo +{{nightly}} fmt --all -- --check
   cargo +{{stable}} check --all-targets --no-default-features
   cargo +{{stable}} check --all-targets --no-default-features --features "28_0"
   cargo +{{stable}} check --all-targets --no-default-features --features "29_0"
   cargo +{{stable}} check --all-targets --no-default-features --features "30_0"
   cargo +{{stable}} clippy --all-targets -- -D warnings

# Run all tests on the workspace
test:
   cargo +{{stable}} test --no-fail-fast

# Run doctests. Build and check docs
doc:
   cargo +{{stable}} test --doc
   RUSTDOCFLAGS='-D warnings' cargo +{{stable}} doc --no-deps

# Run pre-push suite: format, check, test, doc
pre-push: fmt check test doc

# Verify signed commit
_verify-head:
   @[ "$(git log --pretty='format:%G?' -1 HEAD)" = "N" ] && \
      echo "\n⚠️  Unsigned commit: BDK requires that commits be signed." || \
      true
