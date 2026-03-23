# KeVoiceInput 构建指南

本文档提供 KeVoiceInput 在不同平台上的构建说明。

## 目录

- [快速开始](#快速开始)
- [macOS 构建](#macos-构建)
- [Windows 构建](#windows-构建)
- [构建产物](#构建产物)
- [已知问题和解决方案](#已知问题和解决方案)

## 快速开始

### 开发模式

```bash
bun run tauri:dev
```

### 生产构建

```bash
bun run tauri:build
```

## macOS 构建

### 生产构建

```bash
bun run tauri:build
```

这个命令会自动：
1. 构建前端和后端
2. 创建 macOS App Bundle
3. 复制所有必需的动态链接库
4. 重新签名应用
5. 创建包含修复后 app 的 DMG 安装包
6. 测试应用能否启动

### 构建产物

构建成功后，你会在以下位置找到产物：

- **App Bundle**: `src-tauri/target/release/bundle/macos/KeVoiceInput.app`（可直接运行）
- **DMG 安装包**: `src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg`（~34MB，包含所有动态库）

## Windows 构建

> **注意**: Windows 支持目前处于适配阶段。详细的 Windows 快速开始指南请参考 [WINDOWS_QUICKSTART.md](WINDOWS_QUICKSTART.md)。

### 前置要求

1. **Rust** (最新稳定版)
   ```powershell
   winget install Rustlang.Rustup
   ```

2. **Node.js 和 Bun**
   ```powershell
   winget install Oven-sh.Bun
   ```

3. **Visual Studio Build Tools**
   ```powershell
   winget install Microsoft.VisualStudio.2022.BuildTools
   ```
   安装时选择：
   - ✓ Desktop development with C++
   - ✓ MSVC v143 或更高
   - ✓ Windows 10 SDK

4. **WebView2**
   ```powershell
   winget install Microsoft.Edge.WebView2.Runtime
   ```

### 环境变量

设置 sherpa-onnx 库路径：
```powershell
$env:SHERPA_LIB_PATH = "C:\path\to\sherpa-onnx\install\bin"
```

### 使用构建脚本（推荐）

```powershell
# 开发模式（带热重载）
.\scripts\build-windows.ps1 -Dev

# 生产构建
.\scripts\build-windows.ps1

# 清理后构建
.\scripts\build-windows.ps1 -Clean
```

### 手动构建

```powershell
# 安装依赖
bun install

# 构建前端
bun run build

# 构建 Tauri 应用
bun run tauri build
```

### 构建产物

构建成功后，你会在以下位置找到产物：

- **可执行文件**: `src-tauri\target\release\kevoiceinput.exe`
- **MSI 安装包**: `src-tauri\target\release\bundle\msi\KeVoiceInput_*.msi`

### Windows 特定配置

#### DLL 依赖

Windows 版本需要以下 DLL 文件（会自动从 `SHERPA_LIB_PATH` 复制）：
- `sherpa-onnx-c-api.dll`
- `sherpa-onnx-cxx-api.dll`
- `onnxruntime.dll`

#### 编译 sherpa-onnx（可选）

如果需要从源码编译 sherpa-onnx，请参考 [WINDOWS_QUICKSTART.md](WINDOWS_QUICKSTART.md) 中的完整编译步骤。

## 已知问题和解决方案

### 问题：应用崩溃显示"意外退出"

**原因**: Sherpa-ONNX 的动态链接库没有被正确打包到 app bundle 中。

**解决方案**: 现在已经通过 `tauri:build` 命令自动解决。该命令使用包装脚本 `scripts/tauri-build-wrapper.sh` 来：

1. 运行标准 Tauri 构建
2. 复制所需的动态库到 `Contents/Frameworks/`：
   - libcargs.dylib
   - libonnxruntime.1.17.1.dylib
   - libsherpa-onnx-c-api.dylib
   - libsherpa-onnx-cxx-api.dylib
3. 更新二进制的 rpath 引用
4. 重新签名所有组件

### 手动修复已构建的应用

如果你使用了 `tauri build` 而不是 `tauri:build`，可以手动运行修复脚本：

```bash
./scripts/copy-dylibs.sh src-tauri/target/release/bundle/macos/KeVoiceInput.app
```

## 验证构建

### macOS

#### 1. 检查动态库

```bash
ls -la src-tauri/target/release/bundle/macos/KeVoiceInput.app/Contents/Frameworks/
```

应该看到 4 个 .dylib 文件。

#### 2. 验证代码签名

```bash
codesign --verify --deep --strict --verbose=2 \
  src-tauri/target/release/bundle/macos/KeVoiceInput.app
```

应该显示签名有效。

#### 3. 检查库依赖

```bash
otool -L src-tauri/target/release/bundle/macos/KeVoiceInput.app/Contents/MacOS/kevoiceinput
```

所有 sherpa 相关的库应该指向 `@executable_path/../Frameworks/`。

#### 4. 测试启动

```bash
open src-tauri/target/release/bundle/macos/KeVoiceInput.app
```

应用应该能正常启动。

### Windows

#### 1. 检查 DLL 文件

```powershell
dir src-tauri\target\release\*.dll
```

应该看到所有必需的 DLL 文件。

#### 2. 测试启动

```powershell
.\src-tauri\target\release\kevoiceinput.exe
```

或安装 MSI 后测试：
```powershell
.\src-tauri\target\release\bundle\msi\KeVoiceInput_*.msi
```

#### 3. 检查依赖（可选）

使用 [Dependencies](https://github.com/lucasg/Dependencies) 工具检查 DLL 依赖：
```powershell
# 下载并运行 Dependencies
dependencies.exe .\src-tauri\target\release\kevoiceinput.exe
```

## 发布构建

### 本地测试构建（当前方式）

```bash
bun run tauri:build
```

生成的应用使用 adhoc 签名（`-`），只能在本地 Mac 上运行。

### 正式发布构建

要发布给其他用户，需要：

1. **Apple Developer 账号**
2. **配置签名证书**：
   ```json
   // tauri.conf.json
   "macOS": {
     "signingIdentity": "Developer ID Application: Your Name (TEAM_ID)"
   }
   ```
3. **配置公证**：
   设置环境变量：
   - `APPLE_ID`
   - `APPLE_PASSWORD`（应用专用密码）
   - `APPLE_TEAM_ID`

4. **构建并公证**：
   ```bash
   # 可选：对 DMG 内的 Install.command 签名，避免其他 Mac 上出现“无法验证恶意软件”提示
   export MACOS_INSTALLER_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)"
   bun run tauri:build
   ```

   公证会自动进行，成功后生成的 DMG 可以在任何 Mac 上安装。若设置了 `MACOS_INSTALLER_SIGNING_IDENTITY`，安装脚本也会被签名，用户在其他 Mac 上双击 Install.command 时不再被 Gatekeeper 阻止。

## 创建 DMG 安装镜像

如果需要手动创建 DMG：

```bash
cd src-tauri/target/release/bundle/dmg
hdiutil create -volname "KeVoiceInput" \
  -srcfolder "../macos/KeVoiceInput.app" \
  -ov -format UDZO \
  KeVoiceInput_0.0.1_aarch64.dmg
```

## 常见问题

### 通用问题

#### Q: 为什么构建这么慢？

A: Rust 编译本身就慢，sherpa-onnx 是个大型库。首次构建可能需要 5-10 分钟。后续增量构建会快很多。

#### Q: 能否跳过前端构建？

A: 如果只修改了 Rust 代码，可以：

**macOS/Linux**:
```bash
cd src-tauri && cargo build --release
```

**Windows**:
```powershell
cd src-tauri; cargo build --release
```

但这不会生成应用包，只生成二进制文件。

#### Q: 如何清理构建产物？

**macOS/Linux**:
```bash
# 清理前端
rm -rf dist/

# 清理 Rust
cd src-tauri && cargo clean
```

**Windows**:
```powershell
# 清理前端
Remove-Item -Recurse -Force dist

# 清理 Rust
cd src-tauri; cargo clean
```

#### Q: 构建后应用占用空间很大

A: 是的，因为：
- ONNX Runtime 库本身就有 ~48MB (macOS) / ~40MB (Windows)
- Debug 符号占用空间
- 未压缩的资源文件

可以通过以下方式减小：
1. Strip debug symbols（已在 release 模式自动执行）
2. 使用 `upx` 压缩二进制（可能影响签名）
3. 将大型模型文件放在外部下载

### macOS 特定问题

#### Q: 应用崩溃显示"意外退出"

**原因**: Sherpa-ONNX 的动态链接库没有被正确打包到 app bundle 中。

**解决方案**: 使用 `bun run tauri:build` 而不是 `tauri build`。或手动运行修复脚本：
```bash
./scripts/copy-dylibs.sh src-tauri/target/release/bundle/macos/KeVoiceInput.app
```

### Windows 特定问题

#### Q: 提示找不到 DLL 文件

**原因**: sherpa-onnx DLL 文件不在系统 PATH 或可执行文件目录。

**解决方案**:
```powershell
# 方法 1: 设置环境变量
$env:SHERPA_LIB_PATH = "C:\path\to\sherpa-onnx\install\bin"

# 方法 2: 手动复制 DLL
Copy-Item "$env:SHERPA_LIB_PATH\*.dll" -Destination ".\src-tauri\target\release\"
```

#### Q: Rust 编译错误：link.exe not found

**原因**: 未安装 Visual Studio Build Tools 或未找到 MSVC。

**解决方案**:
```powershell
# 安装 Visual Studio Build Tools
winget install Microsoft.VisualStudio.2022.BuildTools

# 或设置环境变量
$env:RUSTFLAGS = "-C target-feature=+crt-static"
```

#### Q: WebView2 错误

**原因**: WebView2 Runtime 未安装。

**解决方案**:
```powershell
winget install Microsoft.Edge.WebView2.Runtime
```

详细的 Windows 问题排查请参考 [WINDOWS_QUICKSTART.md](WINDOWS_QUICKSTART.md)。

## 脚本说明

- `scripts/copy-dylibs.sh` - 复制动态库和重新签名
- `scripts/tauri-build-wrapper.sh` - Tauri 构建包装脚本
- `scripts/clean-port.sh` - 清理开发服务器端口

## 技术细节

详细的技术说明请参考：[BUILD_FIX.md](BUILD_FIX.md)

## 参考资料

- [Tauri 文档](https://tauri.app/)
- [Apple 代码签名指南](https://developer.apple.com/documentation/security/code_signing_services)
- [sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx)
