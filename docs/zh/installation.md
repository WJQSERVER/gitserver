# 安装指南

## 前置条件

- **Rust 工具链**：需要 Rust 1.85 或更高版本（项目使用 Edition 2024）
- **Git**：仅用于测试，运行时不需要

## 从源码安装

```sh
git clone https://github.com/WJQSERVER/gitserver.git gitserver
cd gitserver
cargo install --path crates/gitserver
```

安装完成后，`gitserver` 二进制文件会出现在 Cargo 的 bin 目录中（通常是 `~/.cargo/bin/gitserver`）。

## 构建发布版本

```sh
cargo build --release
```

编译后的二进制文件位于 `target/release/gitserver`。

## 作为库使用

项目以 Cargo workspace 形式组织，包含四个 crate：

| Crate | 说明 |
|-------|------|
| `gitserver` | CLI 二进制入口 |
| `gitserver-core` | Git 协议操作、仓库发现、路径安全 |
| `gitserver-http` | Axum HTTP 路由与处理器 |
| `gitserver-bench` | 性能基准测试（不发布） |

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
gitserver-core = { git = "https://github.com/WJQSERVER/gitserver" }
gitserver-http = { git = "https://github.com/WJQSERVER/gitserver" }
```

## 运行测试

```sh
cargo test --workspace --all-features
```

测试覆盖单元测试、集成测试（`git clone`/`git fetch`）和负载测试。

## 运行基准测试

```sh
cargo bench -p gitserver-bench
```

基准测试包括 pack 生成、引用通告、HTTP 克隆、git clone 和并发场景。

## 代码检查

```sh
# 格式化
cargo fmt

# Clippy 检查
cargo clippy --all-targets --all-features -- -D warnings

# 一键检查（格式化 + clippy + 测试）
make check
```
