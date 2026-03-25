# 快速参考指南

## 📦 依赖模式切换

### 开发模式（本地路径 - 当前）
```toml
# src-tauri/Cargo.toml
sherpa-rs = { path = "/Users/thinkre/Desktop/Thinkre/sherpa-rs/crates/sherpa-rs" }
sherpa-rs-sys = { path = "/Users/thinkre/Desktop/Thinkre/sherpa-rs/crates/sherpa-rs-sys" }
```

优点:
- ⚡ 无需网络下载
- 🔧 可直接修改 sherpa-rs 源码
- 🚀 编译更快（跳过 git 操作）

### 发布模式（Git 依赖）
```toml
# src-tauri/Cargo.toml
sherpa-rs = { git = "https://github.com/Thinkre/sherpa-rs", branch = "main" }
sherpa-rs-sys = { git = "https://github.com/Thinkre/sherpa-rs", branch = "main", package = "sherpa-rs-sys" }
```

优点:
- 📦 其他用户可直接编译
- 🔄 版本管理更清晰
- 🌐 不依赖本地路径

## 🛠️ 常用命令

### 编译
```bash
cd src-tauri

# 调试模式
cargo build

# 发布模式
cargo build --release

# 只检查语法
cargo check
```

### 运行
```bash
# 测试 sherpa API
cargo run --bin test_sherpa_api

# 运行完整应用
cd ..
bun run tauri:dev
```

### 清理
```bash
cd src-tauri
cargo clean
```

## 🔄 更新 sherpa-rs

### 拉取最新代码
```bash
cd ~/Desktop/Thinkre/sherpa-rs
git pull origin main
git submodule update --init --recursive
```

### 修改并提交
```bash
cd ~/Desktop/Thinkre/sherpa-rs

# 修改代码...

git add .
git commit -m "your commit message"
git push origin main
```

## 🐛 调试技巧

### 查看详细编译输出
```bash
cd src-tauri
SHERPA_BUILD_DEBUG=1 cargo build -vv 2>&1 | tee build.log
```

### 检查动态库依赖 (macOS)
```bash
otool -L target/debug/kevoiceinput
```

### 检查动态库依赖 (Linux)
```bash
ldd target/debug/kevoiceinput
```

### 设置环境变量
```bash
# macOS
export DYLD_LIBRARY_PATH=/path/to/libs:$DYLD_LIBRARY_PATH

# Linux
export LD_LIBRARY_PATH=/path/to/libs:$LD_LIBRARY_PATH

# 跳过 CMake 编译（使用预编译库）
export SHERPA_LIB_PATH=/path/to/sherpa-onnx/build
```

## 📝 提交到 Git

### 提交 KeVoiceInput 更改
```bash
cd /Users/thinkre/Desktop/KeVoiceInput
git add src-tauri/Cargo.toml
git commit -m "chore: update sherpa-rs dependencies to use GitHub repos"
git push origin git_repo
```

### 切换回 Git 依赖（发布前）
1. 编辑 `src-tauri/Cargo.toml`
2. 将路径依赖改回 Git 依赖
3. 测试编译: `cargo build --release`
4. 提交: `git commit -m "chore: switch to Git dependencies for release"`

## 🔍 问题诊断

### 编译卡住
```bash
# 查看进程
ps aux | grep -E "(cargo|rustc|cmake|cc)"

# 查看最近的输出
tail -f build.log
```

### 网络问题
```bash
# 清除 Cargo 缓存
rm -rf ~/.cargo/registry
rm -rf ~/.cargo/git

# 使用国内镜像（可选）
# 编辑 ~/.cargo/config.toml
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "sparse+https://mirrors.ustc.edu.cn/crates.io-index/"
```

### sherpa-onnx 构建失败
```bash
# 手动构建 sherpa-onnx
cd ~/Desktop/Thinkre/sherpa-rs/crates/sherpa-rs-sys/sherpa-onnx
mkdir -p build && cd build
cmake ..
make -j$(nproc)

# 然后使用预编译库
export SHERPA_LIB_PATH=$(pwd)
cd /Users/thinkre/Desktop/KeVoiceInput/src-tauri
cargo build
```

## 📊 接口对比

### ParaformerConfig
```rust
// 旧版本（vendor）
ParaformerConfig {
    model: String,
    tokens: String,
    // ...
}

// 新版本（Thinkre/sherpa-rs）- 完全兼容
ParaformerConfig {
    model: String,
    tokens: String,
    model_eb: Option<String>,        // ✅ 新增
    hotwords_file: Option<String>,   // ✅ 新增
    hotwords_score: f32,             // ✅ 新增
    provider: Option<String>,
    num_threads: Option<i32>,
    debug: bool,
}
```

### 使用示例
```rust
// 标准 Paraformer
let config = ParaformerConfig {
    model: "model.onnx".to_string(),
    tokens: "tokens.txt".to_string(),
    model_eb: None,
    hotwords_file: None,
    hotwords_score: 0.0,
    provider: None,
    num_threads: Some(1),
    debug: false,
};

// SeACo Paraformer with 热词
let config = ParaformerConfig {
    model: "model.onnx".to_string(),
    tokens: "tokens.txt".to_string(),
    model_eb: Some("model_eb.onnx".to_string()),      // SeACo
    hotwords_file: Some("hotwords.txt".to_string()),   // 热词
    hotwords_score: 2.0,                               // 热词权重
    provider: None,
    num_threads: Some(1),
    debug: false,
};
```

## 🎯 下一步

- [ ] 等待首次编译完成
- [ ] 运行测试: `cargo run --bin test_sherpa_api`
- [ ] 测试完整应用: `bun run tauri:dev`
- [ ] 验证所有模型类型
- [ ] 提交更改到 git_repo 分支
- [ ] (可选) 切换回 Git 依赖并发布
