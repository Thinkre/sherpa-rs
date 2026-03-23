# Tauri Build Fix - 动态库打包问题

## 问题描述

在 macOS 上运行 `bun run tauri build` 构建应用后，双击打开 DMG 安装的应用会崩溃，显示"kevoiceinput 意外退出"。

## 根本原因

应用依赖以下动态库（dylib），但这些库没有被打包到 `.app` bundle 中：
- `libcargs.dylib`
- `libonnxruntime.1.17.1.dylib`
- `libsherpa-onnx-c-api.dylib`
- `libsherpa-onnx-cxx-api.dylib`

当应用启动时，macOS 找不到这些库，导致启动失败：
```
dyld: Library not loaded: @rpath/libcargs.dylib
```

## 解决方案

我们创建了一个后处理脚本 `scripts/copy-dylibs.sh`，它会：

1. 将所需的动态库复制到 `KeVoiceInput.app/Contents/Frameworks/` 目录
2. 更新二进制文件的 `rpath`，将 `@rpath/` 改为 `@executable_path/../Frameworks/`
3. 重新签名所有动态库、可执行文件和整个 app bundle

## 使用方法

### 方法 1：使用包装脚本（推荐）

```bash
bun run tauri:build
```

这个命令会：
1. 运行标准的 `tauri build`
2. 自动复制动态库到 app bundle
3. 重新签名所有二进制文件

### 方法 2：手动修复已构建的应用

如果你已经构建了应用，可以手动运行修复脚本：

```bash
./scripts/copy-dylibs.sh src-tauri/target/release/bundle/macos/KeVoiceInput.app
```

## 技术细节

### 为什么 Tauri 的 `bundle.macOS.files` 不起作用？

Tauri 2 的配置格式对于动态库打包有特定要求，直接在 `tauri.conf.json` 中配置 `files` 字段容易出错。使用后处理脚本更可靠。

### 为什么需要重新签名？

当我们添加新文件到 app bundle 后，原有的代码签名会失效。macOS 的安全机制要求：
- 所有动态库必须签名
- 所有可执行文件必须签名
- 整个 app bundle 必须签名
- 签名必须一致（不能混合使用不同的 Team ID）

我们使用 `codesign --force --sign -` 进行本地开发签名（adhoc 签名），这对于本地测试足够了。

### 为什么不使用 `--options runtime`？

`runtime` 选项会启用 Hardened Runtime，这要求更严格的签名验证。对于本地开发构建，不使用 runtime 选项可以避免 Team ID 不匹配的问题。

如果你需要分发应用给其他用户，需要：
1. 使用有效的 Apple Developer 证书签名
2. 启用 Hardened Runtime
3. 进行 App 公证（notarization）

## 文件说明

- `scripts/copy-dylibs.sh` - 复制动态库和重新签名的脚本
- `scripts/tauri-build-wrapper.sh` - Tauri build 的包装脚本
- `package.json` - 添加了 `tauri:build` 命令

## 未来改进

理想情况下，这个后处理步骤应该集成到 Tauri 的构建流程中。可能的方案：
1. 在 `build.rs` 中处理
2. 使用 Tauri 的插件系统
3. 贡献到 `sherpa-rs` 项目，改进其构建配置

## 验证

构建成功后，你可以验证：

1. 检查动态库是否存在：
```bash
ls -la src-tauri/target/release/bundle/macos/KeVoiceInput.app/Contents/Frameworks/
```

2. 检查签名是否有效：
```bash
codesign --verify --deep --strict --verbose=2 src-tauri/target/release/bundle/macos/KeVoiceInput.app
```

3. 测试应用启动：
```bash
open src-tauri/target/release/bundle/macos/KeVoiceInput.app
```

4. 检查运行时库依赖：
```bash
otool -L src-tauri/target/release/bundle/macos/KeVoiceInput.app/Contents/MacOS/kevoiceinput
```

## 参考

- [Tauri Bundle Configuration](https://tauri.app/v1/api/config/#bundleconfig)
- [Apple Code Signing Guide](https://developer.apple.com/documentation/security/code_signing_services)
- [dyld - Dynamic Link Editor](https://developer.apple.com/library/archive/documentation/DeveloperTools/Conceptual/DynamicLibraries/)
