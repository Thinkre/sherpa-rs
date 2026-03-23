# Windows CI/CD 构建指南

## 概述

本文档说明如何在 GitHub Actions 中构建 Windows 版本的 KeVoiceInput。

## sherpa-onnx 库依赖

### 问题

Windows 构建需要 sherpa-onnx 动态库（DLL）。与 macOS/Linux 不同，Windows 需要在构建时提供这些库。

### 解决方案

有三种方式在 CI/CD 中处理 sherpa-onnx 依赖：

#### 方案 1：使用预编译二进制文件（推荐）

**优点**：快速、可靠、可重复
**缺点**：需要维护二进制文件存储

```yaml
- name: Download sherpa-onnx pre-built binaries (Windows)
  if: matrix.platform == 'windows-latest'
  shell: powershell
  run: |
    # 从 GitHub Release 或其他存储下载预编译的 DLL
    $SherpaVersion = "1.10.0"
    $SherpaUrl = "https://github.com/k2-fsa/sherpa-onnx/releases/download/v${SherpaVersion}/sherpa-onnx-v${SherpaVersion}-win-x64-shared.tar.bz2"

    # 下载并解压
    Invoke-WebRequest -Uri $SherpaUrl -OutFile "sherpa-onnx.tar.bz2"
    tar -xf sherpa-onnx.tar.bz2

    # 设置环境变量
    $SherpaPath = "$PWD\sherpa-onnx-v${SherpaVersion}-win-x64-shared\lib"
    echo "SHERPA_LIB_PATH=$SherpaPath" >> $env:GITHUB_ENV
```

#### 方案 2：从源代码编译

**优点**：完全控制、最新版本
**缺点**：编译时间长（~10 分钟）

```yaml
- name: Build sherpa-onnx from source (Windows)
  if: matrix.platform == 'windows-latest'
  shell: powershell
  run: |
    # 克隆 sherpa-onnx
    git clone --depth 1 https://github.com/k2-fsa/sherpa-onnx.git
    cd sherpa-onnx

    # 配置和编译
    mkdir build
    cd build
    cmake -G "Visual Studio 17 2022" -A x64 `
          -DCMAKE_BUILD_TYPE=Release `
          -DCMAKE_INSTALL_PREFIX=../install `
          -DSHERPA_ONNX_ENABLE_PYTHON=OFF `
          -DSHERPA_ONNX_ENABLE_TESTS=OFF `
          -DSHERPA_ONNX_ENABLE_CHECK=OFF `
          -DSHERPA_ONNX_ENABLE_WEBSOCKET=OFF `
          -DSHERPA_ONNX_ENABLE_TTS=OFF `
          -DSHERPA_ONNX_ENABLE_BINARY=OFF `
          -DBUILD_SHARED_LIBS=ON `
          ..

    cmake --build . --config Release --parallel
    cmake --install . --config Release

    # 设置环境变量
    $SherpaPath = "$PWD\..\install\bin"
    echo "SHERPA_LIB_PATH=$SherpaPath" >> $env:GITHUB_ENV
```

#### 方案 3：使用 GitHub Release Assets

**优点**：版本化、易于回滚
**缺点**：需要首次手动上传

```yaml
- name: Download sherpa-onnx from GitHub Release (Windows)
  if: matrix.platform == 'windows-latest'
  shell: powershell
  run: |
    # 从项目自己的 Release 下载预编译的 sherpa-onnx
    $SherpaAsset = "sherpa-onnx-windows-x64-v1.10.0.zip"
    $AssetUrl = "https://github.com/${{ github.repository }}/releases/download/sherpa-deps/${SherpaAsset}"

    Invoke-WebRequest -Uri $AssetUrl -OutFile "sherpa.zip"
    Expand-Archive -Path "sherpa.zip" -DestinationPath "sherpa-onnx"

    $SherpaPath = "$PWD\sherpa-onnx\bin"
    echo "SHERPA_LIB_PATH=$SherpaPath" >> $env:GITHUB_ENV
```

## 当前 Workflow 配置

`.github/workflows/release.yml` 已配置 Windows 构建：

```yaml
matrix:
  include:
    - platform: windows-latest
      target: x86_64-pc-windows-msvc
      arch: x86_64
```

### Windows 特定步骤

1. **Setup Windows dependencies**：验证构建工具
2. **Prepare artifacts (Windows)**：使用 PowerShell 打包 MSI
3. **Upload release artifacts**：上传到 GitHub Release

### 产物

Windows 构建生成以下文件：
- `KeVoiceInput-{version}-windows-x64.msi` - 安装包
- `KeVoiceInput-{version}-windows-x64.msi.zip` - 用于 Tauri Updater
- `KeVoiceInput-{version}-windows-x64.msi.zip.sig` - 签名文件

## 构建触发

推送版本标签时自动触发：

```bash
git tag -a v0.0.2 -m "Release 0.0.2"
git push origin v0.0.2
```

## 手动触发

可以使用 GitHub Actions 的 `workflow_dispatch` 手动触发特定平台构建：

```yaml
on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:
    inputs:
      platform:
        description: 'Platform to build (all, windows, macos, linux)'
        required: false
        default: 'all'
```

## 故障排查

### DLL 未找到

**错误**：
```
error: failed to run custom build command for `sherpa-rs-sys`
  couldn't find sherpa-onnx-c-api.dll
```

**解决方案**：
1. 确保 `SHERPA_LIB_PATH` 环境变量已正确设置
2. 检查 DLL 文件是否存在于该路径
3. 验证路径格式（Windows 使用反斜杠）

### MSI 打包失败

**错误**：
```
error: failed to bundle project: error running wix candle
```

**解决方案**：
1. 检查 WiX Toolset 是否安装（GitHub windows-latest 已包含）
2. 验证 `tauri.conf.json` 中的 Windows 配置
3. 确保应用图标路径正确

### 签名失败

**错误**：
```
Error: TAURI_PRIVATE_KEY not set
```

**解决方案**：
1. 在 GitHub Repository Settings → Secrets 中添加 `TAURI_SIGNING_PRIVATE_KEY`
2. 添加 `TAURI_KEY_PASSWORD`（如果私钥有密码）

## 本地测试 CI 构建

可以使用 [act](https://github.com/nektos/act) 在本地测试 GitHub Actions：

```bash
# 安装 act
winget install nektos.act

# 测试 Windows 构建
act push -P windows-latest=-self-hosted
```

注意：需要 Docker Desktop 运行。

## 加速构建

### 缓存依赖

```yaml
- name: Cache Rust dependencies
  uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      src-tauri/target
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

- name: Cache Bun dependencies
  uses: actions/cache@v4
  with:
    path: ~/.bun/install/cache
    key: ${{ runner.os }}-bun-${{ hashFiles('**/bun.lockb') }}
```

### 并行构建

```yaml
strategy:
  fail-fast: false  # 不要因为一个平台失败而取消其他平台
  max-parallel: 3   # 最多同时运行 3 个平台
```

## 参考资料

- [Tauri 构建配置](https://tauri.app/v2/reference/config/)
- [GitHub Actions Windows](https://docs.github.com/en/actions/using-github-hosted-runners/about-github-hosted-runners#supported-runners-and-hardware-resources)
- [WiX Toolset 文档](https://wixtoolset.org/docs/)
- [sherpa-onnx 官方文档](https://k2-fsa.github.io/sherpa/onnx/)

## 相关文档

- [COMPILE_SHERPA_ONNX_WINDOWS.md](COMPILE_SHERPA_ONNX_WINDOWS.md) - Windows 编译 sherpa-onnx 详细指南
- [BUILD_GUIDE.md](BUILD_GUIDE.md) - 通用构建指南
- [WINDOWS_QUICKSTART.md](WINDOWS_QUICKSTART.md) - Windows 快速开始
