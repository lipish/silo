# Silo AI

**Your Data's Fortress** - 隐私优先的本地 Agent 操作系统

采用 Rust GPUI 原生界面构建，替代原 Tauri + React 方案。

## 构建

需要 Rust nightly（GPUI 依赖 edition 2024）：

```bash
cd silo
cargo +nightly build
```

### Linux 依赖

```bash
sudo apt install libstdc++6 libxkbcommon-dev libxkbcommon-x11-dev
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
```

## 运行

```bash
cd silo
cargo +nightly run
```

## 技术栈

- **界面**: Rust GPUI (GPU 加速原生 UI)
- **后端**: 推理引擎、Vault 向量库、Agent 执行器

参考：[GPUI](https://www.gpui.rs/) | [Zed GPUI](https://github.com/zed-industries/zed/blob/main/crates/gpui/README.md)