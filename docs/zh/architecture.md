# 架构设计

## 项目结构

```
repo/
├── crates/
│   ├── gitserver/          # CLI 二进制入口
│   ├── gitserver-core/     # Git 协议操作核心
│   ├── gitserver-http/     # HTTP 层（Axum）
│   └── gitserver-bench/    # 性能基准测试
├── docs/
│   ├── zh/                  # 中文文档
│   └── en/                  # 英文文档
├── Cargo.toml               # Workspace 定义
└── Makefile
```

## Crate 职责

### gitserver-core

核心 Git 协议操作库，不依赖任何 HTTP 框架。

| 模块 | 职责 |
|------|------|
| `refs` | 协议 v1 引用通告（`git-upload-pack`） |
| `receive_pack` | 引用通告与 pack 接收（`git-receive-pack`），含 fast-forward 验证 |
| `pack` | Pack 文件生成，支持 ofs-delta 压缩和 side-band-64k 帧 |
| `protocol_v2` | Git 协议 v2 支持（`ls-refs`、`fetch`、shallow clone） |
| `pktline` | pkt-line 编码/解码 |
| `discovery` | 仓库发现（`RepoStore`）与动态注册（`DynamicRepoRegistry`） |
| `backend` | `GitBackend` 封装，统一调用 refs/pack/receive-pack 操作 |
| `path` | 路径安全验证，防止目录遍历 |
| `error` | 统一错误类型 |

### gitserver-http

基于 Axum 的 HTTP 层，将核心操作暴露为 HTTP 端点。

| 模块 | 职责 |
|------|------|
| `handlers` | HTTP 处理器：路由分发、认证检查、协议协商、压缩协商 |
| `error` | `AppError` 枚举，将核心错误映射为 HTTP 状态码 |
| `lib` | `SharedState`、`router` 函数、认证配置、服务策略 |

### gitserver

CLI 二进制入口，负责：

- 解析命令行参数（clap）
- 初始化 tracing 日志
- 创建 `RepoStore` 并执行初始发现
- 构建 `SharedState` 和 Axum 路由
- 启动后台定期重新扫描任务
- 绑定 TCP 监听器并服务

### gitserver-bench

性能基准测试，不发布。包含：

- pack 生成基准
- 引用通告基准
- HTTP 克隆基准
- git clone 基准
- 并发场景基准

## 请求处理流程

### Clone/Fetch（协议 v1）

```
客户端                          服务器
  |                               |
  | GET /repo/info/refs           |
  | ?service=git-upload-pack      |
  |------------------------------>|
  |                               | 解析仓库路径
  |                               | 验证认证
  |                               | refs::advertise_refs()
  |                               | 返回引用通告
  |<------------------------------|
  |                               |
  | POST /repo/git-upload-pack    |
  | Content-Type: ...-request     |
  |------------------------------>|
  |                               | 解析 UploadPackRequest
  |                               | pack::generate_pack()
  |                               | 流式返回 side-band-64k pack
  |<------------------------------|
```

### Clone/Fetch（协议 v2）

```
客户端                          服务器
  | GET /repo/info/refs           |
  | git-protocol: version=2       |
  |------------------------------>|
  |                               | protocol_v2::advertise_capabilities()
  |<------------------------------|
  |                               |
  | POST /repo/git-upload-pack    |
  | git-protocol: version=2       |
  |------------------------------>|
  |                               | parse_command_request()
  |                               | ls-refs 或 fetch
  |<------------------------------|
```

### Push

```
客户端                          服务器
  | GET /repo/info/refs           |
  | ?service=git-receive-pack     |
  |------------------------------>|
  |                               | receive_pack::advertise_receive_refs()
  |<------------------------------|
  |                               |
  | POST /repo/git-receive-pack   |
  |------------------------------>|
  |                               | receive_pack::receive_pack()
  |                               | 1. 解析更新命令
  |                               | 2. 写入 pack 到 ODB
  |                               | 3. 验证引用更新（fast-forward 检查）
  |                               | 4. 更新引用
  |                               | 5. 返回状态报告
  |<------------------------------|
```

## 关键设计决策

### 无运行时 git 依赖

所有 Git 操作通过 [gitoxide](https://github.com/GitoxideLabs/gitoxide) 原生实现，不需要安装 `git` 二进制文件。Pack 生成、引用解析和对象遍历均由 gitoxide 处理。

### 流式 Pack 生成

Pack 文件通过 Tokio channel 流式传输，不将整个 pack 加载到内存。使用 side-band-64k 帧编码，兼容标准 Git 客户端。

### 双模式仓库管理

- **发现模式**：基于文件系统扫描，适合本地测试和简单部署
- **动态模式**：通过库接口在进程内手动注册/注销仓库，适合多租户场景

### 安全

- 路径遍历防护：词法规范化 + canonicalize 双重检查
- 认证使用 constant-time 比较，防止时序攻击
- Push 操作默认禁用；启用后，`refs/heads/*` 下的分支更新必须是 fast-forward，同时不允许删除引用，也不允许更新已有标签

### 两种仓库解析器

`SharedState` 内部通过 `RepoMode` 枚举区分两种模式：

- `RepoMode::Discovered`：持有 `Arc<RwLock<RepoStore>>`，支持定期刷新
- `RepoMode::Dynamic`：持有 `Arc<dyn RepoResolver>` 和 `Arc<dyn MutableRepoRegistry>`，支持手动注册/注销

两种模式通过统一的 `list()`、`resolve()` 接口访问，对 HTTP 层透明。
