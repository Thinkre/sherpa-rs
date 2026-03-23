# Windows 下编译 sherpa-onnx 完整指南

## 概述

sherpa-onnx 是 KeVoiceInput 的核心语音识别库。本指南详细说明如何在 Windows 上从源码编译 sherpa-onnx。

## 前置要求

### 必需软件

1. **Visual Studio 2022 Build Tools** 或 **Visual Studio 2022 Community**
   - 下载: https://visualstudio.microsoft.com/downloads/
   - 必需组件:
     - ✅ Desktop development with C++
     - ✅ MSVC v143 - VS 2022 C++ x64/x86 build tools (Latest)
     - ✅ Windows 10 SDK (10.0.19041.0 or later)
     - ✅ C++ CMake tools for Windows

2. **CMake** (3.24 或更高)
   ```powershell
   winget install Kitware.CMake
   ```
   或从 https://cmake.org/download/ 下载安装程序

3. **Git**
   ```powershell
   winget install Git.Git
   ```

4. **Python** (可选，用于某些模型工具)
   ```powershell
   winget install Python.Python.3.11
   ```

### 验证安装

```powershell
# 检查 Visual Studio 编译器
cl

# 检查 CMake
cmake --version

# 检查 Git
git --version
```

如果 `cl` 命令不可用，需要启动"Developer Command Prompt for VS 2022"或运行：

```powershell
# 使用 VS 2022 环境
& "C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools\Launch-VsDevShell.ps1"
```

## 编译步骤

### 方法 1: 快速编译（推荐）

使用预设配置快速编译：

```powershell
# 1. 克隆仓库
git clone https://github.com/k2-fsa/sherpa-onnx.git
cd sherpa-onnx

# 2. 创建构建目录
mkdir build
cd build

# 3. 配置 CMake（Visual Studio 2022）
cmake -G "Visual Studio 17 2022" -A x64 `
      -DCMAKE_BUILD_TYPE=Release `
      -DCMAKE_INSTALL_PREFIX=..\install `
      -DSHERPA_ONNX_ENABLE_PYTHON=OFF `
      -DSHERPA_ONNX_ENABLE_TESTS=OFF `
      -DSHERPA_ONNX_ENABLE_CHECK=OFF `
      -DSHERPA_ONNX_ENABLE_WEBSOCKET=OFF `
      -DSHERPA_ONNX_ENABLE_TTS=OFF `
      -DSHERPA_ONNX_ENABLE_BINARY=OFF `
      -DBUILD_SHARED_LIBS=ON `
      ..

# 4. 构建（使用所有 CPU 核心）
cmake --build . --config Release --parallel

# 5. 安装到 install 目录
cmake --install . --config Release

# 6. 验证编译结果
dir ..\install\bin\*.dll
```

**预期输出** (install/bin/ 目录):
```
cargs.dll
onnxruntime.dll
sherpa-onnx-c-api.dll
sherpa-onnx-core.dll
sherpa-onnx-fst.dll
sherpa-onnx-kaldifst-core.dll
```

### 方法 2: 使用 PowerShell 脚本

创建 `build-sherpa-windows.ps1`:

```powershell
#Requires -Version 5.1
$ErrorActionPreference = "Stop"

Write-Host "╔═══════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║   Building sherpa-onnx for Windows (x64)         ║" -ForegroundColor Cyan
Write-Host "╚═══════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Configuration
$SHERPA_REPO = "https://github.com/k2-fsa/sherpa-onnx.git"
$WORK_DIR = "C:\sherpa-build"
$INSTALL_DIR = "$WORK_DIR\sherpa-onnx\install"

# Check prerequisites
Write-Host "[1/6] Checking prerequisites..." -ForegroundColor Yellow

if (-not (Get-Command cmake -ErrorAction SilentlyContinue)) {
    Write-Host "❌ CMake not found. Please install CMake." -ForegroundColor Red
    exit 1
}
Write-Host "  ✓ CMake: $(cmake --version | Select-Object -First 1)" -ForegroundColor Green

if (-not (Get-Command cl -ErrorAction SilentlyContinue)) {
    Write-Host "❌ MSVC compiler not found." -ForegroundColor Red
    Write-Host "  Please run this script from 'Developer Command Prompt for VS 2022'" -ForegroundColor Yellow
    Write-Host "  Or run: & 'C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools\Launch-VsDevShell.ps1'" -ForegroundColor Yellow
    exit 1
}
Write-Host "  ✓ MSVC compiler found" -ForegroundColor Green
Write-Host ""

# Clone repository
Write-Host "[2/6] Cloning sherpa-onnx repository..." -ForegroundColor Yellow

if (-not (Test-Path $WORK_DIR)) {
    New-Item -ItemType Directory -Path $WORK_DIR | Out-Null
}

Set-Location $WORK_DIR

if (Test-Path "sherpa-onnx") {
    Write-Host "  Repository already exists, pulling latest changes..." -ForegroundColor Gray
    Set-Location sherpa-onnx
    git pull
    Set-Location ..
} else {
    git clone $SHERPA_REPO
}
Write-Host "  ✓ Repository ready" -ForegroundColor Green
Write-Host ""

# Create build directory
Write-Host "[3/6] Preparing build directory..." -ForegroundColor Yellow

Set-Location sherpa-onnx

if (Test-Path "build") {
    Write-Host "  Removing existing build directory..." -ForegroundColor Gray
    Remove-Item -Recurse -Force build
}

New-Item -ItemType Directory -Path build | Out-Null
Set-Location build
Write-Host "  ✓ Build directory ready" -ForegroundColor Green
Write-Host ""

# Configure CMake
Write-Host "[4/6] Configuring CMake..." -ForegroundColor Yellow

$cmakeArgs = @(
    "-G", "Visual Studio 17 2022",
    "-A", "x64",
    "-DCMAKE_BUILD_TYPE=Release",
    "-DCMAKE_INSTALL_PREFIX=$INSTALL_DIR",
    "-DSHERPA_ONNX_ENABLE_PYTHON=OFF",
    "-DSHERPA_ONNX_ENABLE_TESTS=OFF",
    "-DSHERPA_ONNX_ENABLE_CHECK=OFF",
    "-DSHERPA_ONNX_ENABLE_WEBSOCKET=OFF",
    "-DSHERPA_ONNX_ENABLE_TTS=OFF",
    "-DSHERPA_ONNX_ENABLE_BINARY=OFF",
    "-DBUILD_SHARED_LIBS=ON",
    ".."
)

& cmake @cmakeArgs

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ CMake configuration failed" -ForegroundColor Red
    exit 1
}
Write-Host "  ✓ CMake configuration complete" -ForegroundColor Green
Write-Host ""

# Build
Write-Host "[5/6] Building (this may take 5-15 minutes)..." -ForegroundColor Yellow

$cores = (Get-CimInstance Win32_ComputerSystem).NumberOfLogicalProcessors
Write-Host "  Using $cores CPU cores" -ForegroundColor Gray

cmake --build . --config Release --parallel $cores

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed" -ForegroundColor Red
    exit 1
}
Write-Host "  ✓ Build complete" -ForegroundColor Green
Write-Host ""

# Install
Write-Host "[6/6] Installing to $INSTALL_DIR..." -ForegroundColor Yellow

cmake --install . --config Release

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Installation failed" -ForegroundColor Red
    exit 1
}
Write-Host "  ✓ Installation complete" -ForegroundColor Green
Write-Host ""

# Summary
Write-Host "╔═══════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║              Build Successful!                    ║" -ForegroundColor Cyan
Write-Host "╚═══════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""
Write-Host "Installation directory:" -ForegroundColor Yellow
Write-Host "  $INSTALL_DIR" -ForegroundColor Green
Write-Host ""
Write-Host "DLL files:" -ForegroundColor Yellow
Get-ChildItem "$INSTALL_DIR\bin\*.dll" | ForEach-Object {
    Write-Host "  ✓ $($_.Name)" -ForegroundColor Green
}
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Set environment variable:" -ForegroundColor Gray
Write-Host "     `$env:SHERPA_LIB_PATH = `"$INSTALL_DIR\bin`"" -ForegroundColor Gray
Write-Host "  2. Build KeVoiceInput:" -ForegroundColor Gray
Write-Host "     cd C:\path\to\KeVoiceInput" -ForegroundColor Gray
Write-Host "     .\scripts\build-windows.ps1" -ForegroundColor Gray
Write-Host ""
```

运行脚本:
```powershell
# 从 Developer Command Prompt 运行
powershell -ExecutionPolicy Bypass -File build-sherpa-windows.ps1
```

## CMake 配置选项详解

| 选项 | 值 | 说明 |
|------|-----|------|
| `-G "Visual Studio 17 2022"` | - | 使用 VS 2022 生成器 |
| `-A x64` | - | 64 位架构 |
| `CMAKE_BUILD_TYPE` | `Release` | 发布版本（优化） |
| `CMAKE_INSTALL_PREFIX` | 路径 | 安装目标目录 |
| `SHERPA_ONNX_ENABLE_PYTHON` | `OFF` | 不需要 Python 绑定 |
| `SHERPA_ONNX_ENABLE_TESTS` | `OFF` | 不编译测试 |
| `SHERPA_ONNX_ENABLE_CHECK` | `OFF` | 跳过额外检查（加速） |
| `SHERPA_ONNX_ENABLE_WEBSOCKET` | `OFF` | 不需要 WebSocket 支持 |
| `SHERPA_ONNX_ENABLE_TTS` | `OFF` | 不需要 TTS（文本转语音） |
| `SHERPA_ONNX_ENABLE_BINARY` | `OFF` | 不编译示例程序 |
| `BUILD_SHARED_LIBS` | `ON` | 编译动态链接库（DLL） |

### 可选配置

**启用 GPU 加速** (需要 CUDA):
```powershell
cmake -G "Visual Studio 17 2022" -A x64 `
      -DCMAKE_BUILD_TYPE=Release `
      -DCMAKE_INSTALL_PREFIX=..\install `
      -DSHERPA_ONNX_ENABLE_GPU=ON `
      -DBUILD_SHARED_LIBS=ON `
      ..
```

**启用 DirectML** (Windows GPU 加速):
```powershell
cmake -G "Visual Studio 17 2022" -A x64 `
      -DCMAKE_BUILD_TYPE=Release `
      -DCMAKE_INSTALL_PREFIX=..\install `
      -DSHERPA_ONNX_ENABLE_DIRECTML=ON `
      -DBUILD_SHARED_LIBS=ON `
      ..
```

**静态链接** (不推荐，会增加可执行文件大小):
```powershell
cmake -G "Visual Studio 17 2022" -A x64 `
      -DCMAKE_BUILD_TYPE=Release `
      -DCMAKE_INSTALL_PREFIX=..\install `
      -DBUILD_SHARED_LIBS=OFF `
      -DSHERPA_STATIC_CRT=ON `
      ..
```

## 编译产物

### 目录结构

成功编译后，`install/` 目录结构：

```
install/
├── bin/                    # DLL 文件
│   ├── cargs.dll
│   ├── onnxruntime.dll
│   ├── sherpa-onnx-c-api.dll
│   ├── sherpa-onnx-core.dll
│   ├── sherpa-onnx-fst.dll
│   └── sherpa-onnx-kaldifst-core.dll
├── lib/                    # 导入库（.lib）
│   ├── cargs.lib
│   ├── onnxruntime.lib
│   ├── sherpa-onnx-c-api.lib
│   ├── sherpa-onnx-core.lib
│   ├── sherpa-onnx-fst.lib
│   └── sherpa-onnx-kaldifst-core.lib
└── include/                # 头文件
    └── sherpa-onnx/
        └── c-api/
            └── c-api.h
```

### 文件说明

| 文件 | 大小 | 说明 |
|------|------|------|
| `onnxruntime.dll` | ~40MB | ONNX Runtime 推理引擎 |
| `sherpa-onnx-c-api.dll` | ~2MB | sherpa-onnx C API |
| `sherpa-onnx-core.dll` | ~5MB | sherpa-onnx 核心功能 |
| `cargs.dll` | <1MB | 命令行参数解析库 |
| `sherpa-onnx-fst.dll` | ~1MB | 有限状态转换器 |
| `sherpa-onnx-kaldifst-core.dll` | ~2MB | Kaldi FST 核心 |

## 配置 KeVoiceInput

### 设置环境变量

**临时设置** (仅当前 PowerShell 会话):
```powershell
$env:SHERPA_LIB_PATH = "C:\sherpa-build\sherpa-onnx\install\bin"
```

**永久设置** (用户级别):
```powershell
[Environment]::SetEnvironmentVariable(
    "SHERPA_LIB_PATH",
    "C:\sherpa-build\sherpa-onnx\install\bin",
    "User"
)
```

**永久设置** (系统级别，需要管理员权限):
```powershell
[Environment]::SetEnvironmentVariable(
    "SHERPA_LIB_PATH",
    "C:\sherpa-build\sherpa-onnx\install\bin",
    "Machine"
)
```

### 验证配置

```powershell
# 检查环境变量
echo $env:SHERPA_LIB_PATH

# 检查 DLL 文件
dir $env:SHERPA_LIB_PATH\*.dll

# 测试加载 DLL (使用 Dependencies.exe)
# 下载: https://github.com/lucasg/Dependencies
dependencies.exe "$env:SHERPA_LIB_PATH\sherpa-onnx-c-api.dll"
```

## 常见问题

### Q1: CMake 找不到 Visual Studio

**错误**:
```
CMake Error: CMake was unable to find a build program corresponding to "Visual Studio 17 2022"
```

**解决方案**:
1. 确保安装了 Visual Studio 2022 或 Build Tools
2. 从"Developer Command Prompt for VS 2022"运行 CMake
3. 或手动指定编译器路径:
   ```powershell
   $env:CC = "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.XX.XXXXX\bin\Hostx64\x64\cl.exe"
   $env:CXX = $env:CC
   ```

### Q2: 编译时内存不足

**错误**:
```
LINK : fatal error LNK1104: cannot open file '...'
```

**解决方案**:
- 减少并行编译任务:
  ```powershell
  cmake --build . --config Release --parallel 2
  ```
- 关闭其他占用内存的程序
- 增加虚拟内存（页面文件）

### Q3: onnxruntime.dll 版本不匹配

**问题**: 运行时提示 ONNX Runtime 版本不匹配。

**解决方案**:
- 使用 sherpa-onnx 自带的 ONNX Runtime（推荐）
- 或从 https://github.com/microsoft/onnxruntime/releases 下载匹配版本

### Q4: 缺少 vcruntime140.dll

**错误**: 应用启动时提示缺少 `vcruntime140.dll`。

**解决方案**:
安装 Visual C++ Redistributable:
```powershell
winget install Microsoft.VCRedist.2015+.x64
```

### Q5: 编译速度慢

**优化方法**:
1. 使用所有 CPU 核心:
   ```powershell
   cmake --build . --config Release --parallel
   ```

2. 使用 Ninja 生成器（需安装 Ninja）:
   ```powershell
   winget install Ninja-build.Ninja

   cmake -G Ninja `
         -DCMAKE_BUILD_TYPE=Release `
         -DCMAKE_INSTALL_PREFIX=..\install `
         -DBUILD_SHARED_LIBS=ON `
         ..

   cmake --build . --parallel
   ```

3. 禁用不需要的功能（见配置选项）

### Q6: Git 克隆失败

**问题**: 网络问题导致 Git 克隆失败。

**解决方案**:
```powershell
# 使用国内镜像
git clone https://gitee.com/mirrors/sherpa-onnx.git

# 或使用浅克隆（更快）
git clone --depth 1 https://github.com/k2-fsa/sherpa-onnx.git
```

## 编译不同版本

### 编译 Debug 版本

用于开发和调试：

```powershell
cmake -G "Visual Studio 17 2022" -A x64 `
      -DCMAKE_BUILD_TYPE=Debug `
      -DCMAKE_INSTALL_PREFIX=..\install-debug `
      -DBUILD_SHARED_LIBS=ON `
      ..

cmake --build . --config Debug
cmake --install . --config Debug
```

### 编译特定提交/标签

```powershell
git clone https://github.com/k2-fsa/sherpa-onnx.git
cd sherpa-onnx

# 切换到特定标签
git checkout v1.9.0

# 或切换到特定提交
git checkout abc123def

# 然后正常编译
mkdir build && cd build
cmake ...
```

## 与 KeVoiceInput 集成

### 构建 KeVoiceInput

编译 sherpa-onnx 后，构建 KeVoiceInput：

```powershell
cd C:\path\to\KeVoiceInput

# 设置环境变量
$env:SHERPA_LIB_PATH = "C:\sherpa-build\sherpa-onnx\install\bin"

# 运行构建脚本
.\scripts\build-windows.ps1
```

### 验证集成

```powershell
# 检查 DLL 是否被复制
dir .\src-tauri\target\release\*.dll | Where-Object { $_.Name -like "*sherpa*" -or $_.Name -like "*onnxruntime*" }

# 运行应用
.\src-tauri\target\release\kevoiceinput.exe
```

## 性能基准

典型编译时间（Intel i7-12700 / 32GB RAM）:

| 配置 | 时间 | 说明 |
|------|------|------|
| 首次完整编译 | ~10 分钟 | 包括下载依赖 |
| 增量编译 | ~2 分钟 | 仅修改少量文件 |
| 清理后重新编译 | ~8 分钟 | 不下载依赖 |

磁盘空间使用：

- 源代码: ~200MB
- 构建目录: ~800MB
- 安装目录: ~50MB

## 进阶主题

### 交叉编译 ARM64

Windows ARM64 设备（如 Surface Pro X）:

```powershell
cmake -G "Visual Studio 17 2022" -A ARM64 `
      -DCMAKE_BUILD_TYPE=Release `
      -DCMAKE_INSTALL_PREFIX=..\install-arm64 `
      -DBUILD_SHARED_LIBS=ON `
      ..
```

### 自定义 ONNX Runtime

使用自己编译的 ONNX Runtime:

```powershell
cmake -G "Visual Studio 17 2022" -A x64 `
      -DCMAKE_BUILD_TYPE=Release `
      -DCMAKE_INSTALL_PREFIX=..\install `
      -DONNXRUNTIME_DIR=C:\path\to\custom\onnxruntime `
      -DBUILD_SHARED_LIBS=ON `
      ..
```

### 启用编译器优化

```powershell
$env:CXXFLAGS = "/O2 /GL /arch:AVX2"
cmake ...
```

## 总结

✅ **sherpa-onnx Windows 编译完全可行**

- 使用 Visual Studio 2022 + CMake
- 编译时间约 10 分钟
- 生成标准 DLL 文件
- 与 KeVoiceInput 无缝集成

**关键步骤**:
1. 安装 Visual Studio Build Tools
2. 使用 CMake 配置和构建
3. 设置 `SHERPA_LIB_PATH` 环境变量
4. 构建 KeVoiceInput

## 相关链接

- [sherpa-onnx GitHub](https://github.com/k2-fsa/sherpa-onnx)
- [sherpa-onnx 官方文档](https://k2-fsa.github.io/sherpa/onnx/)
- [Visual Studio 下载](https://visualstudio.microsoft.com/downloads/)
- [CMake 下载](https://cmake.org/download/)
- [ONNX Runtime](https://onnxruntime.ai/)

## 获取帮助

遇到问题？

1. 查看 [WINDOWS_QUICKSTART.md](WINDOWS_QUICKSTART.md)
2. 查看 [DYNAMIC_LIBRARY_LOADING.md](DYNAMIC_LIBRARY_LOADING.md)
3. 提交 Issue: https://github.com/yourusername/KeVoiceInput/issues
