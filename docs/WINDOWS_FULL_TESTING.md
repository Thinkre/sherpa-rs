# Windows 完整功能测试指南

## 概述

本文档提供 KeVoiceInput Windows 版本的完整功能测试清单和流程。确保所有功能在 Windows 10/11 上正常工作。

## 测试环境

### 最低要求

| 组件 | 要求 |
|------|------|
| **操作系统** | Windows 10 (1809+) 或 Windows 11 |
| **CPU** | x86_64 (64位) |
| **内存** | 4GB RAM（推荐 8GB+） |
| **磁盘** | 500MB 可用空间（不含模型） |
| **显示器** | 1280x720 分辨率 |
| **麦克风** | 可用音频输入设备 |
| **网络** | 互联网连接（首次安装 WebView2） |

### 推荐测试环境

- **Windows 10** (21H2 或更新)
- **Windows 11** (最新版本)
- 多种硬件配置（笔记本、台式机）
- 不同键盘布局（QWERTY、AZERTY、俄文等）

## 测试阶段

### 阶段 1：安装和卸载测试

#### 1.1 全新安装

**测试步骤**：
1. 双击 MSI 安装包
2. 完成安装向导
3. 检查安装结果

**验证清单**：
- [ ] 安装程序正常启动（无签名警告如果已签名）
- [ ] 安装到 `C:\Program Files\KeVoiceInput\`
- [ ] 开始菜单创建快捷方式
- [ ] WebView2 运行时正确安装
- [ ] 应用出现在"设置 → 应用"列表
- [ ] 首次启动应用成功

#### 1.2 升级安装

**前提**：已安装旧版本（例如 0.0.1）

**测试步骤**：
1. 安装新版本（例如 0.0.2）
2. 检查升级过程

**验证清单**：
- [ ] 安装程序识别旧版本
- [ ] 用户设置被保留
- [ ] 模型文件被保留
- [ ] 历史记录被保留
- [ ] 应用版本号正确更新

#### 1.3 卸载测试

**测试步骤**：
1. 从"设置 → 应用"卸载
2. 或运行 MSI 卸载程序
3. 检查残留

**验证清单**：
- [ ] 应用完全卸载
- [ ] 开始菜单快捷方式移除
- [ ] 托盘图标消失
- [ ] 安装目录被删除
- [ ] 用户数据保留在 `%APPDATA%\com.kevoiceinput.app\`（预期行为）
- [ ] 注册表项移除（除用户数据路径）

### 阶段 2：首次运行测试

#### 2.1 应用启动

**测试步骤**：
1. 从开始菜单启动应用
2. 观察启动过程

**验证清单**：
- [ ] 应用窗口正常显示
- [ ] 界面无错位或乱码
- [ ] 托盘图标出现在系统托盘
- [ ] 默认设置已加载

#### 2.2 Onboarding 流程

**注意**：Windows 不需要权限授权，应直接跳过 onboarding 或显示简化版。

**验证清单**：
- [ ] 不显示辅助功能权限请求（macOS 专用）
- [ ] 显示欢迎界面或功能介绍（可选）
- [ ] 直接进入主界面

#### 2.3 界面元素

**验证清单**：
- [ ] 所有按钮、图标显示正常
- [ ] 字体清晰，无乱码
- [ ] 设置选项卡可访问（General、Shortcuts、Models、History、Post-Processing）
- [ ] 下拉菜单、复选框等组件正常

### 阶段 3：核心功能测试

#### 3.1 快捷键注册

**测试步骤**：
1. 检查默认快捷键设置
2. 按下默认快捷键（Ctrl+Space）
3. 自定义快捷键并测试

**验证清单**：
- [ ] 默认快捷键：Ctrl+Space（不是 Option+Space）
- [ ] 按下快捷键启动录音
- [ ] 按下 Escape 取消录音
- [ ] 自定义快捷键生效（例如 Ctrl+Shift+V）
- [ ] 快捷键冲突检测（可选）

#### 3.2 音频录制

**测试步骤**：
1. 选择麦克风设备
2. 按下快捷键录音
3. 说出测试文本
4. 自动或手动停止录音

**验证清单**：
- [ ] 列出所有可用麦克风设备
- [ ] 成功切换麦克风设备
- [ ] 录音时播放提示音（可选）
- [ ] 录音时托盘图标变为"Recording"状态
- [ ] VAD 自动检测语音结束（如果启用）
- [ ] 手动按键停止录音正常工作

#### 3.3 语音转录

**测试引擎和模型**：

| 引擎 | 模型 | 测试语言 | 预期结果 |
|------|------|---------|---------|
| **Whisper** | Small | 中文/英文 | 基本准确 |
| **Whisper** | Turbo | 中文/英文 | 高准确度 |
| **Transducer** | Zipformer 双语 | 中文/英文 | 准确，支持热词 |
| **Paraformer** | KeSeaCo | 中文 | 准确，支持热词 |
| **FireRedAsr** | Large | 中文方言 | 方言准确 |

**测试步骤**（每个模型）：
1. 下载或导入模型
2. 选择模型
3. 录制测试音频
4. 检查转录结果

**测试文本示例**：
- 中文："你好，这是一个语音转录测试。今天天气很好，我们来测试一下语音识别的准确性。"
- 英文："Hello, this is a voice transcription test. Today's weather is great, and we're testing the accuracy of speech recognition."
- 混合："今天我要写一些 code，然后 push 到 GitHub。"

**验证清单**：
- [ ] 模型下载成功（本地模型）
- [ ] 模型加载无错误
- [ ] 转录结果基本准确（允许少量错别字）
- [ ] 转录速度合理（< 5秒，取决于模型）
- [ ] 托盘图标切换到"Transcribing"状态

#### 3.4 输入模拟

**测试场景**：见 `WINDOWS_INPUT_TESTING.md`

**关键测试**：
1. **记事本**（Ctrl+V）
2. **PowerShell**（Ctrl+V 和 Shift+Insert）
3. **Chrome**（Ctrl+V）
4. **VS Code**（Ctrl+V）

**验证清单**：
- [ ] Ctrl+V 在大多数应用中有效
- [ ] Shift+Insert 在终端中有效
- [ ] Direct 模式在密码框中有效（可选）
- [ ] 文本插入到光标位置
- [ ] 文本编码正确（UTF-8，无乱码）

#### 3.5 热词功能

**前提**：使用支持热词的模型（Transducer Zipformer、Paraformer KeSeaCo）

**测试步骤**：
1. 在设置中添加热词（例如："KeVoiceInput"、"GitHub"）
2. 录音时说出热词
3. 检查转录结果

**验证清单**：
- [ ] 热词列表正常显示
- [ ] 添加热词成功
- [ ] 删除热词成功
- [ ] 热词在转录中被正确识别
- [ ] 热词规则生效（如果配置）

### 阶段 4：高级功能测试

#### 4.1 LLM 后处理

**测试 LLM 提供商**：
- OpenAI（需要 API Key）
- Anthropic Claude（需要 API Key）
- Groq（需要 API Key）
- DashScope（需要 API Key）
- Custom LLM（本地 Ollama）

**测试步骤**：
1. 配置 LLM API Key
2. 启用后处理
3. 选择角色（Polisher、Translator 等）
4. 录制并转录
5. 检查后处理结果

**验证清单**：
- [ ] API Key 配置成功保存
- [ ] 后处理请求成功发送
- [ ] 后处理结果正确显示
- [ ] 错误处理（无效 API Key、网络错误）友好
- [ ] Apple Intelligence 选项**不显示**（Windows 不支持）

#### 4.2 历史记录

**测试步骤**：
1. 执行多次转录（至少 5 次）
2. 在 History 选项卡查看记录
3. 播放历史音频
4. 复制历史转录
5. 删除历史记录

**验证清单**：
- [ ] 所有转录记录显示
- [ ] 记录包含时间戳和模型信息
- [ ] 音频文件可播放
- [ ] 复制转录文本到剪贴板成功
- [ ] 删除记录后文件也被删除
- [ ] 搜索和过滤功能正常（如果有）

#### 4.3 自动更新

**前提**：应用已发布新版本，`latest.json` 已更新

**测试步骤**：
1. 在 Settings 中启用自动更新
2. 点击"Check for Updates"
3. 如果有更新，下载并安装

**验证清单**：
- [ ] 成功检测到新版本
- [ ] 更新下载进度显示
- [ ] 签名验证通过
- [ ] 更新安装后应用重启
- [ ] 版本号正确更新

### 阶段 5：设置和配置测试

#### 5.1 通用设置

**验证清单**：
- [ ] 语言切换正常（14 种语言）
- [ ] 音频设备切换生效
- [ ] VAD 灵敏度调整有效
- [ ] 提示音开关正常
- [ ] 输入方法切换（CtrlV、Direct、ShiftInsert）

#### 5.2 快捷键设置

**验证清单**：
- [ ] 快捷键绑定界面正常
- [ ] 快捷键冲突检测（可选）
- [ ] 重置为默认快捷键
- [ ] 自定义快捷键立即生效

#### 5.3 模型管理

**验证清单**：
- [ ] 列出所有可用模型
- [ ] 下载模型进度显示
- [ ] 暂停/恢复下载（如果支持）
- [ ] 导入本地模型
- [ ] 删除模型（确认对话框）
- [ ] 切换活动模型

### 阶段 6：性能和稳定性测试

#### 6.1 性能基准

**测试场景**：

| 测试 | 指标 | 预期 |
|------|------|------|
| 应用启动 | 时间 | < 3秒 |
| 模型加载 | 时间 | < 5秒 |
| 转录（10秒音频）| 时间 | < 5秒（Whisper Small）|
| 内存使用 | 峰值 | < 1GB（无模型）|
| CPU 使用 | 转录时 | < 80% |

**测试工具**：
- Windows 任务管理器（性能监视）
- Process Explorer（详细分析）

#### 6.2 长时间运行测试

**测试步骤**：
1. 启动应用
2. 运行 24 小时（或更长）
3. 定期执行转录（每小时 1-2 次）
4. 监控资源使用

**验证清单**：
- [ ] 无内存泄漏（内存使用稳定）
- [ ] 无 CPU 占用异常
- [ ] 应用保持响应
- [ ] 托盘图标正常

#### 6.3 压力测试

**测试场景**：
1. 连续转录 50 次
2. 快速切换模型（10 次）
3. 同时下载多个模型
4. 快速打开/关闭设置窗口

**验证清单**：
- [ ] 应用不崩溃
- [ ] 响应速度正常
- [ ] 无错误提示
- [ ] 资源使用合理

### 阶段 7：兼容性测试

#### 7.1 键盘布局测试

**测试布局**：
- QWERTY（美式）
- QWERTZ（德式）
- AZERTY（法式）
- 俄文布局
- 中文拼音输入法

**验证清单**：
- [ ] 快捷键在所有布局下有效
- [ ] Ctrl+V 粘贴在所有布局下有效
- [ ] 输入文本编码正确

#### 7.2 多显示器测试

**测试场景**：
- 双显示器
- 不同 DPI 设置（100%、125%、150%）
- 主副显示器切换

**验证清单**：
- [ ] 窗口在正确显示器上显示
- [ ] 托盘图标在所有显示器上可见
- [ ] UI 缩放正确（高 DPI）
- [ ] 拖动窗口到不同显示器正常

#### 7.3 虚拟机和远程桌面

**测试环境**：
- VMware Workstation
- VirtualBox
- Windows 远程桌面（RDP）

**验证清单**：
- [ ] 应用在虚拟机中正常运行
- [ ] 音频录制正常（虚拟麦克风）
- [ ] 远程桌面中应用正常显示
- [ ] 剪贴板功能正常（远程桌面）

### 阶段 8：国际化测试

#### 8.1 语言切换

**测试所有支持语言**：
1. 简体中文
2. 繁体中文
3. English
4. 日本語
5. 한국어
6. Español
7. Français
8. Deutsch
9. Italiano
10. Português
11. Русский
12. العربية
13. Türkçe
14. Tiếng Việt

**验证清单**：
- [ ] 所有语言翻译完整
- [ ] 无乱码或显示问题
- [ ] UI 布局适应不同文本长度
- [ ] 托盘菜单正确翻译

#### 8.2 地区格式

**验证清单**：
- [ ] 日期时间格式符合地区习惯
- [ ] 数字格式正确（小数点、千分位）
- [ ] 文件路径处理正确（中文路径）

## 自动化测试脚本

### PowerShell 测试脚本

创建 `tests/scripts/full-windows-test.ps1`：

```powershell
#Requires -Version 5.1
$ErrorActionPreference = "Stop"

Write-Host "=== KeVoiceInput Windows 完整功能测试 ===" -ForegroundColor Cyan
Write-Host ""

# 测试 1: 安装验证
Write-Host "[1/8] 安装验证..." -ForegroundColor Yellow
$installPath = "C:\Program Files\KeVoiceInput\kevoiceinput.exe"
if (Test-Path $installPath) {
    Write-Host "  ✅ 应用已安装: $installPath" -ForegroundColor Green
} else {
    Write-Host "  ❌ 应用未安装" -ForegroundColor Red
    exit 1
}

# 测试 2: 启动测试
Write-Host "[2/8] 启动测试..." -ForegroundColor Yellow
$process = Start-Process $installPath -PassThru
Start-Sleep -Seconds 5
if ($process -and !$process.HasExited) {
    Write-Host "  ✅ 应用成功启动 (PID: $($process.Id))" -ForegroundColor Green
} else {
    Write-Host "  ❌ 应用启动失败" -ForegroundColor Red
    exit 1
}

# 测试 3: 托盘图标
Write-Host "[3/8] 托盘图标测试..." -ForegroundColor Yellow
Write-Host "  请手动验证系统托盘中是否显示 KeVoiceInput 图标" -ForegroundColor Gray
Read-Host "按 Enter 继续"

# 测试 4: 快捷键
Write-Host "[4/8] 快捷键测试..." -ForegroundColor Yellow
Write-Host "  请按 Ctrl+Space 测试快捷键" -ForegroundColor Gray
Read-Host "按 Enter 继续"

# 测试 5: 音频录制
Write-Host "[5/8] 音频录制测试..." -ForegroundColor Yellow
Write-Host "  请录制一段测试音频" -ForegroundColor Gray
Read-Host "按 Enter 继续"

# 测试 6: 输入模拟
Write-Host "[6/8] 输入模拟测试..." -ForegroundColor Yellow
Start-Process notepad
Start-Sleep -Seconds 2
Write-Host "  请在记事本中测试 Ctrl+V 粘贴" -ForegroundColor Gray
Read-Host "按 Enter 继续"

# 测试 7: 设置界面
Write-Host "[7/8] 设置界面测试..." -ForegroundColor Yellow
Write-Host "  请打开设置并检查所有选项卡" -ForegroundColor Gray
Read-Host "按 Enter 继续"

# 测试 8: 清理
Write-Host "[8/8] 清理..." -ForegroundColor Yellow
Stop-Process -Id $process.Id -Force
Write-Host "  ✅ 测试进程已停止" -ForegroundColor Green

Write-Host ""
Write-Host "=== 测试完成！===" -ForegroundColor Cyan
Write-Host "请根据上述测试结果填写测试报告" -ForegroundColor Gray
```

## 测试报告模板

### 测试执行记录

**测试环境**：
- 操作系统：Windows 10/11 版本
- CPU：型号
- 内存：容量
- 测试日期：YYYY-MM-DD
- 应用版本：0.0.1

**测试结果**：

| 测试项 | 结果 | 说明 |
|--------|------|------|
| 安装 | ✅ 通过 | - |
| 卸载 | ✅ 通过 | - |
| 首次启动 | ✅ 通过 | - |
| 快捷键 | ✅ 通过 | Ctrl+Space 正常 |
| 音频录制 | ✅ 通过 | - |
| 语音转录 | ✅ 通过 | Whisper Small |
| 输入模拟 | ✅ 通过 | Ctrl+V 正常 |
| 热词功能 | ✅ 通过 | - |
| LLM 后处理 | ⚠️ 警告 | 需要 API Key |
| 历史记录 | ✅ 通过 | - |
| 自动更新 | ⏭️ 跳过 | 无可用更新 |
| 性能测试 | ✅ 通过 | 内存 < 800MB |
| 长时间运行 | ✅ 通过 | 24 小时稳定 |
| 多显示器 | ✅ 通过 | - |
| 国际化 | ✅ 通过 | 14 种语言 |

**发现的问题**：
1. [问题描述]
2. [问题描述]

**建议改进**：
1. [改进建议]
2. [改进建议]

## 回归测试清单

发布前必须通过的测试：

### 关键功能（P0）
- [ ] 应用安装成功
- [ ] 应用启动无崩溃
- [ ] 快捷键（Ctrl+Space）正常
- [ ] 音频录制正常
- [ ] 至少一个模型转录成功
- [ ] Ctrl+V 输入到记事本成功

### 重要功能（P1）
- [ ] 托盘图标显示和菜单
- [ ] 设置保存和加载
- [ ] 模型下载和管理
- [ ] 历史记录查看
- [ ] 卸载干净

### 次要功能（P2）
- [ ] 热词功能
- [ ] LLM 后处理
- [ ] 自动更新检测
- [ ] 多语言切换
- [ ] 高 DPI 支持

## 测试工具

### 推荐工具

1. **Process Monitor**（SysInternals）
   - 监控文件、注册表操作
   - 调试权限问题

2. **Dependency Walker**
   - 检查 DLL 依赖

3. **Wireshark**
   - 监控网络请求（更新检测）

4. **Windows Performance Recorder**
   - 性能分析

5. **Accessibility Insights**
   - UI 可访问性测试

## 已知限制

### Windows 平台特定限制

1. **管理员权限应用**：无法向以管理员权限运行的应用发送输入
2. **UAC 对话框**：无法在 UAC 提示框中输入
3. **某些游戏**：全屏游戏可能无法接收输入
4. **密码框**：某些密码输入框禁用粘贴

### 性能限制

1. **大模型加载**：Large 模型加载可能需要 5-10 秒
2. **长音频转录**：> 1 分钟音频转录时间较长
3. **内存使用**：加载多个模型会占用较多内存

## 总结

✅ **Windows 功能测试清单完整**

**测试覆盖**：
- 8 个测试阶段
- 100+ 验证项
- 15+ 测试场景
- 自动化测试脚本
- 详细测试报告模板

**重点测试项**：
- 安装和卸载
- 核心转录功能
- 输入模拟兼容性
- 性能和稳定性
- 跨环境兼容性

## 相关文档

- [WINDOWS_INPUT_TESTING.md](WINDOWS_INPUT_TESTING.md) - 输入模拟详细测试
- [WINDOWS_QUICKSTART.md](WINDOWS_QUICKSTART.md) - Windows 快速开始
- [WINDOWS_MSI_CONFIG.md](WINDOWS_MSI_CONFIG.md) - MSI 安装包配置
- [CROSS_PLATFORM_SHORTCUTS.md](CROSS_PLATFORM_SHORTCUTS.md) - 快捷键处理
- [CROSS_PLATFORM_TRAY.md](CROSS_PLATFORM_TRAY.md) - 托盘图标
