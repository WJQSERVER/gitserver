# Usage Guide

## Basic Usage

```sh
gitserver [OPTIONS] <ROOT>
```

`<ROOT>` is the root directory containing bare Git repositories.

## Command-Line Arguments

### Positional Arguments

| Argument | Description |
|----------|-------------|
| `<ROOT>` | Root directory containing bare Git repositories |

### Options

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--bind <ADDR>` | `-b` | `127.0.0.1` | Bind address |
| `--port <PORT>` | `-p` | `3000` | Port number |
| `--log-level <LEVEL>` | `-l` | `info` | Log level (trace/debug/info/warn/error) |
| `--log-format <FORMAT>` | - | `text` | Log format: `text` or `json` |
| `--workers <N>` | `-w` | auto | Number of Tokio worker threads |
| `--max-depth <N>` | - | `3` | Max directory depth for repo discovery |
| `--rescan-interval-secs <N>` | - | `30` | Interval in seconds for automatic repo list rescan |
| `--auth-basic-username <USER>` | - | - | HTTP Basic auth username (requires `--auth-basic-password`) |
| `--auth-basic-password <PASS>` | - | - | HTTP Basic auth password (requires `--auth-basic-username`) |
| `--auth-bearer-token <TOKEN>` | - | - | Bearer auth token |
| `--enable-receive-pack` | - | `false` | Enable git-receive-pack to allow push operations |
| `--request-timeout-secs <N>` | - | `300` | Timeout in seconds for upload-pack and receive-pack requests |
| `--max-pack-bytes <BYTES>` | - | - | Maximum uncompressed upload-pack bytes allowed before rejecting the request |

## Examples

### Start the Server

```sh
# With default settings
gitserver ./repos

# Custom address and port
gitserver -b 0.0.0.0 -p 8080 ./repos

# JSON log format
gitserver --log-format json ./repos

# Limit scan depth to 1
gitserver --max-depth 1 ./repos
```

### Enable Authentication

```sh
# Basic auth
gitserver --auth-basic-username admin --auth-basic-password secret ./repos

# Bearer token auth
gitserver --auth-bearer-token my-secret-token ./repos
```

### Enable Push Support

```sh
gitserver --enable-receive-pack ./repos
```

> Note: push operations are disabled by default. When enabled, the server only accepts fast-forward updates and does not allow ref deletion or overwriting existing tags.

### Configure Limits

```sh
# Reject clone/fetch responses that would stream more than 512 MiB of pack data
gitserver --max-pack-bytes $((512 * 1024 * 1024)) ./repos

# Fail stalled or long-running fetch/push requests after 2 minutes
gitserver --request-timeout-secs 120 ./repos
```

### Clone and Fetch

```sh
# Clone a repository
git clone http://127.0.0.1:3000/my-project.git

# Fetch updates from remote
git fetch

# Clone with authentication
git clone http://admin:secret@127.0.0.1:3000/my-project.git
```

## Repository Discovery

On startup, the server automatically scans the `<ROOT>` directory for bare Git repositories. Scan depth is controlled by `--max-depth`:

- `--max-depth 0`: only scan the root directory
- `--max-depth 1`: scan root and its immediate subdirectories
- `--max-depth 3` (default): scan up to three levels of subdirectories

The server also periodically rescans the directory tree at the interval specified by `--rescan-interval-secs`, automatically picking up new or removed repositories.

## Repository Descriptions

If a bare repository directory contains a `description` file, the server includes its content in the repository listing API. The default placeholder text (`Unnamed repository; edit this file 'description' to name the repository.`) is filtered out.

## Supported Git Protocols

### Protocol v1 (default)

Used when the client does not specify a version via the `git-protocol` header. Supports:

- `git-upload-pack` (clone/fetch)
- `git-receive-pack` (push, must be enabled)

### Protocol v2

Automatically used when the client sets `git-protocol: version=2` in the request header. Supports:

- `ls-refs` command
- `fetch` command (including shallow clone support)

## Embed as a Library

The `gitserver-http` crate provides `SharedState` and `router` functions for embedding the server into a larger Axum application:

```rust
use gitserver_core::discovery::RepoStore;
use gitserver_http::{SharedState, router};

let store = RepoStore::discover("./repos".into(), 3)?;
let state = SharedState::new(store);
let app = router(state);

// Mount `app` into your Axum service
```

`SharedState` supports two repository management modes:

- **Discovery mode**: automatic filesystem scanning via `RepoStore`
- **Dynamic mode**: manual register/unregister of repositories via `DynamicRepoRegistry`
