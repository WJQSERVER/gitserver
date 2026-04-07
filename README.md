# gitserver

Rust implementation libraries for a Git Smart HTTP server with no runtime `git` dependency and protocol v2 support, plus an optional CLI for local testing.

`gitserver` is based on the upstream [ggueret/git-server](https://github.com/ggueret/git-server) project. Most users will interact with the `gitserver-core` and `gitserver-http` libraries, which embed Git Smart HTTP serving into an existing Rust application. The `gitserver` binary is a small wrapper around the same stack for local testing and standalone use.

## Origin

- Upstream project: [`ggueret/git-server`](https://github.com/ggueret/git-server)

## License

- Primary repository license: MPL-2.0. See `LICENSE`.
- This repository includes material based on the upstream project `ggueret/git-server`; the original MIT license text is preserved in `LICENSE-UPSTREAM-MIT` and in each crate's `license/UPSTREAM-LICENSE`.
- Files created in this repository use MPL-2.0 notices where applicable. Preserved upstream notices and license files continue to apply to inherited upstream material.

## Features

- **Library-first design** -- use `gitserver-core` and `gitserver-http` directly in an existing Axum/Tokio application
- **Native Git operations** -- clone, fetch, protocol v2, and optional receive-pack without a `git` runtime dependency
- **Repository discovery and dynamic registration** -- serve repos from filesystem scanning or register them in process
- **Optional CLI** -- run the same library stack as a standalone server for local testing and simple deployments

## Library quick start

Add dependencies:

```toml
[dependencies]
gitserver-core = { git = "https://github.com/WJQSERVER/gitserver" }
gitserver-http = { git = "https://github.com/WJQSERVER/gitserver" }
axum = "0.8"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

Embed the server into an Axum application:

```rust
use gitserver_core::discovery::RepoStore;
use gitserver_http::{SharedState, router};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let store = RepoStore::discover("./repos".into(), 3)?;
    let state = SharedState::new(store);
    let app = router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

Use the lower-level crates when you need finer control:

- `gitserver-core` handles protocol operations, repository discovery, path validation, protocol v2, and receive-pack
- `gitserver-http` provides `SharedState`, auth/policy configuration, and the Axum router/handlers
- `gitserver` is the CLI binary built on top of those crates

See [Library Usage](docs/en/library.md) for embedding, dynamic registration, auth, and low-level handler examples.

## CLI quick start

```sh
git clone https://github.com/WJQSERVER/gitserver.git gitserver
cd gitserver
cargo install --path crates/gitserver

# Serve all bare repos under ./repos
gitserver ./repos

# Clone from the server
git clone http://127.0.0.1:3000/my-project.git
```

Use the CLI when you want a standalone process. You do not need it when embedding the libraries.

## CLI usage

```
gitserver [OPTIONS] <ROOT>

Arguments:
  <ROOT>  Root directory containing bare Git repositories

Options:
  -b, --bind <ADDR>              Bind address [default: 127.0.0.1]
  -p, --port <PORT>              Port number [default: 3000]
  -l, --log-level <LEVEL>        Log level [default: info]
      --log-format <FORMAT>      Log format: text or json [default: text]
  -w, --workers <N>              Number of Tokio worker threads
  --max-depth <N>            Max directory depth for repo discovery [default: 3]
      --rescan-interval-secs <N> Periodic rescan interval in seconds [default: 30]
      --auth-basic-username <USER> Require HTTP Basic auth username
      --auth-basic-password <PASS> Require HTTP Basic auth password
      --auth-bearer-token <TOKEN>  Require Bearer auth token
      --enable-receive-pack        Enable push support over git-receive-pack
```

## API

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/healthz` | Health check endpoint |
| GET | `/` | JSON array of available repositories |
| GET | `/{repo}/info/refs?service=git-upload-pack` | Git ref advertisement |
| GET | `/{repo}/info/refs?service=git-receive-pack` | Git receive-pack advertisement, disabled by default |
| POST | `/{repo}/git-upload-pack` | Git pack negotiation |
| POST | `/{repo}/git-receive-pack` | Git push handling, disabled by default |

Repository listing response:

```json
[
  {
    "name": "my-project.git",
    "relative_path": "my-project.git",
    "description": "My project"
  }
]
```

## Architecture

The project is organized as a Cargo workspace with four crates:

- **gitserver-core** -- Git protocol operations (ref advertisement, pack generation), repository discovery, path security
- **gitserver-http** -- Axum HTTP routing, handlers, auth/policy configuration, shared state
- **gitserver** -- thin CLI binary that assembles the libraries into a standalone server
- **gitserver-bench** -- Performance benchmarks

## Documentation

Full documentation is available in [English](docs/en/index.md) and [Chinese](docs/zh/index.md):

- [Library Usage](docs/en/library.md)
- [Installation Guide](docs/en/installation.md)
- [Usage Guide](docs/en/usage.md)
- [API Reference](docs/en/api.md)
- [Architecture](docs/en/architecture.md)

## Building from source

```sh
git clone https://github.com/WJQSERVER/gitserver.git gitserver
cd gitserver
cargo build --release
```

The binary is at `target/release/gitserver`.

## Running tests

```sh
cargo test --workspace --all-features
```

The test suite includes unit tests, integration tests (`git clone`/`git fetch` against a running server), and load tests (concurrent clones).
