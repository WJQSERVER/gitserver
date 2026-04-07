# Installation Guide

## Prerequisites

- **Rust toolchain**: Rust 1.85 or later (the project uses Edition 2024)
- **Git**: required for tests only, not needed at runtime

## Install from Source

```sh
git clone https://github.com/WJQSERVER/git-server.git
cd git-server
cargo install --path crates/git-server
```

After installation, the `git-server` binary will be in Cargo's bin directory (typically `~/.cargo/bin/git-server`).

## Build a Release Binary

```sh
cargo build --release
```

The compiled binary is at `target/release/git-server`.

## Use as a Library

The project is organized as a Cargo workspace with four crates:

| Crate | Description |
|-------|-------------|
| `git-server` | CLI binary entry point |
| `git-server-core` | Git protocol operations, repo discovery, path security |
| `git-server-http` | Axum HTTP routing and handlers |
| `git-server-bench` | Performance benchmarks (not published) |

Add dependencies in your `Cargo.toml`:

```toml
[dependencies]
git-server-core = { git = "https://github.com/WJQSERVER/git-server" }
git-server-http = { git = "https://github.com/WJQSERVER/git-server" }
```

## Running Tests

```sh
cargo test --all-features
```

The test suite covers unit tests, integration tests (`git clone`/`git fetch`), and load tests.

## Running Benchmarks

```sh
cargo bench -p git-server-bench
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
