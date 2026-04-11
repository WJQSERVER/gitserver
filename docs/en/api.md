# API Reference

## REST API

### Health Check

```
GET /healthz
```

Lightweight readiness/liveness probe. No authentication required.

Returns `200 OK` with `{"status":"ok"}` during normal operation. Once graceful shutdown begins, it returns `503 Service Unavailable` with `{"status":"shutting_down"}` so external load balancers can stop sending new traffic while in-flight Git requests finish draining.

**Response:**

```json
{
  "status": "ok"
}
```

### Repository Listing

```
GET /
```

Returns a list of bare Git repositories currently exposed by the server, whether they come from filesystem discovery or dynamic registration. Requires valid credentials if authentication is configured.

**Response:**

```json
[
  {
    "name": "my-project.git",
    "relative_path": "my-project.git",
    "description": "My project"
  },
  {
    "name": "nested.git",
    "relative_path": "org/nested.git"
  }
]
```

The `absolute_path` field is skipped during JSON serialization and is used internally for path resolution only.

When authentication is configured, all Git endpoints and the repository listing endpoint require valid credentials. `GET /healthz` remains unauthenticated.

## Git Smart HTTP Protocol

### Ref Advertisement (info/refs)

```
GET /{repo}/info/refs?service=git-upload-pack
GET /{repo}/info/refs?service=git-receive-pack
```

`git-receive-pack` is disabled by default and requires `--enable-receive-pack`.

Returns the Git ref advertisement used for the initial handshake in `git clone` and `git fetch`.

**Request Headers:**

- `git-protocol`: set to `version=2` to enable protocol v2
- `Accept-Encoding`: supports `gzip` and `zstd` compression

**Response Headers:**

- `Content-Type`: `application/x-git-upload-pack-advertisement` or `application/x-git-receive-pack-advertisement`
- `Cache-Control`: `no-cache`
- `Content-Encoding`: `gzip` or `zstd` (if supported by client)

### Pack Upload (git-upload-pack)

```
POST /{repo}/git-upload-pack
```

Streams clone/fetch pack data. When `--max-pack-bytes` is configured, requests whose estimated uncompressed pack output exceeds the limit return `413 Payload Too Large`. If request body upload or server-side handling exceeds `--request-timeout-secs`, the server returns `408 Request Timeout`.

Handles the client's pack request, generating and returning a Git pack file containing the requested objects.

**Request Headers:**

- `Content-Type`: `application/x-git-upload-pack-request`
- `git-protocol`: optional, set to `version=2` to enable protocol v2

**Response Headers:**

- `Content-Type`: `application/x-git-upload-pack-result`
- `Cache-Control`: `no-cache`

The response body is a stream of side-band-64k framed pack data.

### Pack Receive (git-receive-pack)

```
POST /{repo}/git-receive-pack
```

This endpoint is disabled by default and requires `--enable-receive-pack`.

Receives pack data pushed by the client and updates references. Must be enabled via `--enable-receive-pack`.

**Request Headers:**

- `Content-Type`: `application/x-git-receive-pack-request`

**Response Headers:**

- `Content-Type`: `application/x-git-receive-pack-result`
- `Cache-Control`: `no-cache`

**Restrictions:**

- Branch updates under `refs/heads/*` must be fast-forward
- Ref deletion is not permitted (new_id cannot be zero OID)
- Updating existing tags is not allowed
- Branch updates must point to a commit object
- Receive timeout of 300 seconds, idle timeout of 30 seconds

## Protocol v2 Commands

When the client specifies `version=2` in the `git-protocol` header, the `/info/refs` and `/git-upload-pack` endpoints support protocol v2.

### Server-Advertised Capabilities

```
ls-refs=unborn
fetch=shallow wait-for-done
object-format=sha1
```

### ls-refs Command

List repository references.

**Request Arguments:**

- `peel`: request peeled ref information
- `symrefs`: request symbolic ref targets
- `unborn`: support unborn HEAD
- `ref-prefix <prefix>`: filter by ref prefix (e.g. `refs/heads/`)

### fetch Command

Fetch object packs. Supports shallow clones and depth limiting.

**Request Arguments:**

- `want <oid>`: requested objects
- `have <oid>`: objects the client already has
- `done`: negotiation complete
- `ofs-delta`: support offset delta compression
- `deepen <N>`: limit clone depth
- `shallow <oid>`: client's existing shallow boundaries
- `deepen-relative`: relative depth calculation
- `thin-pack`, `no-progress`, `include-tag`, `wait-for-done`: supported but ignored

## Authentication

The server supports two HTTP authentication methods:

### Basic Authentication

Configured via `--auth-basic-username` and `--auth-basic-password`. Uses constant-time comparison to prevent timing attacks.

### Bearer Authentication

Configured via `--auth-bearer-token`. Pass `Bearer <token>` in the `Authorization` header.

Failed authentication returns a 401 status with a `WWW-Authenticate` header.

## Error Responses

Application-level errors produced by `gitserver-http` are returned in JSON format:

```json
{
  "error": "error_code",
  "message": "Human readable description"
}
```

| HTTP Status | Error Code | Description |
|-------------|-----------|-------------|
| 400 | `bad_request` | Malformed request (e.g. path traversal attempt, invalid pkt-line) |
| 401 | `unauthorized` | Authentication required |
| 404 | `not_found` | Repository not found or service disabled |
| 500 | `internal_error` | Internal server error |

Framework-level request validation failures may still use Axum's default non-JSON responses. For example, `GET /{repo}/info/refs` without the required `service` query parameter returns a 400 generated before the handler runs.

## Service Policy

Service availability is controlled via `ServicePolicy`:

| Policy | Default | Description |
|--------|---------|-------------|
| `upload_pack` | `true` | Enable git-upload-pack (clone/fetch) |
| `upload_pack_v2` | `true` | Enable protocol v2 |
| `receive_pack` | `false` | Enable git-receive-pack (push) |
