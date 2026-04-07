# gitserver

Smart HTTP Git server for local testing, with no `git` runtime dependency.

`gitserver` is based on the upstream [ggueret/git-server](https://github.com/ggueret/git-server) project. It serves bare Git repositories over HTTP for `git clone` and `git fetch`, without requiring the `git` binary at runtime. It is built with [gitoxide](https://github.com/GitoxideLabs/gitoxide) for native Git operations and [Axum](https://github.com/tokio-rs/axum) / [Tokio](https://tokio.rs) for asynchronous HTTP.

## Origin

- Upstream project: [`ggueret/git-server`](https://github.com/ggueret/git-server)

## License

- Primary repository license: MPL-2.0. See `LICENSE`.
- This repository includes material based on the upstream project `ggueret/git-server`; the original MIT license text is preserved in `LICENSE-UPSTREAM-MIT` and in each crate's `license/UPSTREAM-LICENSE`.
- Files created in this repository use MPL-2.0 notices where applicable. Preserved upstream notices and license files continue to apply to inherited upstream material.

## Features

- **Single binary, no git required** -- all Git operations are handled natively, no runtime dependencies
- **Multi-repository** -- serves all bare repos under a root directory with configurable scan depth
- **JSON API** -- repository listing endpoint for programmatic discovery
- **Structured logging** -- text or JSON log output via tracing

## Quick start

```sh
cargo install --path crates/gitserver

# Serve all bare repos under ./repos
gitserver ./repos

# Clone from the server
git clone http://127.0.0.1:3000/my-project.git
```

## Usage

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
| GET | `/` | JSON array of discovered repositories |
| GET | `/{repo}/info/refs?service=git-upload-pack` | Git ref advertisement |
| GET | `/{repo}/info/refs?service=git-receive-pack` | Git receive-pack advertisement |
| POST | `/{repo}/git-upload-pack` | Git pack negotiation |
| POST | `/{repo}/git-receive-pack` | Git push handling |

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
- **gitserver-http** -- Axum HTTP routing, handlers, error responses
- **gitserver** -- CLI binary, tracing setup, server assembly
- **gitserver-bench** -- Performance benchmarks

## Documentation

Full documentation is available in [English](docs/en/index.md) and [Chinese](docs/zh/index.md):

- [Installation Guide](docs/en/installation.md)
- [Usage Guide](docs/en/usage.md)
- [API Reference](docs/en/api.md)
- [Architecture](docs/en/architecture.md)
- [Library Usage](docs/en/library.md)

## Building from source

```sh
git clone https://github.com/WJQSERVER/git-server.git
cd git-server
cargo build --release
```

The binary is at `target/release/gitserver`.

## Running tests

```sh
cargo test --workspace
```

The test suite includes unit tests, integration tests (`git clone`/`git fetch` against a running server), and load tests (concurrent clones).
