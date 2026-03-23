# KeVoiceInput Windows 快速开始

本指南帮助Windows 用户快速构建和运行 KeVoiceInput。

## 前置条件

### 必需软件

1. **Rust** (最新稳定版)
   ```powershell
   winget install Rustlang.Rustup
   # 或访问 https://rustup.rs/
   ```

2. **Node.js** 和 **Bun**
   ```powershell
   # 安装 Bun (推荐)
   winget install Oven-sh.Bun

   # 或安装 Node.js
   winget install OpenJS.NodeJS
   ```

3. **Visual Studio Build Tools**
   ```powershell
   winget install Microsoft.VisualStudio.2022.BuildTools
   ```

   安装时选择：
   - ✓ Desktop development with C++
   - ✓ MSVC v143 或更高
   - ✓ Windows 10 SDK

4. **WebView2** (通常已预装在 Windows 11)
   ```powershell
   winget install Microsoft.Edge.WebView2.Runtime
   ```

### 可选软件

- **CMake** (用于编译 sherpa-onnx)
  ```powershell
  winget install Kitware.CMake
  ```

- **Git**
  ```powershell
  winget install Git.Git
  ```

## 快速构建（使用预编译库）

如果你有预编译的 sherpa-onnx 库：

### Step 1: 克隆仓库

```powershell
git clone https://github.com/yourusername/KeVoiceInput.git
cd KeVoiceInput
```

### Step 2: 设置环境变量

```powershell
# 设置 sherpa-onnx 库路径（替换为实际路径）
$env:SHERPA_LIB_PATH = "C:\path\to\sherpa-onnx\install\bin"
```

### Step 3: 运行构建脚本

```powershell
# 开发模式（带热重载）
.\scripts\build-windows.ps1 -Dev

# 生产构建
.\scripts\build-windows.ps1

# 清理后构建
.\scripts\build-windows.ps1 -Clean
```

### Step 4: 运行应用

```powershell
# 运行可执行文件
.\src-tauri\target\release\kevoiceinput.exe

# 或安装 MSI
.\src-tauri\target\release\bundle\msi\KeVoiceInput_*.msi
```

## 完整构建（从源码编译 sherpa-onnx）

### Step 1: 安装依赖

确保已安装 Visual Studio Build Tools 和 CMake。

### Step 2: 编译 sherpa-onnx

```powershell
# 创建工作目录
mkdir C:\workspace
cd C:\workspace

# 克隆 sherpa-onnx
git clone https://github.com/k2-fsa/sherpa-onnx.git
cd sherpa-onnx

# 创建构建目录
mkdir build
cd build

# CMake 配置 (Visual Studio 2022)
cmake -G "Visual Studio 17 2022" -A x64 `
      -DCMAKE_BUILD_TYPE=Release `
      -DCMAKE_INSTALL_PREFIX=..\install `
      -DSHERPA_ONNX_ENABLE_PYTHON=OFF `
      -DSHERPA_ONNX_ENABLE_TESTS=OFF `
      -DSHERPA_ONNX_ENABLE_CHECK=OFF `
      -DBUILD_SHARED_LIBS=ON `
      ..

# 构建 (大约 5-10 分钟)
cmake --build . --config Release

# 安装到 install 目录
cmake --install . --config Release
```

### Step 3: 设置环境变量

```powershell
# 添加到系统环境变量（永久）
[Environment]::SetEnvironmentVariable(
    "SHERPA_LIB_PATH",
    "C:\workspace\sherpa-onnx\install\bin",
    "User"
)

# 或仅当前会话
$env:SHERPA_LIB_PATH = "C:\workspace\sherpa-onnx\install\bin"
```

### Step 4: 构建 KeVoiceInput

```powershell
cd C:\path\to\KeVoiceInput
.\scripts\build-windows.ps1
```

## 开发工作流

### 1. 启动开发模式

```powershell
# 使用 Bun
bun run tauri:dev

# 或使用 npm
npm run tauri:dev
```

应用会自动打开，前端代码更改会热重载。

### 2. 修改代码

- **前端**: 编辑 `src/` 目录中的 React 组件
- **后端**: 编辑 `src-tauri/src/` 目录中的 Rust 代码

### 3. 代码格式化

```powershell
# 格式化所有代码
bun run format

# 仅格式化前端
bun run format:frontend

# 仅格式化后端
bun run format:backend
```

### 4. 代码检查

```powershell
# ESLint
bun run lint

# 自动修复
bun run lint:fix
```

## 常见问题

### 问题 1: Rust 编译错误

**错误**: `link.exe not found` 或 `MSVC not found`

**解决**:
```powershell
# 确保安装了 Visual Studio Build Tools
winget install Microsoft.VisualStudio.2022.BuildTools

# 或设置环境变量
$env:RUSTFLAGS = "-C target-feature=+crt-static"
```

### 问题 2: sherpa-onnx DLL 未找到

**错误**: 应用启动时提示缺少 DLL

**解决**:
```powershell
# 检查 DLL 是否在正确位置
dir $env:SHERPA_LIB_PATH\*.dll

# 手动复制 DLL 到可执行文件目录
Copy-Item "$env:SHERPA_LIB_PATH\*.dll" -Destination ".\src-tauri\target\release\"
```

### 问题 3: WebView2 错误

**错误**: `WebView2 runtime not found`

**解决**:
```powershell
# 安装 WebView2 Runtime
winget install Microsoft.Edge.WebView2.Runtime

# 或下载离线安装程序
# https://developer.microsoft.com/en-us/microsoft-edge/webview2/
```

### 问题 4: 端口被占用

**错误**: `Port 1420 is already in use`

**解决**:
```powershell
# 查找占用端口的进程
netstat -ano | findstr :1420

# 终止进程 (替换 <PID> 为实际 PID)
taskkill /PID <PID> /F

# 或使用不同端口
$env:PORT = 1421
bun run tauri:dev
```

### 问题 5: 权限错误

**错误**: 应用无法写入文件

**解决**:
- 以管理员身份运行开发工具
- 或更改应用数据目录权限

### 问题 6: 中文路径问题

**错误**: 路径包含中文时出错

**解决**:
```powershell
# 确保使用 UTF-8 编码
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$env:PYTHONIOENCODING = "utf-8"

# 或避免在路径中使用中文
```

## 目录结构

```
KeVoiceInput/
├── src/                    # 前端 React 代码
├── src-tauri/             # 后端 Rust 代码
│   ├── src/
│   ├── target/            # 构建输出
│   │   └── release/
│   │       ├── kevoiceinput.exe
│   │       └── bundle/
│   │           └── msi/   # MSI 安装包
│   └── Cargo.toml
├── scripts/               # 构建脚本
│   └── build-windows.ps1  # Windows 构建脚本
└── docs/                  # 文档
    └── WINDOWS_PORT.md    # Windows 适配详细文档
```

## 调试技巧

### 1. 查看 Rust 日志

```powershell
# 设置日志级别
$env:RUST_LOG = "debug"
bun run tauri:dev
```

### 2. 查看前端控制台

开发模式下：
- 按 `F12` 打开开发者工具
- 查看 Console 标签

### 3. 查看网络请求

- 开发者工具 → Network 标签
- 监控 API 调用

### 4. Rust 调试

```rust
// 添加日志
log::info!("Debug message: {:?}", variable);
log::error!("Error: {:?}", error);
```

## 性能优化

### 1. 使用发布模式

```powershell
# 而不是 tauri:dev
bun run tauri:build
```

发布模式构建速度更快，性能更好。

### 2. 增量编译

Rust 默认启用增量编译，后续构建会更快。

### 3. 并行编译

```powershell
# 设置并行编译任务数
$env:CARGO_BUILD_JOBS = 4
```

## 下一步

- 阅读 [WINDOWS_PORT.md](WINDOWS_PORT.md) 了解详细的适配信息
- 查看 [BUILD_GUIDE.md](BUILD_GUIDE.md) 了解通用构建指南
- 参考 [DEBUGGING.md](DEBUGGING.md) 进行故障排查
- 阅读 [CONTRIBUTING.md](../CONTRIBUTING.md) 参与贡献

## 获取帮助

- **Issues**: [GitHub Issues](https://github.com/yourusername/KeVoiceInput/issues)
- **讨论**: [GitHub Discussions](https://github.com/yourusername/KeVoiceInput/discussions)
- **文档**: [docs/](.) 目录

## 许可证

MIT License - 查看 [LICENSE](../LICENSE) 文件
