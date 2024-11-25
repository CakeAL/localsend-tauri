# Localsend Tauri 使用 Tauri 构建的 Localsend 客户端

# Forever WIP, just toys

## 简介

使用 Rust 实现了 Localsend 协议，用 Tauri 包了层壳。

## 开发

```bash
# 安装 create-tauri-app
cargo install create-tauri-app --locked
# 安装tauri 命令行 https://tauri.app/zh-cn/blog/2022/09/15/tauri-1-1/#cargo-binstall-support-for-tauri-cli
cargo install tauri-cli --version "^2.0.0" --locked
# 运行
# 如果安装了 just
just d
# 或者
cargo tauri dev
# 或者
pnpm i
pnpm tauri dev
```