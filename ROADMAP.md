# Roadmap

## v0.1 -- Foundation (current)

Native smart HTTP Git server with gitoxide. Read-only clone and fetch.

- [x] Multi-crate workspace (core, http, binary)
- [x] Repository discovery with configurable depth
- [x] Ref advertisement via gitoxide
- [x] Pack generation with side-band-64k framing
- [x] JSON repository listing endpoint
- [x] CLI with configurable bind, port, log level/format, workers
- [x] Path traversal protection
- [x] Content-Type validation
- [x] Integration tests (clone, fetch, errors)
- [x] Load tests (concurrent clones)
- [x] Criterion benchmarks (pack generation, ref advertisement, clone end-to-end, concurrent clones)

## v0.2 -- Streaming and efficiency

Reduce memory footprint and improve performance for large repositories.

- [x] Git protocol v2 support (`ls-refs`, `fetch` over smart HTTP)
- [x] Streaming pack generation (avoid building the entire packfile in memory)
- [x] Delta compression (OFS_DELTA) to reduce transfer size
- [x] Multi-ack negotiation for efficient incremental fetches
- [x] Shallow clone support
- [x] Response compression (gzip/zstd on ref advertisement)

## v0.3 -- Production readiness

Operational features for running at scale.

- [ ] Hot-reload of repository list (watch filesystem or periodic rescan)
- [ ] Health check endpoint (`GET /healthz`)
- [ ] Prometheus metrics (request count, latency, pack size, active connections)
- [ ] Graceful shutdown with in-flight request draining
- [ ] Configuration file (TOML) as alternative to CLI flags
- [ ] Request timeout and max pack size limits

## v0.4 -- Write support

Enable push operations.

- [ ] `git-receive-pack` endpoint (POST + ref advertisement)
- [ ] Pre-receive and post-receive hook support
- [ ] Per-repository access control (read-only vs read-write)
- [ ] Ref update validation

## Future considerations

- Authentication (Basic, Bearer token, mTLS)
- Repository creation via API
- Web UI for repository browsing
- Replication / mirroring between instances
- Container image (distroless)
