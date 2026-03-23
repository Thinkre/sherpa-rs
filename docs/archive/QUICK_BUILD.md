# KeVoiceInput 快速构建指南

## 一键构建

```bash
bun run tauri:build
```

这个命令会自动完成所有工作，包括：
- ✅ 构建应用
- ✅ 打包动态库
- ✅ 代码签名
- ✅ 创建 DMG

## 构建产物

构建完成后，你会得到：

1. **App Bundle** (可直接运行)
   ```
   src-tauri/target/release/bundle/macos/KeVoiceInput.app
   ```

2. **DMG 安装包** (~34MB)
   ```
   src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg
   ```

## 使用 DMG

### 方法 1：查看安装说明（推荐）📖
1. 双击打开 DMG 文件
2. 双击 **README.txt** 查看完整安装指南
3. 按照说明选择最适合的安装方式

### 方法 2：自动安装脚本 🚀
1. 双击打开 DMG 文件
2. 双击 **Install.command**
3. 如果 macOS 阻止，使用终端：`/Volumes/KeVoiceInput/Install.command`
4. 按提示完成安装（会自动放行）

### 方法 3：手动拖拽 ✅
1. 双击打开 DMG 文件
2. 将 KeVoiceInput.app 拖到 Applications 文件夹
3. 打开终端：`xattr -cr /Applications/KeVoiceInput.app`
4. 首次启动需右键点击选择"打开"

## 常见问题

### Q: 构建时间太长？
A: 首次构建需要 5-10 分钟，后续增量构建会快很多（约 2-3 分钟）。

### Q: 构建失败？
A: 清理后重试：
```bash
cd src-tauri && cargo clean && cd ..
bun run tauri:build
```

### Q: 应用闪退？
A: 如果你使用的是旧的 DMG，请重新构建：
```bash
bun run tauri:build
```
新构建的 DMG 包含所有必需的动态库。

### Q: 想要开发模式？
A: 使用：
```bash
bun run tauri:dev
```

## 详细文档

- [BUILD_GUIDE.md](docs/BUILD_GUIDE.md) - 完整构建指南
- [BUILD_FIX.md](docs/BUILD_FIX.md) - 技术细节

## 验证构建

检查 DMG 是否包含动态库：
```bash
# 挂载 DMG
hdiutil attach src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg

# 检查库文件（应该看到 4 个 .dylib 文件）
ls -la /Volumes/KeVoiceInput/KeVoiceInput.app/Contents/Frameworks/

# 卸载 DMG
hdiutil detach /Volumes/KeVoiceInput
```

应该看到：
- libcargs.dylib
- libonnxruntime.1.17.1.dylib
- libsherpa-onnx-c-api.dylib
- libsherpa-onnx-cxx-api.dylib

## 发布准备

如果要发布给其他用户：
1. 获取 Apple Developer 账号
2. 配置代码签名证书
3. 配置 App 公证环境变量
4. 运行 `bun run tauri:build`

详见 [BUILD_GUIDE.md](docs/BUILD_GUIDE.md) 的"发布构建"部分。

---

**最后更新**: 2026-02-02  
**状态**: ✅ 所有已知问题已修复
