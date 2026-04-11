# gitserver

基于**Rust**的无 git 依赖(非绑定)的 Git Smart HTTP 服务端实现(支持 v2), 另提供一个可选 CLI 便于本地测试.

gitserver 基于上游项目 [ggueret/git-server](https://github.com/ggueret/git-server). 如果你要把 Git Smart HTTP 服务接入现有**Rust**应用, 通常直接使用 `gitserver-core` 和 `gitserver-http` 两个库; 仓库里的 `gitserver` 二进制则是在此之上的一层 CLI 封装, 适合本地测试或独立运行.

## 上游来源

- 上游项目: [`ggueret/git-server`](https://github.com/ggueret/git-server)

## 许可证

- 本仓库以 MPL-2.0 作为主许可证, 见 `LICENSE`.
- 本仓库包含源自上游项目 `ggueret/git-server` 的代码; 原始 MIT 许可证文本保存在 `LICENSE-UPSTREAM-MIT` 以及各 crate 的 `license/UPSTREAM-LICENSE` 中.
- 由本仓库创建的文件会在适用处带有 MPL-2.0 声明; 继承自上游的文件仍以其保留的声明和许可证文件为准.

## 特性

- **库优先设计** -- 直接在现有 Axum/Tokio 应用中使用 `gitserver-core` 和 `gitserver-http`
- **原生 Git 操作** -- 支持 `clone`, `fetch`, protocol v2, 以及可选的 `receive-pack`, 运行时无需 `git`
- **仓库发现与动态注册** -- 既可以扫描文件系统, 也可以在进程内手动注册仓库
- **可选 CLI** -- 当你需要独立进程时, 可直接运行同一套库构成的服务器

## 库快速开始

### 添加依赖

```toml
[dependencies]
gitserver-core = { git = "https://github.com/WJQSERVER/gitserver" }
gitserver-http = { git = "https://github.com/WJQSERVER/gitserver" }
axum = "0.8"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

### 嵌入示例

把服务嵌入到 Axum 应用中:

```rust
use gitserver_core::discovery::RepoStore;
use gitserver_http::{SharedState, router};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let store = RepoStore::discover("./repos".into(), 3)?;
    let state = SharedState::new(store);
    let app = router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.expect("install Ctrl+C handler");
        })
        .await?;
    Ok(())
}
```

### 控制层说明

如果你需要更细粒度的控制:

- `gitserver-core` 负责协议操作, 仓库发现, 路径校验, protocol v2 和 receive-pack
- `gitserver-http` 提供 `SharedState`, 认证/策略配置, 以及 Axum 路由与处理器
- `gitserver` 是构建在这些库之上的 CLI 二进制文件

更完整的嵌入式用法见 [库调用指南](docs/zh/library.md).

## CLI 快速开始

```sh
git clone https://github.com/WJQSERVER/gitserver.git gitserver
cd gitserver
cargo install --path crates/gitserver

# 提供 ./repos 目录下所有裸仓库服务
gitserver ./repos

# 从服务器克隆
git clone http://127.0.0.1:3000/my-project.git
```

> 如果你想直接跑一个独立服务, 就用 CLI; 如果你是把能力嵌入现有应用, 则不需要它.

## CLI 使用方法

```
gitserver [OPTIONS] <ROOT>

参数:
  <ROOT>            包含裸 Git 仓库的根目录

选项:
  -b, --bind <ADDR>             绑定地址 [默认值: 127.0.0.1]
  -p, --port <PORT>             端口号 [默认值: 3000]
  -l, --log-level <LEVEL>       日志级别 [默认值: info]
      --log-format <FORMAT>     日志格式: text 或 json [默认值: text]
  -w, --workers <N>             Tokio 工作线程数
  --max-depth <N>               仓库发现的最大目录深度 [默认值: 3]
  --rescan-interval-secs <N>    定期重新扫描间隔 (秒) [默认值: 30]
  --auth-basic-username <USER>  要求 HTTP Basic 认证用户名
  --auth-basic-password <PASS>  要求 HTTP Basic 认证密码
  --auth-bearer-token <TOKEN>   要求 Bearer 认证令牌
  --enable-receive-pack         启用 git-receive-pack, 允许 push
  --request-timeout-secs <N>    upload-pack 和 receive-pack 请求超时时间(秒) [默认值: 300]
  --max-pack-bytes <BYTES>      upload-pack 响应允许的未压缩 pack 最大字节数
```

独立 CLI 在收到 `SIGINT`/`SIGTERM` 后会执行优雅关闭: `/healthz` 会切换为 `503`, listener 停止接受新连接, 已在进行中的 Git 请求会继续排空完成.

## API

| 方法 | 端点 | 描述 |
|------|------|------|
| GET | `/healthz` | 健康检查端点; 优雅关闭排空期间返回 `503` |
| GET | `/` | 返回当前暴露的仓库列表(来自扫描或动态注册) |
| GET | `/{repo}/info/refs?service=git-upload-pack` | Git 引用通告 |
| GET | `/{repo}/info/refs?service=git-receive-pack` | Git receive-pack 通告, 默认关闭 |
| POST | `/{repo}/git-upload-pack` | Git 包传输 |
| POST | `/{repo}/git-receive-pack` | 处理 Git push, 默认关闭 |

仓库列表响应示例:

```json
[
  {
    "name": "my-project.git",
    "relative_path": "my-project.git",
    "description": "我的项目"
  }
]
```

## 架构

项目组织为 Cargo 工作空间, 包含四个 crate:

- **gitserver-core** -- Git 协议操作(引用通告, 包生成), 仓库发现, 路径安全
- **gitserver-http** -- Axum HTTP 路由, 处理器, 认证/策略配置, 共享状态
- **gitserver** -- 对上述库做独立服务器组装的薄 CLI 封装
- **gitserver-bench** -- 性能基准测试

## 文档

完整文档见:
- [中文版](docs/zh/index.md)
- [英文版](docs/en/index.md)

包括:
- [库调用指南](docs/zh/library.md)
- [安装指南](docs/zh/installation.md)
- [使用指南](docs/zh/usage.md)
- [API 参考](docs/zh/api.md)
- [架构设计](docs/zh/architecture.md)

## 从源码构建

```sh
git clone https://github.com/WJQSERVER/gitserver.git gitserver
cd gitserver
cargo build --release
```

二进制文件位于 `target/release/gitserver`.

## 运行测试

```sh
cargo test --workspace --all-features
```

测试套件包括单元测试, 集成测试(针对运行中的服务器执行 `git clone`/`git fetch`)和负载测试(并发克隆).
