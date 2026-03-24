# KeVoiceInput 构建脚本

本目录包含用于构建和发布 KeVoiceInput 的脚本。

## Windows 构建

### build-windows.ps1

PowerShell 脚本，用于在 Windows 上构建 KeVoiceInput。

**功能**:
- ✅ 前置条件检查（Rust, Bun/npm, Visual Studio）
- ✅ sherpa-onnx 库验证（可从 `%LOCALAPPDATA%\sherpa-rs\...` 缓存自动解析 `SHERPA_LIB_PATH`）
- ✅ 可选清理构建产物
- ✅ 自动安装依赖
- ✅ 构建前端和后端
- ✅ 自动复制 DLL 到输出目录
- ✅ 友好的彩色输出和错误处理

**用法**:

```powershell
# 开发模式（启动 dev 服务器）
.\scripts\build-windows.ps1 -Dev

# 生产构建
.\scripts\build-windows.ps1

# 清理后重新构建
.\scripts\build-windows.ps1 -Clean

# 指定 sherpa-onnx 库路径
.\scripts\build-windows.ps1 -SherpaPath "C:\path\to\sherpa-onnx\install\bin"

# 将当前解析到的 SHERPA_LIB_PATH 写入用户环境变量（新终端生效）
.\scripts\build-windows.ps1 -PersistUserEnv
```

**参数**:

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `-Dev` | Switch | `false` | 开发模式，启动 Tauri dev 服务器 |
| `-Clean` | Switch | `false` | 构建前清理旧的构建产物 |
| `-SherpaPath` | String | `$env:SHERPA_LIB_PATH` | sherpa-onnx DLL 路径 |
| `-PersistUserEnv` | Switch | `false` | 将 `SHERPA_LIB_PATH` 写入当前用户环境（需配合已解析路径或 `-SherpaPath`） |

**环境变量**:

```powershell
# 设置 sherpa-onnx 库路径（推荐）
$env:SHERPA_LIB_PATH = "C:\sherpa-build\sherpa-onnx\install\bin"

# 或者在系统级别设置（永久）
[Environment]::SetEnvironmentVariable(
    "SHERPA_LIB_PATH",
    "C:\sherpa-build\sherpa-onnx\install\bin",
    "User"
)
```

**输出**:

生产构建输出位置：
- 可执行文件: `src-tauri\target\release\kevoiceinput.exe`
- MSI 安装包: `src-tauri\target\release\bundle\msi\KeVoiceInput_*.msi`
- DLL 文件: 自动复制到 `src-tauri\target\release\`

**前置要求**:

1. **Rust** (1.70+)
   ```powershell
   winget install Rustlang.Rustup
   ```

2. **Bun** 或 **Node.js**
   ```powershell
   winget install Oven-sh.Bun
   ```

3. **Visual Studio Build Tools** (2022)
   ```powershell
   winget install Microsoft.VisualStudio.2022.BuildTools
   ```

4. **WebView2 Runtime**
   ```powershell
   winget install Microsoft.Edge.WebView2.Runtime
   ```

5. **sherpa-onnx 库**（已编译）
   - 参见 [docs/COMPILE_SHERPA_ONNX_WINDOWS.md](../docs/COMPILE_SHERPA_ONNX_WINDOWS.md)

**故障排查**:

| 错误 | 原因 | 解决方案 |
|------|------|---------|
| "Rust not found" | 未安装 Rust | `winget install Rustlang.Rustup` |
| "Neither Bun nor npm found" | 未安装包管理器 | `winget install Oven-sh.Bun` |
| "Sherpa library path not found" | `SHERPA_LIB_PATH` 未设置 | 设置环境变量、`-SherpaPath`，或先成功构建一次后脚本会从 `%LOCALAPPDATA%\sherpa-rs` 自动解析 |
| "Missing: *.dll" | DLL 文件不完整 | 重新编译 sherpa-onnx |
| "Tauri build failed" | 编译错误 | 检查 Rust 和 VS Build Tools |

**详细文档**:

- [Windows 快速开始](../docs/WINDOWS_QUICKSTART.md)
- [Windows 适配指南](../docs/WINDOWS_PORT.md)
- [编译 sherpa-onnx (Windows)](../docs/COMPILE_SHERPA_ONNX_WINDOWS.md)

## macOS 构建

### tauri-build-wrapper.sh

Bash 脚本，包装 Tauri 构建并自动处理动态库。

**用法**:

```bash
# 通过 package.json script 调用
bun run tauri:build

# 或直接调用
./scripts/tauri-build-wrapper.sh
```

**功能**:
- 运行标准 Tauri 构建
- 调用 `copy-dylibs.sh` 复制动态库
- 验证应用可以启动

### copy-dylibs.sh

复制 sherpa-onnx 动态库到 macOS .app bundle 并重新签名。

**用法**:

```bash
./scripts/copy-dylibs.sh path/to/KeVoiceInput.app
```

**功能**:
- 从 `$SHERPA_LIB_PATH` 复制 .dylib 文件到 `Contents/Frameworks/`
- 使用 `install_name_tool` 修改库路径为 `@rpath`
- 重新签名所有组件

### post-bundle.sh

Tauri 构建后钩子，在创建 DMG 前运行。

**功能**:
- 自动调用 `copy-dylibs.sh`
- 创建带安装脚本的 DMG

### clean-port.sh

清理占用的开发服务器端口（1420）。

**用法**:

```bash
./scripts/clean-port.sh
```

## 通用脚本

### sync-version.sh

同步所有配置文件的版本号。

**用法**:

```bash
# 更新版本到 0.0.2
./scripts/sync-version.sh 0.0.2

# 更新版本到 1.0.0-beta.1
./scripts/sync-version.sh 1.0.0-beta.1
```

**更新的文件**:
- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

**验证版本格式**: 遵循语义化版本规范 (SemVer)。

### generate-latest-json.sh

生成 Tauri Updater 的 `latest.json` 文件。

**用法**:

```bash
# 为当前版本生成 latest.json
./scripts/generate-latest-json.sh

# 为特定版本生成
./scripts/generate-latest-json.sh 0.1.0
```

**功能**:
- 自动提取 CHANGELOG.md 中的 Release Notes
- 生成多平台配置（macOS, Windows, Linux）
- 计算文件签名

### release-artifacts.sh

准备发布产物（签名、压缩）。

**用法**:

```bash
./scripts/release-artifacts.sh
```

**功能**:
- 签名可执行文件和安装包
- 生成 .sig 签名文件
- 创建发布归档

## 开发工作流

### 快速开发（所有平台）

```bash
# macOS/Linux
bun run tauri:dev

# Windows
.\scripts\build-windows.ps1 -Dev
```

### 生产构建

```bash
# macOS/Linux
bun run tauri:build

# Windows
.\scripts\build-windows.ps1
```

### 发布新版本

```bash
# 1. 更新版本号
./scripts/sync-version.sh 0.1.0

# 2. 更新 CHANGELOG.md
# 手动编辑 CHANGELOG.md，添加发布说明

# 3. 构建所有平台
# macOS:
bun run tauri:build

# Windows:
.\scripts\build-windows.ps1

# Linux (在 Linux 机器上):
bun run tauri:build

# 4. 生成 latest.json
./scripts/generate-latest-json.sh 0.1.0

# 5. 创建 Git 标签
git add -A
git commit -m "chore: bump version to 0.1.0"
git tag -a v0.1.0 -m "Release 0.1.0"
git push origin main --tags

# 6. 上传到 GitHub Release
# 使用 GitHub Actions 或手动上传
```

## CI/CD

### GitHub Actions

`.github/workflows/release.yml` 自动化发布流程：

**触发条件**: 推送 `v*` 标签

**步骤**:
1. 检出代码
2. 在 macOS, Windows, Linux 上并行构建
3. 生成签名
4. 创建 GitHub Release
5. 上传构建产物
6. 生成并提交 `latest.json`

**手动触发**:
```bash
# 创建并推送标签
git tag -a v0.1.0 -m "Release 0.1.0"
git push origin v0.1.0
```

## 脚本依赖

### macOS

- `bash` 4.0+
- `install_name_tool` (Xcode Command Line Tools)
- `codesign` (Xcode Command Line Tools)
- `otool` (Xcode Command Line Tools)

### Windows

- `PowerShell` 5.1+ (Windows 10+)
- `Rust` 1.70+
- `Bun` 或 `npm`
- `Visual Studio Build Tools` 2022

### 通用

- `git`
- `Tauri CLI` (自动安装)

## 脚本权限

### macOS/Linux

```bash
# 确保脚本可执行
chmod +x scripts/*.sh
```

### Windows

```powershell
# PowerShell 执行策略
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

## 获取帮助

遇到问题？

1. 查看详细文档：
   - [docs/BUILD_GUIDE.md](../docs/BUILD_GUIDE.md)
   - [docs/WINDOWS_QUICKSTART.md](../docs/WINDOWS_QUICKSTART.md)
   - [docs/DEBUGGING.md](../docs/DEBUGGING.md)

2. 检查常见问题：
   - [docs/WINDOWS_PORT.md#常见问题](../docs/WINDOWS_PORT.md#常见问题)

3. 提交 Issue:
   - https://github.com/yourusername/KeVoiceInput/issues
