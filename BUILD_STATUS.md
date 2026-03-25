# 编译状态更新

**时间**: 2026-03-25 12:45
**状态**: 🔄 编译进行中

## 最新更改

### 禁用 download-binaries 功能

更新了 `src-tauri/Cargo.toml`:
```toml
sherpa-rs = { path = "/Users/thinkre/Desktop/Thinkre/sherpa-rs/crates/sherpa-rs", default-features = false }
sherpa-rs-sys = { path = "/Users/thinkre/Desktop/Thinkre/sherpa-rs/crates/sherpa-rs-sys", default-features = false }
```

**原因**:
- sherpa-rs 默认启用 `download-binaries` feature
- 这会尝试从网络下载预编译的二进制文件
- 由于网络速度慢，下载卡住
- 禁用后将使用本地 CMake 编译

### 当前编译流程

1. ✅ 解析本地依赖
2. 🔄 编译 sherpa-rs-sys (包括 C++ 代码)
3. ⏳ 编译 sherpa-rs
4. ⏳ 编译 KeVoiceInput

## 预计时间

- **首次编译**: 15-30 分钟（需要编译 C++ 代码）
- **后续编译**: 几秒到几分钟（增量编译）

## 编译内容

### sherpa-rs-sys (最耗时)
- 使用 CMake 构建 sherpa-onnx C++ 库
- 包括 onnxruntime 和大量 C++ 源文件
- 生成静态库或动态库

### sherpa-rs
- Rust 绑定层
- 封装 C API 为安全的 Rust 接口

### kevoiceinput
- 主应用程序代码
- Tauri 后端

## 监控编译进度

### 查看进程
```bash
ps aux | grep -E "(cargo|cmake|cc|clang)" | grep -v grep
```

### 查看构建目录
```bash
cd /Users/thinkre/Desktop/KeVoiceInput/src-tauri
ls -la target/debug/build/
```

### 实时日志
```bash
tail -f /tmp/cargo_build.log
```

## 如果编译失败

### 常见错误

1. **CMake 相关错误**
   ```bash
   # 检查 CMake 是否安装
   cmake --version

   # 如果未安装
   brew install cmake
   ```

2. **C++ 编译器错误**
   ```bash
   # 检查 Xcode Command Line Tools
   xcode-select --install
   ```

3. **内存不足**
   - 关闭其他应用程序
   - 或使用预编译的库（见 CURRENT_STATUS.md）

### 使用预编译库（快速方案）

如果等待时间太长或编译失败，可以使用预编译的库：

```bash
# 方案 1: 使用 vendor 目录的库（临时）
export SHERPA_LIB_PATH=/Users/thinkre/Desktop/KeVoiceInput/vendor/libs/macos-arm64
cd /Users/thinkre/Desktop/KeVoiceInput/src-tauri
cargo build

# 方案 2: 启用 download-binaries（如果网络恢复）
# 编辑 Cargo.toml，移除 default-features = false
```

## 编译完成后

### 1. 验证编译成功
```bash
cd /Users/thinkre/Desktop/KeVoiceInput/src-tauri
ls -lh target/debug/kevoiceinput
```

### 2. 测试程序
```bash
# 测试 sherpa API
cargo run --bin test_sherpa_api

# 运行完整应用
cd ..
bun run tauri:dev
```

### 3. 清理并发布构建
```bash
cd src-tauri
cargo clean
cargo build --release
```

## 提交更改

编译成功并测试通过后:

```bash
cd /Users/thinkre/Desktop/KeVoiceInput
git add src-tauri/Cargo.toml
git add *.md
git commit -m "feat: migrate to Thinkre sherpa-rs/sherpa-onnx repos

- Use local path dependencies for development
- Disable download-binaries feature
- All interfaces remain compatible
- Add comprehensive migration documentation"

git push origin git_repo
```

## 后续计划

### 短期
- [x] 配置 Git 仓库
- [x] 更新项目依赖
- [x] 禁用自动下载功能
- [ ] 完成首次编译
- [ ] 测试所有功能

### 中期
- [ ] 验证所有模型类型
- [ ] 性能测试
- [ ] 创建 Pull Request

### 长期
- [ ] 切换到 Git 依赖（发布版）
- [ ] 更新文档
- [ ] 发布新版本

## 相关文档

- `MIGRATION_SUMMARY.md` - 迁移总结
- `QUICK_REFERENCE.md` - 快速参考
- `CURRENT_STATUS.md` - 详细状态

## 注意事项

### 关于 default-features = false

当前配置禁用了 sherpa-rs 的默认 features，这意味着：
- ❌ 不会自动下载预编译二进制
- ✅ 使用本地 CMake 编译
- ✅ 可以自定义编译选项

如果需要启用某些 features（如 TTS），可以手动指定：
```toml
sherpa-rs = {
    path = "...",
    default-features = false,
    features = ["tts"]  # 根据需要添加
}
```

### 关于编译时间

首次编译 sherpa-onnx 需要较长时间是正常的，因为：
1. 包含大量 C++ 源文件
2. 需要编译 onnxruntime
3. 需要生成绑定代码

**好消息**: 这是一次性的！后续编译会非常快。

## 联系支持

如果遇到问题：
1. 查看上面的故障排除部分
2. 检查 GitHub Issues: https://github.com/Thinkre/sherpa-rs/issues
3. 参考 sherpa-onnx 文档: https://k2-fsa.github.io/sherpa/onnx/

---

**提示**: 可以开始做其他工作，编译会在后台继续进行。编译完成后会有通知。
