# Windows 输入模拟测试指南

## 概述

本文档提供 Windows 平台输入模拟功能的详细测试指南，确保转录文本能正确输入到各种应用程序中。

## 输入方法

KeVoiceInput 支持四种输入方法：

| 方法 | 快捷键 | 适用场景 | Windows 支持 |
|------|--------|---------|-------------|
| **CtrlV** | Ctrl+V | 大多数应用 | ✅ 完全支持 |
| **Direct** | 直接输入 | 需要逐字符输入 | ⚠️ 部分应用 |
| **ShiftInsert** | Shift+Insert | 终端、命令行 | ✅ 完全支持 |
| **CtrlShiftV** | Ctrl+Shift+V | 终端（无格式粘贴） | ✅ 完全支持 |

## 技术实现

### Virtual Key Code 使用

Windows 输入模拟使用 **虚拟键码（VK_*）** 而不是字符，确保在所有键盘布局下都能工作。

**代码位置**：`src-tauri/src/input.rs`

```rust
// Ctrl+V 实现（第 32-33 行）
#[cfg(target_os = "windows")]
let (modifier_key, v_key_code) = (Key::Control, Key::Other(0x56)); // VK_V

// Shift+Insert 实现（第 93-94 行）
#[cfg(target_os = "windows")]
let insert_key_code = Key::Other(0x2D); // VK_INSERT
```

### 关键虚拟键码

| 键 | 虚拟键码 | 十六进制 | 说明 |
|----|---------|---------|------|
| V | 86 | 0x56 | 粘贴操作 |
| C | 67 | 0x43 | 复制操作 |
| Insert | 45 | 0x2D | Shift+Insert 粘贴 |
| Escape | 27 | 0x1B | 取消操作 |

**为什么使用虚拟键码？**
- 独立于键盘布局（QWERTY、AZERTY、DVORAK、俄文等）
- 直接映射到物理键位置
- 更可靠的跨布局兼容性

## 测试场景

### 1. 标准应用测试

#### 1.1 记事本（Notepad）

**目标**：验证基本文本输入

**测试步骤**：
1. 打开记事本（`notepad.exe`）
2. 在 KeVoiceInput 中选择输入方法：`CtrlV`
3. 触发快捷键（默认 `Ctrl+Space`）
4. 说出测试文本："你好，这是一个测试。"
5. 验证文本正确出现在记事本中

**预期结果**：✅ 文本准确插入

#### 1.2 Microsoft Word

**目标**：验证富文本编辑器兼容性

**测试步骤**：
1. 打开 Microsoft Word
2. 输入方法：`CtrlV`
3. 在文档中定位光标
4. 转录测试文本
5. 验证文本格式（应保持纯文本）

**预期结果**：✅ 文本插入，格式正确

#### 1.3 Google Chrome / Microsoft Edge

**目标**：验证浏览器输入框兼容性

**测试场景**：
- 搜索框（Google、Bing）
- 文本编辑器（Google Docs、Notion）
- 社交媒体输入框（Twitter、Facebook）

**测试步骤**：
1. 打开浏览器并访问测试网站
2. 点击输入框
3. 输入方法：`CtrlV`
4. 转录文本
5. 验证文本正确插入

**预期结果**：✅ 在所有现代浏览器输入框中正常工作

### 2. 命令行和终端测试

#### 2.1 Command Prompt（命令提示符）

**目标**：验证传统终端兼容性

**测试步骤**：
1. 打开命令提示符（`cmd.exe`）
2. 输入方法：**ShiftInsert**（推荐）或 `CtrlV`
3. 转录命令："echo 测试文本"
4. 按 Enter 执行

**预期结果**：
- ✅ ShiftInsert：总是有效
- ⚠️ CtrlV：在新版 Windows 10/11 中有效，旧版可能无效

#### 2.2 PowerShell

**目标**：验证 PowerShell 输入

**测试步骤**：
1. 打开 PowerShell
2. 输入方法：`CtrlV` 或 `ShiftInsert`
3. 转录 PowerShell 命令："Get-Process | Select-Object -First 5"
4. 执行命令

**预期结果**：✅ 两种方法都有效

#### 2.3 Windows Terminal

**目标**：验证现代终端应用

**测试步骤**：
1. 打开 Windows Terminal
2. 测试多个 Shell：
   - PowerShell
   - Command Prompt
   - WSL (Ubuntu)
3. 输入方法：`CtrlShiftV`（推荐）或 `CtrlV`
4. 转录并执行命令

**预期结果**：✅ 所有 Shell 中均有效

### 3. 开发工具测试

#### 3.1 Visual Studio Code

**目标**：验证代码编辑器兼容性

**测试步骤**：
1. 打开 VS Code
2. 创建新文件或打开现有文件
3. 输入方法：`CtrlV`
4. 转录代码注释："// 这是一个测试注释"
5. 转录代码："function test() { return true; }"

**预期结果**：✅ 文本正确插入编辑器

**注意事项**：
- 不要期望自动代码格式化
- 不会触发 IntelliSense
- 适合快速注释和简单代码片段

#### 3.2 Visual Studio 2022

**目标**：验证 IDE 兼容性

**测试步骤**：
1. 打开 Visual Studio
2. 在代码编辑器中定位光标
3. 输入方法：`CtrlV`
4. 转录代码或注释

**预期结果**：✅ 正常工作

#### 3.3 JetBrains IDE（IntelliJ IDEA, PyCharm）

**目标**：验证 JetBrains 系列 IDE

**测试步骤**：
1. 打开 JetBrains IDE
2. 输入方法：`CtrlV`
3. 转录代码

**预期结果**：✅ 正常工作

### 4. 办公软件测试

#### 4.1 Microsoft Excel

**目标**：验证电子表格应用

**测试步骤**：
1. 打开 Excel
2. 选择单元格
3. 输入方法：`CtrlV`
4. 转录文本或数字
5. 按 Enter 确认输入

**预期结果**：✅ 文本插入单元格

#### 4.2 Microsoft PowerPoint

**目标**：验证演示文稿应用

**测试步骤**：
1. 打开 PowerPoint
2. 在文本框中点击
3. 输入方法：`CtrlV`
4. 转录文本

**预期结果**：✅ 文本插入文本框

### 5. 特殊场景测试

#### 5.1 密码输入框

**问题**：某些密码输入框禁用粘贴功能。

**测试**：
1. 打开银行网站或需要密码的应用
2. 尝试使用 `CtrlV` 输入密码
3. 如果失败，切换到 `Direct` 方法

**预期结果**：
- ❌ CtrlV：可能被阻止
- ✅ Direct：应该有效（逐字符输入）

#### 5.2 管理员权限应用

**问题**：普通权限的 KeVoiceInput 无法向管理员权限应用发送输入。

**测试**：
1. 以管理员身份运行记事本（右键 → 以管理员身份运行）
2. 尝试使用 KeVoiceInput 输入文本

**预期结果**：
- ❌ 输入失败（UIPI 限制）

**解决方案**：
- 方案 A：关闭管理员权限应用或以普通权限运行
- 方案 B：以管理员身份运行 KeVoiceInput（不推荐）
- 方案 C：文本已复制到剪贴板，手动粘贴

#### 5.3 虚拟机和远程桌面

**测试场景**：
- VMware Workstation
- VirtualBox
- Windows 远程桌面（RDP）

**测试步骤**：
1. 在虚拟机或远程桌面内运行 KeVoiceInput
2. 测试各种输入方法

**预期结果**：
- ✅ 大多数场景下有效
- ⚠️ 某些远程桌面配置可能限制剪贴板

### 6. 键盘布局兼容性测试

#### 6.1 非 QWERTY 布局

**测试布局**：
- AZERTY（法语）
- QWERTZ（德语）
- DVORAK
- 俄文布局
- 中文拼音输入法

**测试步骤**：
1. 在 Windows 设置中添加键盘布局
2. 切换到测试布局
3. 使用 KeVoiceInput 输入文本（`CtrlV` 方法）
4. 验证输入成功

**预期结果**：
- ✅ 虚拟键码确保在所有布局下都能工作
- V 键的物理位置不变，无论显示什么字符

#### 6.2 中文输入法激活状态

**测试**：
1. 启用中文输入法（搜狗、微软拼音等）
2. 输入法处于中文模式
3. 使用 KeVoiceInput 转录文本

**预期结果**：
- ✅ CtrlV：应该有效，直接插入中文
- ⚠️ Direct：可能触发拼音输入，产生意外结果

**建议**：Windows 下推荐使用 `CtrlV` 方法

## 自动化测试脚本

### PowerShell 测试脚本

创建 `tests/scripts/test-windows-input.ps1`：

```powershell
#Requires -Version 5.1
$ErrorActionPreference = "Stop"

Write-Host "KeVoiceInput Windows 输入模拟测试" -ForegroundColor Cyan
Write-Host ""

# 测试 1: 记事本
Write-Host "[1/4] 测试记事本..." -ForegroundColor Yellow
Start-Process notepad
Start-Sleep -Seconds 2
Write-Host "  请在 KeVoiceInput 中转录测试文本" -ForegroundColor Gray
Write-Host "  文本应出现在记事本中" -ForegroundColor Gray
Read-Host "按 Enter 继续"

# 测试 2: PowerShell
Write-Host "[2/4] 测试 PowerShell..." -ForegroundColor Yellow
Start-Process powershell
Start-Sleep -Seconds 2
Write-Host "  请转录 PowerShell 命令（例如：Get-Date）" -ForegroundColor Gray
Read-Host "按 Enter 继续"

# 测试 3: Chrome
Write-Host "[3/4] 测试 Chrome..." -ForegroundColor Yellow
Start-Process chrome "https://www.google.com"
Start-Sleep -Seconds 3
Write-Host "  请在搜索框中转录文本" -ForegroundColor Gray
Read-Host "按 Enter 继续"

# 测试 4: VS Code
Write-Host "[4/4] 测试 VS Code..." -ForegroundColor Yellow
if (Get-Command code -ErrorAction SilentlyContinue) {
    code
    Start-Sleep -Seconds 3
    Write-Host "  请在编辑器中转录文本" -ForegroundColor Gray
} else {
    Write-Host "  ⚠️  VS Code 未安装，跳过" -ForegroundColor Yellow
}
Read-Host "按 Enter 继续"

Write-Host ""
Write-Host "✅ 测试完成！" -ForegroundColor Green
Write-Host "如果所有测试都通过，Windows 输入模拟功能正常" -ForegroundColor Green
```

运行测试：
```powershell
cd tests/scripts
.\test-windows-input.ps1
```

## 常见问题

### Q1: Ctrl+V 不工作

**可能原因**：
1. 目标应用禁用了粘贴功能
2. 应用以管理员权限运行
3. enigo 库初始化失败

**排查步骤**：
```powershell
# 1. 手动测试剪贴板
echo "测试文本" | clip
# 然后在目标应用中手动按 Ctrl+V

# 2. 检查应用日志
# 查找 src-tauri/target/release/kevoiceinput.exe 的日志输出

# 3. 尝试其他输入方法
# 切换到 ShiftInsert 或 Direct
```

### Q2: 文本乱码

**可能原因**：
1. 字符编码问题
2. 目标应用不支持 UTF-8

**解决方案**：
- 确保 Windows 系统语言设置正确
- 在"区域"设置中启用"Beta: 使用 Unicode UTF-8 提供全球语言支持"

### Q3: 输入速度慢

**原因**：Direct 方法逐字符输入，速度较慢。

**解决方案**：
- 切换到 CtrlV 方法（推荐）
- 调整输入延迟设置（如果提供）

### Q4: 终端中无法输入

**问题**：旧版 Command Prompt 不支持 Ctrl+V。

**解决方案**：
1. 在命令提示符属性中启用"使用 Ctrl+V 粘贴"（Windows 10+）
2. 使用 ShiftInsert 方法
3. 升级到 Windows Terminal

## 性能基准

典型输入延迟（Intel i7 / 16GB RAM）：

| 方法 | 延迟 | 字符/秒 | 适用场景 |
|------|------|---------|---------|
| CtrlV | ~100ms | 即时 | 推荐 |
| Direct | ~50ms/字符 | ~20 | 特殊应用 |
| ShiftInsert | ~100ms | 即时 | 终端 |
| CtrlShiftV | ~100ms | 即时 | 终端 |

## 回归测试清单

在发布新版本前，运行以下测试：

- [ ] 记事本（Ctrl+V）
- [ ] Word（Ctrl+V）
- [ ] Chrome 搜索框（Ctrl+V）
- [ ] VS Code（Ctrl+V）
- [ ] PowerShell（Ctrl+V 和 Shift+Insert）
- [ ] Command Prompt（Shift+Insert）
- [ ] Windows Terminal（Ctrl+Shift+V）
- [ ] Excel 单元格（Ctrl+V）
- [ ] 非 QWERTY 键盘布局（至少测试一种）
- [ ] 管理员权限应用（预期失败，验证错误处理）

## 参考资料

- [Windows 虚拟键码](https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes)
- [SendInput API](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput)
- [enigo 文档](https://docs.rs/enigo/)
- [UIPI 文档](https://docs.microsoft.com/en-us/windows/win32/secauthz/user-interface-privilege-isolation)

## 相关文档

- [CROSS_PLATFORM_SHORTCUTS.md](CROSS_PLATFORM_SHORTCUTS.md) - 跨平台快捷键处理
- [CROSS_PLATFORM_PERMISSIONS.md](CROSS_PLATFORM_PERMISSIONS.md) - 权限管理
- [WINDOWS_QUICKSTART.md](WINDOWS_QUICKSTART.md) - Windows 快速开始
