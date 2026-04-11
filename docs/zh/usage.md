# 使用指南

## 基本用法

```sh
gitserver [OPTIONS] <ROOT>
```

`<ROOT>` 是包含裸 Git 仓库的根目录.

## 命令行参数

### 位置参数

| 参数 | 说明 |
|------|------|
| `<ROOT>` | 包含裸 Git 仓库的根目录 |

### 选项

| 选项 | 简写 | 默认值 | 说明 |
|------|------|--------|------|
| `--bind <ADDR>` | `-b` | `127.0.0.1` | 绑定地址 |
| `--port <PORT>` | `-p` | `3000` | 端口号 |
| `--log-level <LEVEL>` | `-l` | `info` | 日志级别(trace/debug/info/warn/error) |
| `--log-format <FORMAT>` | - | `text` | 日志格式: `text` 或 `json` |
| `--workers <N>` | `-w` | 自动 | Tokio 工作线程数 |
| `--max-depth <N>` | - | `3` | 仓库发现的最大目录深度 |
| `--rescan-interval-secs <N>` | - | `30` | 仓库列表自动重新扫描的间隔(秒) |
| `--auth-basic-username <USER>` | - | - | HTTP Basic 认证用户名(需配合 `--auth-basic-password`) |
| `--auth-basic-password <PASS>` | - | - | HTTP Basic 认证密码(需配合 `--auth-basic-username`) |
| `--auth-bearer-token <TOKEN>` | - | - | Bearer 认证令牌 |
| `--enable-receive-pack` | - | `false` | 启用 git-receive-pack, 允许 push 操作 |
| `--request-timeout-secs <N>` | - | `300` | upload-pack 和 receive-pack 请求的超时时间(秒) |
| `--max-pack-bytes <BYTES>` | - | - | 超过该未压缩大小的 upload-pack 响应会被拒绝 |

## 示例

### 启动服务器

```sh
# 使用默认设置
gitserver ./repos

# 指定地址和端口
gitserver -b 0.0.0.0 -p 8080 ./repos

# JSON 格式日志
gitserver --log-format json ./repos

# 限制扫描深度为 1
gitserver --max-depth 1 ./repos
```

### 启用认证

```sh
# Basic 认证
gitserver --auth-basic-username admin --auth-basic-password secret ./repos

# Bearer 令牌认证
gitserver --auth-bearer-token my-secret-token ./repos
```

### 启用 Push 支持

```sh
gitserver --enable-receive-pack ./repos
```

> 注意: push 操作默认禁用. 启用后, 服务器仅接受 fast-forward 更新, 不允许删除引用或覆盖已有标签.

### 配置限制

```sh
# 拒绝会返回超过 512 MiB pack 数据的 clone/fetch 请求
gitserver --max-pack-bytes $((512 * 1024 * 1024)) ./repos

# 在 2 分钟后终止卡住或过长的 fetch/push 请求
gitserver --request-timeout-secs 120 ./repos
```

### 克隆和获取

```sh
# 克隆仓库
git clone http://127.0.0.1:3000/my-project.git

# 从远程获取更新
git fetch

# 使用认证克隆
git clone http://admin:secret@127.0.0.1:3000/my-project.git
```

## 仓库发现

服务器启动时会自动扫描 `<ROOT>` 目录, 查找裸 Git 仓库. 扫描深度由 `--max-depth` 控制:

- `--max-depth 0`: 仅扫描根目录下的仓库
- `--max-depth 1`: 扫描根目录及其直接子目录
- `--max-depth 3` (默认): 最多扫描三层子目录

服务器还会按 `--rescan-interval-secs` 指定的间隔定期重新扫描, 自动发现新增或移除的仓库.

## 仓库描述

如果裸仓库目录下存在 `description` 文件, 服务器会在仓库列表 API 中返回其内容. 默认占位文本(`Unnamed repository; edit this file 'description' to name the repository.`)会被过滤掉.

## 支持的 Git 协议

### 协议 v1 (默认)

客户端通过 `git-protocol` 头不指定版本时使用. 支持:

- `git-upload-pack` (clone/fetch)
- `git-receive-pack` (push, 需启用)

### 协议 v2

客户端在请求头中设置 `git-protocol: version=2` 时自动使用. 支持:

- `ls-refs` 命令
- `fetch` 命令(含 shallow clone 支持)

## 作为库嵌入

`gitserver-http` crate 提供了 `SharedState` 和 `router` 函数, 可以将服务器嵌入到更大的 Axum 应用中:

```rust
use gitserver_core::discovery::RepoStore;
use gitserver_http::{SharedState, router};

let store = RepoStore::discover("./repos".into(), 3)?;
let state = SharedState::new(store);
let app = router(state);

// 将 app 挂载到你的 Axum 服务中
```

`SharedState` 支持两种仓库管理模式:

- **发现模式**: 基于文件系统自动扫描(通过 `RepoStore`)
- **动态模式**: 通过 `DynamicRepoRegistry` 手动注册/注销仓库
