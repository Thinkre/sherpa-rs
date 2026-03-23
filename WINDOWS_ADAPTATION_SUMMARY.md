# KeVoiceInput Windows 平台适配总结

**创建日期**: 2026-03-05
**状态**: 规划完成，待实施

## 📋 概述

本文档总结了将 KeVoiceInput 从 macOS 适配到 Windows 平台的完整规划。

## 🎯 适配目标

将功能完整的 macOS 版本 KeVoiceInput 适配到 Windows 平台，保持所有核心功能：
- ✅ 多引擎语音识别（Whisper, Paraformer, Transducer, FireRedAsr）
- ✅ 实时语音转录
- ✅ 全局快捷键
- ✅ 键盘输入模拟
- ✅ 历史记录管理
- ✅ LLM 后处理
- ✅ 自动更新

## 📊 任务统计

**总任务数**: 13 个
**优先级分布**:
- P0 (关键): 5 个
- P1 (重要): 6 个
- P2 (一般): 2 个

## 📁 创建的文档和脚本

### 1. 文档

#### docs/WINDOWS_PORT.md
- **内容**: 完整的 Windows 适配技术文档
- **包含**:
  - 平台差异详细分析
  - 13 个具体适配任务（带代码示例）
  - 实施步骤指南
  - 完整测试计划
  - 已知问题和解决方案

#### docs/WINDOWS_QUICKSTART.md
- **内容**: Windows 用户快速开始指南
- **包含**:
  - 前置条件和软件安装
  - 快速构建步骤（预编译库）
  - 完整构建步骤（从源码编译）
  - 开发工作流
  - 常见问题解决
  - 调试技巧

### 2. 脚本

#### scripts/build-windows.ps1
- **功能**: Windows 自动化构建脚本
- **特性**:
  - 前置条件检查
  - sherpa-onnx 库验证
  - 清理构建产物
  - 安装依赖
  - 构建前端
  - 构建 Tauri 应用
  - 自动复制 DLL
  - 友好的彩色输出
- **用法**:
  ```powershell
  # 开发模式
  .\scripts\build-windows.ps1 -Dev

  # 生产构建
  .\scripts\build-windows.ps1

  # 清理后构建
  .\scripts\build-windows.ps1 -Clean -SherpaPath "C:\path\to\sherpa"
  ```

## 🔧 核心适配任务

### Phase 1: 核心功能 (P0)

#### ✅ 任务创建

所有 13 个任务已创建并在任务系统中跟踪：

1. **#15**: 适配文件路径处理（Windows）
2. **#26**: 适配动态库加载（Windows DLL）
3. **#16**: 适配权限管理（Windows）
4. **#24**: 测试输入模拟（Windows）
5. **#19**: 更新快捷键处理（Windows）
6. **#25**: 适配托盘图标（Windows）
7. **#22**: 创建 Windows 构建脚本 ✅
8. **#17**: 配置 Windows 安装包（MSI）
9. **#27**: 编译 sherpa-onnx（Windows）
10. **#23**: 更新 GitHub Actions（Windows 构建）
11. **#20**: 隐藏 Apple Intelligence 功能（Windows）
12. **#18**: Windows 完整功能测试
13. **#21**: 更新文档（Windows 支持）

### 关键适配点

#### 1. 文件路径
```rust
// Before (macOS only)
let app_data = PathBuf::from("~/Library/Application Support/com.kevoiceinput.app");

// After (Cross-platform)
#[cfg(target_os = "windows")]
let app_data = dirs::config_dir()
    .unwrap()
    .join("com.kevoiceinput.app");

#[cfg(target_os = "macos")]
let app_data = dirs::config_dir()
    .unwrap()
    .join("com.kevoiceinput.app");
```

#### 2. 动态库
| 平台 | 扩展名 | 环境变量 |
|------|--------|----------|
| macOS | `.dylib` | `DYLD_LIBRARY_PATH` |
| Windows | `.dll` | `PATH` 或同目录 |

#### 3. 权限
- **macOS**: 辅助功能权限（accessibility）
- **Windows**: 通常无需特殊权限，某些场景需 UIAccess

#### 4. 快捷键
- **macOS**: Cmd ⌘
- **Windows**: Ctrl

## 📦 依赖管理

### sherpa-onnx 编译 (Windows)

```cmd
REM CMake 配置
cmake -G "Visual Studio 17 2022" -A x64 ^
      -DCMAKE_BUILD_TYPE=Release ^
      -DCMAKE_INSTALL_PREFIX=..\install ^
      -DSHERPA_ONNX_ENABLE_PYTHON=OFF ^
      -DSHERPA_ONNX_ENABLE_TESTS=OFF ^
      -DBUILD_SHARED_LIBS=ON ^
      ..

REM 构建
cmake --build . --config Release

REM 安装
cmake --install . --config Release
```

### 环境变量

```powershell
# 临时设置
$env:SHERPA_LIB_PATH = "C:\path\to\sherpa-onnx\install\bin"

# 永久设置
[Environment]::SetEnvironmentVariable(
    "SHERPA_LIB_PATH",
    "C:\path\to\sherpa-onnx\install\bin",
    "User"
)
```

## 🔍 测试覆盖

### 测试环境
- Windows 10 (21H2+)
- Windows 11
- 不同语言版本（中文、英文）

### 测试类别
1. **基本功能**: 启动、设置、模型管理
2. **音频功能**: 录制、VAD、播放
3. **转录功能**: 所有引擎测试
4. **文本处理**: ITN、词典、标点、LLM
5. **输入模拟**: 多种应用测试（记事本、Word、Chrome、cmd）
6. **快捷键**: 全局快捷键、自定义
7. **系统集成**: 托盘、通知、自动启动
8. **历史记录**: 数据库操作
9. **更新功能**: 检查、下载、安装更新

## 🚀 实施计划

### 阶段 1: 环境准备（第 1 周）
- [ ] 设置 Windows 开发环境
- [ ] 编译 sherpa-onnx
- [ ] 验证所有依赖可用

### 阶段 2: 核心功能适配（第 2-3 周）
- [ ] 文件路径处理
- [ ] 动态库加载
- [ ] 权限管理
- [ ] 输入模拟测试

### 阶段 3: 系统集成（第 4 周）
- [ ] 托盘图标
- [ ] 快捷键处理
- [ ] 自动启动
- [ ] 窗口管理

### 阶段 4: 构建和测试（第 5-6 周）
- [ ] 构建脚本完善
- [ ] MSI 安装包配置
- [ ] 完整功能测试
- [ ] 性能优化

### 阶段 5: 文档和发布（第 7 周）
- [ ] 更新所有文档
- [ ] 配置 CI/CD
- [ ] 首个 Windows 版本发布

## 📝 已知限制

### 1. Apple Intelligence
- **限制**: 仅 macOS 可用
- **处理**: Windows 版本隐藏此功能

### 2. 权限模型差异
- **macOS**: 辅助功能权限（系统级）
- **Windows**: 通常无需特殊权限，但某些受保护应用可能需要管理员权限

### 3. 快捷键
- 需要调整默认快捷键以符合 Windows 习惯
- Cmd → Ctrl

## 📈 进度跟踪

使用 GitHub Issues/Projects 跟踪：
- 标签: `platform: windows`
- 里程碑: `Windows Support v1.0`
- 优先级: `priority: P0/P1/P2`

查看任务列表：
```bash
# 使用 Claude Code 任务系统
/tasks

# 或使用 GitHub CLI
gh issue list --label "platform: windows"
```

## 🔗 相关资源

### 文档链接
- [Windows 适配详细文档](docs/WINDOWS_PORT.md)
- [Windows 快速开始](docs/WINDOWS_QUICKSTART.md)
- [构建指南](docs/BUILD_GUIDE.md)
- [调试指南](docs/DEBUGGING.md)

### 外部资源
- [Tauri Windows Prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites#windows)
- [sherpa-onnx Windows Build](https://k2-fsa.github.io/sherpa/onnx/install/windows.html)
- [ONNX Runtime Windows](https://onnxruntime.ai/docs/install/)
- [Rust Windows Guide](https://doc.rust-lang.org/book/ch01-01-installation.html#installing-rustup-on-windows)

## 💡 技术亮点

### 跨平台架构
- 使用条件编译 `#[cfg(target_os = "...")]`
- 平台无关的核心逻辑
- 平台特定的适配层

### 构建自动化
- PowerShell 构建脚本
- 前置条件自动检查
- DLL 自动复制
- 友好的错误提示

### 文档完善
- 快速开始指南
- 详细技术文档
- 常见问题解答
- 代码示例丰富

## 🎓 经验总结

### 适配要点
1. **路径处理**: 使用 `dirs` crate 获取平台特定路径
2. **库加载**: 正确设置环境变量和复制 DLL
3. **权限**: 了解平台权限模型差异
4. **测试**: 在真实 Windows 环境充分测试
5. **文档**: 提供清晰的 Windows 用户指南

### 最佳实践
1. 使用 `#[cfg]` 条件编译分离平台代码
2. 集中管理平台差异（如路径获取函数）
3. 提供自动化构建脚本
4. 完善的错误处理和日志
5. 持续的跨平台测试

## 🚦 下一步行动

### 立即可做
1. 审查创建的文档和脚本
2. 设置 Windows 开发环境
3. 开始 Phase 1 任务实施

### 需要决策
1. 是否支持 Windows 7/8（仅 10+）？
2. 是否需要 ARM64 Windows 支持？
3. 安装包格式：仅 MSI 还是也提供 .exe？
4. 代码签名策略

### 社区参与
- 欢迎 Windows 开发者贡献
- 设置 Windows 测试环境
- 收集 Windows 用户反馈

## 📞 联系方式

- **Issues**: [GitHub Issues](https://github.com/yourusername/KeVoiceInput/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/KeVoiceInput/discussions)
- **Label**: 使用 `platform: windows` 标签

---

**状态**: ✅ 规划完成
**下一步**: 开始实施 Phase 1 任务
**预计完成**: 7 周后第一个 Windows 版本发布
