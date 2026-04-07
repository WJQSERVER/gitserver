# Architecture

## Project Structure

```
gitserver/
├── crates/
│   ├── gitserver/          # CLI binary entry point
│   ├── gitserver-core/     # Core Git protocol operations
│   ├── gitserver-http/     # HTTP layer (Axum)
│   └── gitserver-bench/    # Performance benchmarks
├── docs/
│   ├── zh/                  # Chinese documentation
│   └── en/                  # English documentation
├── Cargo.toml               # Workspace definition
└── Makefile
```

## Crate Responsibilities

### gitserver-core

Core Git protocol operations library, with no HTTP framework dependency.

| Module | Responsibility |
|--------|---------------|
| `refs` | Protocol v1 ref advertisement (`git-upload-pack`) |
| `receive_pack` | Ref advertisement and pack reception (`git-receive-pack`), with fast-forward validation |
| `pack` | Pack file generation with ofs-delta compression and side-band-64k framing |
| `protocol_v2` | Git protocol v2 support (`ls-refs`, `fetch`, shallow clone) |
| `pktline` | pkt-line encoding/decoding |
| `discovery` | Repository discovery (`RepoStore`) and dynamic registration (`DynamicRepoRegistry`) |
| `backend` | `GitBackend` wrapper unifying refs/pack/receive-pack operations |
| `path` | Path safety validation to prevent directory traversal |
| `error` | Unified error types |

### gitserver-http

Axum-based HTTP layer exposing core operations as HTTP endpoints.

| Module | Responsibility |
|--------|---------------|
| `handlers` | HTTP handlers: route dispatch, auth checks, protocol negotiation, compression negotiation |
| `error` | `AppError` enum mapping core errors to HTTP status codes |
| `lib` | `SharedState`, `router` function, auth config, service policy |

### gitserver

CLI binary entry point, responsible for:

- Parsing CLI arguments (clap)
- Initializing tracing logging
- Creating `RepoStore` and running initial discovery
- Building `SharedState` and Axum routes
- Spawning the background periodic rescan task
- Binding the TCP listener and serving

### gitserver-bench

Performance benchmarks, not published. Includes:

- Pack generation benchmarks
- Ref advertisement benchmarks
- HTTP clone benchmarks
- Git clone benchmarks
- Concurrent scenario benchmarks

## Request Flow

### Clone/Fetch (Protocol v1)

```
Client                            Server
  |                                 |
  | GET /repo/info/refs             |
  | ?service=git-upload-pack        |
  |-------------------------------->|
  |                                 | Resolve repo path
  |                                 | Verify auth
  |                                 | refs::advertise_refs()
  |                                 | Return ref advertisement
  |<--------------------------------|
  |                                 |
  | POST /repo/git-upload-pack      |
  | Content-Type: ...-request       |
  |-------------------------------->|
  |                                 | Parse UploadPackRequest
  |                                 | pack::generate_pack()
  |                                 | Stream side-band-64k pack
  |<--------------------------------|
```

### Clone/Fetch (Protocol v2)

```
Client                            Server
  | GET /repo/info/refs             |
  | git-protocol: version=2         |
  |-------------------------------->|
  |                                 | protocol_v2::advertise_capabilities()
  |<--------------------------------|
  |                                 |
  | POST /repo/git-upload-pack      |
  | git-protocol: version=2         |
  |-------------------------------->|
  |                                 | parse_command_request()
  |                                 | ls-refs or fetch
  |<--------------------------------|
```

### Push

```
Client                            Server
  | GET /repo/info/refs             |
  | ?service=git-receive-pack       |
  |-------------------------------->|
  |                                 | receive_pack::advertise_receive_refs()
  |<--------------------------------|
  |                                 |
  | POST /repo/git-receive-pack     |
  |-------------------------------->|
  |                                 | receive_pack::receive_pack()
  |                                 | 1. Parse update commands
  |                                 | 2. Write pack to ODB
  |                                 | 3. Validate ref updates (fast-forward check)
  |                                 | 4. Update references
  |                                 | 5. Return status report
  |<--------------------------------|
```

## Key Design Decisions

### No Runtime Git Dependency

All Git operations are implemented natively via [gitoxide](https://github.com/GitoxideLabs/gitoxide), with no need for the `git` binary at runtime. Pack generation, ref parsing, and object traversal are all handled by gitoxide.

### Streaming Pack Generation

Pack files are streamed through a Tokio channel rather than loaded entirely into memory. Side-band-64k framing ensures compatibility with standard Git clients.

### Dual-Mode Repository Management

- **Discovery mode**: filesystem-based scanning, suitable for local testing and simple deployments
- **Dynamic mode**: manual register/unregister through the library API, suitable for multi-tenant scenarios

### Security

- Path traversal protection: lexical normalization + canonicalize double-check
- Authentication uses constant-time comparison to prevent timing attacks
- Push operations are disabled by default; when enabled, branch updates under `refs/heads/*` must be fast-forward, ref deletions are rejected, and updating existing tags is blocked

### Two Repository Resolvers

`SharedState` uses a `RepoMode` enum internally to distinguish the two modes:

- `RepoMode::Discovered`: holds `Arc<RwLock<RepoStore>>`, supports periodic refresh
- `RepoMode::Dynamic`: holds `Arc<dyn RepoResolver>` and `Arc<dyn MutableRepoRegistry>`, supports manual register/unregister

Both modes are accessed through a unified `list()` and `resolve()` interface, transparent to the HTTP layer.
