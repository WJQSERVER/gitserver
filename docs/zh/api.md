# API 参考

## REST API

### 健康检查

```
GET /healthz
```

轻量级就绪/存活探针. 不需要认证.

**响应:**

```json
{
  "status": "ok"
}
```

### 仓库列表

```
GET /
```

返回当前由服务器暴露的裸 Git 仓库列表, 这些仓库既可以来自文件系统扫描, 也可以来自动态注册. 如果配置了认证, 需要有效的认证凭据.

**响应:**

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

`absolute_path` 字段在 JSON 序列化时被跳过, 仅用于内部解析.

配置认证后, 仓库列表和所有 Git 端点都需要有效凭据; 只有 `GET /healthz` 不受认证保护.

## Git Smart HTTP 协议

### 引用通告(info/refs)

```
GET /{repo}/info/refs?service=git-upload-pack
GET /{repo}/info/refs?service=git-receive-pack
```

`git-receive-pack` 默认关闭, 需通过 `--enable-receive-pack` 启用.

返回 Git 引用通告, 用于 `git clone` 和 `git fetch` 的初始握手.

**请求头:**

- `git-protocol`: 设置为 `version=2` 启用协议 v2
- `Accept-Encoding`: 支持 `gzip` 和 `zstd` 压缩

**响应头:**

- `Content-Type`: `application/x-git-upload-pack-advertisement` 或 `application/x-git-receive-pack-advertisement`
- `Cache-Control`: `no-cache`
- `Content-Encoding`: `gzip` 或 `zstd`(如果客户端支持)

### 包上传(git-upload-pack)

```
POST /{repo}/git-upload-pack
```

处理客户端的包请求, 生成并返回包含所需对象的 Git pack 文件.

**请求头:**

- `Content-Type`: `application/x-git-upload-pack-request`
- `git-protocol`: 可选, 设置为 `version=2` 启用协议 v2

**响应头:**

- `Content-Type`: `application/x-git-upload-pack-result`
- `Cache-Control`: `no-cache`

响应体为 side-band-64k 帧编码的 pack 数据流.

### 包接收(git-receive-pack)

```
POST /{repo}/git-receive-pack
```

该端点默认关闭, 需通过 `--enable-receive-pack` 启用.

接收客户端推送的 pack 数据并更新引用. 需要通过 `--enable-receive-pack` 启用.

**请求头:**

- `Content-Type`: `application/x-git-receive-pack-request`

**响应头:**

- `Content-Type`: `application/x-git-receive-pack-result`
- `Cache-Control`: `no-cache`

**限制:**

- `refs/heads/*` 下的分支更新必须是 fast-forward
- 不允许删除引用(new_id 不能为零 OID)
- 不允许更新已有标签
- 分支更新必须指向 commit 对象
- 接收超时 300 秒, 空闲超时 30 秒

## 协议 v2 命令

当客户端在 `git-protocol` 头中指定 `version=2` 时, `/info/refs` 和 `/git-upload-pack` 端点支持协议 v2.

### 服务端通告的能力

```
ls-refs=unborn
fetch=shallow wait-for-done
object-format=sha1
```

### ls-refs 命令

列出仓库引用.

**请求参数:**

- `peel`: 请求 peeled 引用信息
- `symrefs`: 请求符号引用目标
- `unborn`: 支持未初始化分支的 HEAD
- `ref-prefix <prefix>`: 过滤引用前缀(如 `refs/heads/`)

### fetch 命令

获取对象包. 支持 shallow clone 和深度限制.

**请求参数:**

- `want <oid>`: 请求的对象
- `have <oid>`: 客户端已有的对象
- `done`: 协商完成
- `ofs-delta`: 支持 offset delta 压缩
- `deepen <N>`: 限制克隆深度
- `shallow <oid>`: 客户端已有的浅边界
- `deepen-relative`: 相对深度计算
- `thin-pack`, `no-progress`, `include-tag`, `wait-for-done`: 支持但忽略

## 认证

服务器支持两种 HTTP 认证方式:

### Basic 认证

通过 `--auth-basic-username` 和 `--auth-basic-password` 配置. 使用 constant-time 比较防止时序攻击.

### Bearer 认证

通过 `--auth-bearer-token` 配置. 在 `Authorization` 头中传递 `Bearer <token>`.

未通过认证时返回 401 状态码和 `WWW-Authenticate` 头.

## 错误响应

由 `gitserver-http` 产生的应用层错误会以 JSON 格式返回:

```json
{
  "error": "error_code",
  "message": "Human readable description"
}
```

| HTTP 状态码 | error 代码 | 说明 |
|-------------|-----------|------|
| 400 | `bad_request` | 请求格式错误(如路径遍历尝试, 无效的 pkt-line) |
| 401 | `unauthorized` | 需要认证 |
| 404 | `not_found` | 仓库不存在或服务已禁用 |
| 500 | `internal_error` | 服务器内部错误 |

框架层的请求校验失败仍可能返回 Axum 默认的非 JSON 响应. 例如, 请求 `GET /{repo}/info/refs` 时缺少必需的 `service` 查询参数, 会在进入处理器之前直接返回 400.

## 服务策略

通过 `ServicePolicy` 控制各服务的启用状态:

| 策略 | 默认值 | 说明 |
|------|--------|------|
| `upload_pack` | `true` | 启用 git-upload-pack(clone/fetch) |
| `upload_pack_v2` | `true` | 启用协议 v2 |
| `receive_pack` | `false` | 启用 git-receive-pack(push) |
