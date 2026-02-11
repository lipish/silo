# 故障排除指南

## 编译错误：`edition2024` 特性未稳定

### 问题描述

如果遇到以下错误：
```
error: feature `edition2024` is required
The package requires the Cargo feature called `edition2024`, but that feature is not stabilized in this version of Cargo (1.82.0)
```

这是因为某些依赖（如 `dlopen2_derive`）需要更新的 Cargo 版本。

### 解决方案

**重要**：这个问题是由于 Tauri v2 的某些依赖需要更新的 Cargo 版本。必须更新 Cargo 才能解决。

#### 方案 1：更新 Rust/Cargo 工具链（必需）

```bash
# 使用 rustup 更新到最新稳定版
rustup update stable

# 验证版本
cargo --version
# 应该显示 1.83.0 或更高版本

# 如果还是旧版本，尝试：
rustup self update
rustup update stable --force
```

#### 方案 2：使用 nightly 版本（如果稳定版不可用）

```bash
# 安装 nightly 工具链
rustup toolchain install nightly

# 在项目目录中设置使用 nightly
cd silo/src-tauri
rustup override set nightly

# 验证
cargo --version
```

#### 方案 2：使用 nightly 版本（临时方案）

```bash
# 安装 nightly 工具链
rustup toolchain install nightly

# 在项目中使用 nightly
rustup override set nightly

# 或者只对当前命令使用
cargo +nightly build
```

#### 方案 3：清理并重新构建

```bash
cd silo/src-tauri
rm -rf Cargo.lock target
cargo clean
cargo build
```

### 当前状态

项目已暂时移除了以下依赖以避免编译问题：
- `lancedb` - 向量数据库（当前使用内存存储）
- `libp2p` - P2P 网络（当前未实现）
- `wasmtime` - WebAssembly 运行时（当前使用模拟实现）
- `tauri-plugin-dialog` - 文件对话框（改用原生 HTML input）
- `tauri-plugin-shell` - Shell 插件（暂时不需要）
- `tauri-plugin-fs` - 文件系统插件（暂时不需要）

这些功能将在后续版本中逐步集成。

### 验证安装

运行以下命令验证环境：

```bash
# 检查 Rust 版本
rustc --version

# 检查 Cargo 版本
cargo --version

# 检查 Tauri CLI
npm run tauri -- --version
```

### 如果问题仍然存在

1. 确保使用最新的 Rust 稳定版：`rustup update stable`
2. 清理缓存：`cargo clean && rm -rf Cargo.lock`
3. 重新安装依赖：`cd silo && npm install`
4. 重新构建：`npm run tauri dev`

---

**最后更新**: 2026-02-11
