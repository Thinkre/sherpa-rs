# KeVoiceInput Windows 平台适配指南

本文档描述如何将 macOS 版本的 KeVoiceInput 适配到 Windows 平台。

## 目录

- [概述](#概述)
- [平台差异分析](#平台差异分析)
- [适配任务清单](#适配任务清单)
- [实施步骤](#实施步骤)
- [测试计划](#测试计划)
- [已知问题](#已知问题)

## 概述

KeVoiceInput 当前主要针对 macOS 开发，包含以下平台特定功能：
- macOS 辅助功能权限管理
- 动态库加载（.dylib）
- 文件路径（~/Library/Application Support）
- 系统菜单栏和 Dock 集成
- macOS 特定的快捷键处理

本文档提供完整的 Windows 适配方案。

## 平台差异分析

### 1. 文件系统路径

| 类型 | macOS | Windows |
|------|-------|---------|
| 应用数据 | `~/Library/Application Support/com.kevoiceinput.app/` | `%APPDATA%\com.kevoiceinput.app\` |
| 日志 | `~/Library/Logs/com.kevoiceinput.app/` | `%APPDATA%\com.kevoiceinput.app\logs\` |
| 缓存 | `~/Library/Caches/com.kevoiceinput.app/` | `%LOCALAPPDATA%\com.kevoiceinput.app\cache\` |
| 临时文件 | `/tmp/` | `%TEMP%\` |

### 2. 动态库

| 平台 | 文件扩展名 | 加载方式 |
|------|-----------|----------|
| macOS | `.dylib` | `DYLD_LIBRARY_PATH` 或打包到 `.app` |
| Windows | `.dll` | 放在可执行文件同目录或 `PATH` |

### 3. 权限管理

| 功能 | macOS | Windows |
|------|-------|---------|
| 键盘输入模拟 | 需要辅助功能权限 | 需要 UIAccess 或管理员权限（特定场景）|
| 麦克风访问 | 系统权限提示 | 系统权限提示 |
| 全局快捷键 | 系统支持 | 系统支持 |
| 自动启动 | Launch Agents | 注册表或启动文件夹 |

### 4. 系统集成

| 功能 | macOS | Windows |
|------|-------|---------|
| 托盘图标 | 菜单栏 | 系统托盘 |
| 通知 | macOS 通知中心 | Windows 通知中心 |
| 窗口管理 | NSWindow | Win32 API |

### 5. 构建工具

| 类型 | macOS | Windows |
|------|-------|---------|
| 安装包 | `.dmg` | `.msi` 或 `.exe` |
| 代码签名 | Apple Developer Certificate | Authenticode |
| 依赖管理 | Homebrew | vcpkg 或手动 |

## 适配任务清单

### Phase 1: 核心功能适配

#### 1.1 文件路径处理 ⭐ P0
- [ ] **文件**: `src-tauri/src/settings.rs`
  - [ ] 添加 Windows 路径获取函数
  - [ ] 使用 `dirs` crate 或 `tauri::path` API
  - [ ] 更新 `get_app_data_dir()` 函数

  ```rust
  #[cfg(target_os = "windows")]
  pub fn get_app_data_dir() -> PathBuf {
      dirs::config_dir()
          .unwrap_or_else(|| PathBuf::from("."))
          .join("com.kevoiceinput.app")
  }

  #[cfg(target_os = "macos")]
  pub fn get_app_data_dir() -> PathBuf {
      dirs::config_dir()
          .unwrap_or_else(|| PathBuf::from("."))
          .join("com.kevoiceinput.app")
  }
  ```

- [ ] **文件**: `src-tauri/src/managers/history.rs`
  - [ ] 更新数据库路径
  - [ ] 更新录音文件保存路径

- [ ] **文件**: `src-tauri/src/managers/model.rs`
  - [ ] 更新模型文件存储路径

#### 1.2 动态库加载 ⭐ P0
- [ ] **文件**: `vendor/sherpa-rs/crates/sherpa-rs-sys/build.rs`
  - [ ] 添加 Windows DLL 路径处理
  - [ ] 更新库文件搜索逻辑

  ```rust
  #[cfg(target_os = "windows")]
  fn get_lib_extension() -> &'static str {
      ".dll"
  }

  #[cfg(target_os = "macos")]
  fn get_lib_extension() -> &'static str {
      ".dylib"
  }
  ```

- [ ] **配置**: 环境变量
  - [ ] macOS: `DYLD_LIBRARY_PATH`
  - [ ] Windows: `PATH` 或复制到 exe 同目录

#### 1.3 权限管理 ⭐ P0
- [ ] **文件**: `src-tauri/src/commands/mod.rs`
  - [ ] 添加 Windows 权限检查命令
  - [ ] 移除 macOS 特定的辅助功能权限检查

  ```rust
  #[tauri::command]
  #[specta::specta]
  pub fn check_permissions(app: AppHandle) -> Result<PermissionStatus, String> {
      #[cfg(target_os = "macos")]
      {
          // macOS 辅助功能权限检查
          check_accessibility_permission()
      }

      #[cfg(target_os = "windows")]
      {
          // Windows 不需要特殊权限检查
          Ok(PermissionStatus {
              accessibility: true,
              microphone: true,
          })
      }
  }
  ```

- [ ] **前端**: `src/components/AccessibilityPermissions.tsx`
  - [ ] 添加平台检测
  - [ ] Windows 显示不同的权限说明

#### 1.4 输入模拟 ⭐ P1
- [ ] **文件**: `src-tauri/src/input.rs`
  - [ ] 验证 Windows 虚拟键码
  - [ ] 测试 Ctrl+V 粘贴功能
  - [ ] 测试 Shift+Insert 粘贴（Windows 常用）

  ```rust
  #[cfg(target_os = "windows")]
  let (modifier_key, v_key_code) = (Key::Control, Key::Other(0x56)); // VK_V
  ```

- [ ] **测试**: 不同应用程序
  - [ ] 记事本
  - [ ] Word
  - [ ] Chrome
  - [ ] VS Code
  - [ ] 命令提示符/PowerShell

#### 1.5 快捷键处理 ⭐ P1
- [ ] **文件**: `src-tauri/src/shortcut.rs`
  - [ ] 验证 Windows 快捷键格式
  - [ ] 更新默认快捷键绑定

  ```rust
  // macOS: CommandOrControl+Shift+Space
  // Windows: Ctrl+Shift+Space
  ```

- [ ] **前端**: 快捷键显示
  - [ ] 显示平台正确的修饰键（⌘ vs Ctrl）

### Phase 2: 系统集成

#### 2.1 托盘图标 ⭐ P1
- [ ] **文件**: `src-tauri/src/tray.rs`
  - [ ] 更新托盘图标资源（Windows 格式）
  - [ ] 测试右键菜单功能

- [ ] **资源**: `src-tauri/icons/`
  - [ ] 添加 Windows .ico 文件
  - [ ] 更新 `tauri.conf.json` 图标配置

#### 2.2 自动启动 ⭐ P2
- [ ] **插件**: `tauri-plugin-autostart`
  - [ ] 验证 Windows 自动启动
  - [ ] 测试注册表写入
  - [ ] 测试用户登录时启动

#### 2.3 窗口管理 P2
- [ ] **文件**: `src-tauri/src/overlay.rs`
  - [ ] 测试覆盖窗口（overlay）显示
  - [ ] 测试窗口置顶功能
  - [ ] 测试窗口位置计算

#### 2.4 Apple Intelligence 功能 P2
- [ ] **条件编译**:
  ```rust
  #[cfg(target_os = "macos")]
  {
      // Apple Intelligence 代码
  }

  #[cfg(not(target_os = "macos"))]
  {
      return Err("Apple Intelligence only available on macOS");
  }
  ```

- [ ] **前端**: 隐藏 Apple Intelligence 选项（Windows）

### Phase 3: 构建和部署

#### 3.1 构建脚本 ⭐ P1
- [ ] **文件**: `scripts/`
  - [ ] 创建 `build-windows.ps1` PowerShell 脚本
  - [ ] 或创建 `build-windows.bat` 批处理脚本

  ```powershell
  # build-windows.ps1
  $ErrorActionPreference = "Stop"

  Write-Host "Building KeVoiceInput for Windows..."

  # Install frontend dependencies
  bun install

  # Build frontend
  bun run build

  # Build Tauri
  bun run tauri build

  Write-Host "Build complete!"
  Write-Host "Output: src-tauri/target/release/bundle/msi/"
  ```

- [ ] **DLL 打包**
  - [ ] 复制 sherpa-onnx DLLs 到输出目录
  - [ ] 复制 ONNX Runtime DLLs

#### 3.2 安装包 ⭐ P1
- [ ] **配置**: `tauri.conf.json`
  - [ ] 配置 WiX 安装器
  - [ ] 设置安装路径
  - [ ] 配置开始菜单快捷方式

  ```json
  {
    "bundle": {
      "windows": {
        "certificateThumbprint": null,
        "digestAlgorithm": "sha256",
        "timestampUrl": "",
        "wix": {
          "language": ["zh-CN", "en-US"]
        }
      }
    }
  }
  ```

#### 3.3 代码签名 P2
- [ ] 获取 Authenticode 证书
- [ ] 配置 GitHub Actions 密钥
- [ ] 更新发布工作流

#### 3.4 更新机制 P1
- [ ] 测试 Tauri Updater 在 Windows
- [ ] 生成 `.msi.zip` 和签名
- [ ] 更新 `latest.json` 格式

### Phase 4: 依赖管理

#### 4.1 sherpa-onnx P0
- [ ] 编译 Windows 版本 sherpa-onnx
  - [ ] 使用 Visual Studio 2019+
  - [ ] CMake 配置
  - [ ] 生成 DLL 文件

- [ ] 环境变量
  ```cmd
  set SHERPA_LIB_PATH=C:\path\to\sherpa-onnx\install\lib
  ```

#### 4.2 ONNX Runtime P0
- [ ] 下载 Windows 版本
- [ ] 设置正确的版本（与 macOS 一致）
- [ ] 复制到构建输出

#### 4.3 其他依赖 P1
- [ ] 验证所有 Cargo 依赖支持 Windows
- [ ] 更新 `Cargo.toml` 平台特定依赖

### Phase 5: 文档更新

#### 5.1 用户文档 P1
- [ ] **文件**: `README.md`
  - [ ] 添加 Windows 安装说明
  - [ ] 更新系统要求

- [ ] **文件**: `docs/BUILD_GUIDE.md`
  - [ ] 添加 Windows 构建步骤
  - [ ] 添加依赖安装说明

#### 5.2 开发文档 P2
- [ ] **文件**: `CLAUDE.md`
  - [ ] 更新平台特定注意事项

- [ ] **文件**: `docs/WINDOWS_PORT.md`
  - [ ] 本文档，持续更新

## 实施步骤

### Step 1: 准备 Windows 开发环境

```powershell
# 1. 安装 Rust
winget install Rustlang.Rustup

# 2. 安装 Node.js 和 Bun
winget install Oven-sh.Bun

# 3. 安装 Visual Studio Build Tools
winget install Microsoft.VisualStudio.2022.BuildTools

# 4. 安装 WebView2
# Tauri 会自动处理，或手动安装运行时

# 5. 克隆仓库
git clone https://github.com/yourusername/KeVoiceInput.git
cd KeVoiceInput
```

### Step 2: 编译 sherpa-onnx (Windows)

```cmd
REM 下载 sherpa-onnx
git clone https://github.com/k2-fsa/sherpa-onnx.git
cd sherpa-onnx

REM 创建构建目录
mkdir build
cd build

REM CMake 配置
cmake -G "Visual Studio 17 2022" -A x64 ^
      -DCMAKE_BUILD_TYPE=Release ^
      -DCMAKE_INSTALL_PREFIX=../install ^
      -DSHERPA_ONNX_ENABLE_PYTHON=OFF ^
      -DSHERPA_ONNX_ENABLE_TESTS=OFF ^
      -DSHERPA_ONNX_ENABLE_CHECK=OFF ^
      -DBUILD_SHARED_LIBS=ON ^
      ..

REM 构建
cmake --build . --config Release

REM 安装
cmake --install . --config Release

REM 设置环境变量
set SHERPA_LIB_PATH=%CD%\..\install\bin
```

### Step 3: 适配核心代码

按照任务清单逐个修改文件，优先处理 P0 任务。

### Step 4: 构建测试

```powershell
# 设置环境变量
$env:SHERPA_LIB_PATH = "C:\path\to\sherpa-onnx\install\bin"

# 安装依赖
bun install

# 开发模式测试
bun run tauri:dev

# 生产构建
bun run tauri:build

# 检查输出
dir src-tauri\target\release\bundle\msi\
```

### Step 5: 功能测试

按照测试计划逐项验证功能。

## 测试计划

### 测试环境

- Windows 10 (21H2 或更高)
- Windows 11
- 不同语言版本（中文、英文）

### 测试清单

#### 基本功能
- [ ] 应用启动
- [ ] 设置保存和加载
- [ ] 模型下载和管理
- [ ] 模型切换

#### 音频功能
- [ ] 麦克风列表
- [ ] 音频录制
- [ ] VAD 检测
- [ ] 录音播放

#### 转录功能
- [ ] Whisper 模型转录
- [ ] Paraformer 模型转录
- [ ] SeACo Paraformer 热词
- [ ] Transducer 流式转录
- [ ] FireRedAsr 转录

#### 文本处理
- [ ] ITN 转换
- [ ] 自定义词典
- [ ] 标点符号
- [ ] 热词规则
- [ ] LLM 后处理

#### 输入模拟
- [ ] Ctrl+V 粘贴（记事本）
- [ ] Ctrl+V 粘贴（Word）
- [ ] Ctrl+V 粘贴（Chrome）
- [ ] Shift+Insert 粘贴（cmd）
- [ ] 特殊字符处理

#### 快捷键
- [ ] 全局快捷键注册
- [ ] 快捷键触发转录
- [ ] 取消快捷键
- [ ] 自定义快捷键

#### 系统集成
- [ ] 托盘图标显示
- [ ] 托盘菜单
- [ ] 窗口最小化到托盘
- [ ] 自动启动
- [ ] Windows 通知

#### 历史记录
- [ ] 历史列表
- [ ] 历史搜索
- [ ] 录音播放
- [ ] 历史删除
- [ ] SQLite 数据库

#### 更新功能
- [ ] 检查更新
- [ ] 下载更新
- [ ] 安装更新
- [ ] 回滚机制

## 已知问题

### 1. 权限问题

**问题**: Windows UAC 可能阻止某些输入模拟

**解决方案**:
- 方案 A: 使用 UIAccess（需要代码签名和特殊安装位置）
- 方案 B: 提示用户以管理员身份运行（某些场景）
- 方案 C: 使用剪贴板方式（推荐）

### 2. 路径问题

**问题**: Windows 路径分隔符（`\` vs `/`）

**解决方案**: 使用 `std::path::PathBuf` 处理所有路径

### 3. DLL 依赖

**问题**: MSVC Runtime、ONNX Runtime 等 DLL 缺失

**解决方案**:
- 静态链接 MSVC Runtime
- 打包所有必需 DLL
- 使用 WiX 安装器安装依赖

### 4. 中文路径

**问题**: Windows 用户名包含中文时路径处理问题

**解决方案**: 确保使用 UTF-8 编码处理路径

## 资源链接

- [Tauri Windows Prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites#windows)
- [sherpa-onnx Windows Build](https://k2-fsa.github.io/sherpa/onnx/install/windows.html)
- [ONNX Runtime Windows](https://onnxruntime.ai/docs/install/)
- [Windows API Documentation](https://docs.microsoft.com/en-us/windows/win32/)

## 贡献

欢迎贡献 Windows 适配代码！请参考 [CONTRIBUTING.md](../CONTRIBUTING.md)。

## 进度跟踪

使用 GitHub Issues 或 Project Board 跟踪适配进度。

创建标签：
- `platform: windows`
- `priority: P0` / `P1` / `P2`
- `status: in-progress` / `testing` / `done`
