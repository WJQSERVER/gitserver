# gitserver

适用于本地测试、运行时无需 git 的 smart HTTP Git 服务器。

`gitserver` 基于上游项目 [ggueret/git-server](https://github.com/ggueret/git-server)。它通过 HTTP 提供裸 Git 仓库的 `git clone` 和 `git fetch` 服务，运行时不需要 `git` 二进制文件。项目基于 [gitoxide](https://github.com/GitoxideLabs/gitoxide) 实现原生 Git 操作，采用 [Axum](https://github.com/tokio-rs/axum) / [Tokio](https://tokio.rs) 提供异步 HTTP 服务。

## 上游来源

- 上游项目：[`ggueret/git-server`](https://github.com/ggueret/git-server)

## 许可证

- 本仓库以 MPL-2.0 作为主许可证，见 `LICENSE`。
- 本仓库包含源自上游项目 `ggueret/git-server` 的代码；原始 MIT 许可证文本保存在 `LICENSE-UPSTREAM-MIT` 以及各 crate 的 `license/UPSTREAM-LICENSE` 中。
- 由本仓库创建的文件会在适用处带有 MPL-2.0 声明；继承自上游的文件仍以其保留的声明和许可证文件为准。

## 特性

- **单二进制文件，无需 git** -- 所有 Git 操作均原生处理，无运行时依赖
- **多仓库支持** -- 服务于根目录下的所有裸仓库，扫描深度可配置
- **JSON API** -- 提供仓库列表端点，支持程序化发现
- **结构化日志** -- 支持文本或 JSON 格式的日志输出

## 快速开始

```sh
cargo install --path crates/gitserver

# 提供 ./repos 目录下所有裸仓库服务
gitserver ./repos

# 从服务器克隆
git clone http://127.0.0.1:3000/my-project.git
```

## 使用方法

```
gitserver [OPTIONS] <ROOT>

参数:
  <ROOT>            包含裸 Git 仓库的根目录

选项:
  -b, --bind <ADDR>     绑定地址 [默认值：127.0.0.1]
  -p, --port <PORT>     端口号 [默认值：3000]
  -l, --log-level <LEVEL>  日志级别 [默认值：info]
  --log-format <FORMAT>  日志格式：text 或 json [默认值：text]
  -w, --workers <N>      Tokio 工作线程数
  --max-depth <N>        仓库发现的最大目录深度 [默认值：3]
  --rescan-interval-secs <N>  定期重新扫描间隔（秒）[默认值：30]
  --auth-basic-username <USER>  要求 HTTP Basic 认证用户名
  --auth-basic-password <PASS>  要求 HTTP Basic 认证密码
  --auth-bearer-token <TOKEN>   要求 Bearer 认证令牌
  --enable-receive-pack         启用 git-receive-pack，允许 push
```

## API

| 方法   | 端点                                      | 描述                |
| ------ | ----------------------------------------- | ------------------ |
| GET    | `/healthz`                                | 健康检查端点        |
| GET    | `/`                                       | 返回已发现仓库的 JSON 数组 |
| GET    | `/{repo}/info/refs?service=git-upload-pack` | Git 引用通告        |
| GET    | `/{repo}/info/refs?service=git-receive-pack` | Git receive-pack 通告 |
| POST   | `/{repo}/git-upload-pack`                 | Git 包传输          |
| POST   | `/{repo}/git-receive-pack`                | 处理 Git push       |

仓库列表响应示例：

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

项目组织为 Cargo 工作空间，包含四个 crate：

- **gitserver-core** -- Git 协议操作（引用通告、包生成）、仓库发现、路径安全
- **gitserver-http** -- Axum HTTP 路由、处理器、错误响应
- **gitserver** -- CLI 二进制文件、tracing 设置、服务器组装
- **gitserver-bench** -- 性能基准测试

## 文档

完整文档见[中文版](docs/zh/index.md)和[英文版](docs/en/index.md)：

- [安装指南](docs/zh/installation.md)
- [使用指南](docs/zh/usage.md)
- [API 参考](docs/zh/api.md)
- [架构设计](docs/zh/architecture.md)
- [库调用指南](docs/zh/library.md)

## 从源码构建

```sh
git clone https://github.com/WJQSERVER/git-server.git
cd git-server
cargo build --release
```

二进制文件位于 `target/release/gitserver`。

## 运行测试

```sh
cargo test --workspace
```

测试套件包括单元测试、集成测试（针对运行中的服务器执行 `git clone`/`git fetch`）和负载测试（并发克隆）。
