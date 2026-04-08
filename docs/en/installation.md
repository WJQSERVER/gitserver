# Installation Guide

## Prerequisites

- **Rust toolchain**: Rust 1.85 or later (the project uses Edition 2024)
- **Git**: required for tests only, not needed at runtime

## Install from Source

```sh
git clone https://github.com/WJQSERVER/gitserver.git gitserver
cd gitserver
cargo install --path crates/gitserver
```

After installation, the `gitserver` binary will be in Cargo's bin directory (typically `~/.cargo/bin/gitserver`).

## Build a Release Binary

```sh
cargo build --release
```

The compiled binary is at `target/release/gitserver`.

## Use as a Library

The project is organized as a Cargo workspace with four crates:

| Crate | Description |
|-------|-------------|
| `gitserver` | CLI binary entry point |
| `gitserver-core` | Git protocol operations, repo discovery, path security |
| `gitserver-http` | Axum HTTP routing and handlers |
| `gitserver-bench` | Performance benchmarks (not published) |

Add dependencies in your `Cargo.toml`:

```toml
[dependencies]
gitserver-core = { git = "https://github.com/WJQSERVER/gitserver" }
gitserver-http = { git = "https://github.com/WJQSERVER/gitserver" }
```

## Running Tests

```sh
cargo test --workspace --all-features
```

The test suite covers unit tests, integration tests (`git clone`/`git fetch`), and load tests.

## Running Benchmarks

```sh
cargo bench -p gitserver-bench
```

Benchmarks cover pack generation, ref advertisement, HTTP clone, git clone, and concurrent scenarios.

## Code Quality Checks

```sh
# Format code
cargo fmt

# Clippy linting
cargo clippy --all-targets --all-features -- -D warnings

# All-in-one check (format + clippy + tests)
make check
```
