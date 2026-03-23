# 跨平台权限管理

## 概述

KeVoiceInput 需要以下权限才能正常工作：
- **麦克风访问**: 录制语音输入
- **键盘输入模拟**: 将转录文本输入到其他应用（macOS 需要辅助功能权限）

不同平台的权限模型差异很大，本文档说明如何处理这些差异。

## 平台差异

| 权限类型 | macOS | Windows | Linux |
|---------|-------|---------|-------|
| 麦克风访问 | ✅ 需要系统权限 | ✅ 需要系统权限 | ✅ 需要系统权限 |
| 键盘输入模拟 | ✅ 需要辅助功能权限 | ❌ 通常无需特殊权限 | ❌ 通常无需特殊权限 |
| 权限请求方式 | 系统对话框 | 系统对话框 | 系统对话框 |
| 权限管理位置 | 系统偏好设置 → 安全性与隐私 | 设置 → 隐私 | 应用特定 |

## macOS 权限详细说明

### 辅助功能权限 (Accessibility)

**用途**: 允许应用模拟键盘输入，将转录文本输入到其他应用。

**请求流程**:
1. 应用调用 `requestAccessibilityPermission()`
2. 系统打开"安全性与隐私"设置
3. 用户手动在"辅助功能"列表中勾选 KeVoiceInput
4. 应用轮询检查权限状态

**代码实现**:
```typescript
// 仅在 macOS 上加载权限 API
import {
  checkAccessibilityPermission,
  requestAccessibilityPermission
} from "tauri-plugin-macos-permissions-api";

// 检查权限
const hasPermission = await checkAccessibilityPermission();

// 请求权限（打开系统设置）
await requestAccessibilityPermission();
```

**后端验证**:
```rust
// src-tauri/src/commands/mod.rs
#[tauri::command]
pub fn initialize_enigo(app: AppHandle) -> Result<(), String> {
    // 尝试初始化 Enigo (键盘模拟)
    // 在 macOS 上，如果没有辅助功能权限会失败
    let mut enigo_state = app.state::<EnigoState>();
    let mut enigo = enigo_state.0.lock().unwrap();

    if enigo.is_none() {
        *enigo = Some(Enigo::new(&Settings::default())
            .map_err(|e| format!("Failed to initialize Enigo: {}", e))?);
    }
    Ok(())
}
```

### 麦克风权限

**用途**: 录制用户语音输入。

**请求流程**:
1. 应用调用 `requestMicrophonePermission()`
2. 系统显示权限对话框
3. 用户点击"允许"或"拒绝"
4. 立即返回结果

**代码实现**:
```typescript
import {
  checkMicrophonePermission,
  requestMicrophonePermission
} from "tauri-plugin-macos-permissions-api";

// 检查权限
const hasPermission = await checkMicrophonePermission();

// 请求权限
await requestMicrophonePermission();
```

## Windows 权限说明

### 麦克风访问

**权限级别**: 应用级别

**请求流程**:
- 首次访问麦克风时，Windows 自动显示权限对话框
- 用户可以在"设置 → 隐私 → 麦克风"中管理权限

**代码实现**:
- 使用标准的 Web Audio API / CPAL 库
- 无需额外的权限请求代码
- 系统自动处理

### 键盘输入模拟

**权限级别**: 通常无需特殊权限

**说明**:
- Windows 允许应用使用 `SendInput()` API 模拟键盘输入
- 大多数应用可以接收模拟输入
- **例外情况**: 以管理员权限运行的应用可能无法接收来自普通权限应用的输入

**受保护的场景**:
- 以管理员身份运行的应用（如 Task Manager）
- UAC 提示对话框
- Windows 登录界面

**解决方案**:
- 推荐使用剪贴板 + Ctrl+V 粘贴方式
- 如果需要直接输入，可以请求用户以管理员身份运行应用（不推荐）

**代码实现**:
```rust
// src-tauri/src/input.rs
// enigo 库已经处理了 Windows 的 SendInput API
use enigo::{Enigo, Settings, Key, Keyboard};

let mut enigo = Enigo::new(&Settings::default())?;
enigo.key(Key::Control, enigo::Direction::Press)?;
enigo.key(Key::Unicode('v'), enigo::Direction::Click)?;
enigo.key(Key::Control, enigo::Direction::Release)?;
```

## Linux 权限说明

### 麦克风访问

**权限级别**: 取决于桌面环境

- **PulseAudio/PipeWire**: 通常无需额外权限
- **Flatpak/Snap**: 需要在应用清单中声明权限
- **AppImage**: 通常直接访问

### 键盘输入模拟

**方法**:
- 使用 X11 的 `XTest` 扩展
- 使用 Wayland 的输入协议（可能受限）

**限制**:
- Wayland 下某些操作可能需要额外权限
- 不同桌面环境行为可能不同

## 前端实现

### 跨平台权限组件

`src/components/AccessibilityPermissions.tsx`:

```typescript
import { type } from "@tauri-apps/plugin-os";

// 条件导入 macOS 权限 API
let checkAccessibilityPermission: (() => Promise<boolean>) | null = null;
let requestAccessibilityPermission: (() => Promise<void>) | null = null;

if (type() === "macos") {
  import("tauri-plugin-macos-permissions-api").then((mod) => {
    checkAccessibilityPermission = mod.checkAccessibilityPermission;
    requestAccessibilityPermission = mod.requestAccessibilityPermission;
  });
}

const AccessibilityPermissions: React.FC = () => {
  const [osType] = useState<string>(type());

  const checkPermissions = async (): Promise<boolean> => {
    // Windows 和 Linux 不需要辅助功能权限
    if (osType !== "macos") {
      return true;
    }

    // macOS 需要检查
    if (!checkAccessibilityPermission) {
      return false;
    }
    return await checkAccessibilityPermission();
  };

  // ...
};
```

### Onboarding 流程

`src/components/onboarding/AccessibilityOnboarding.tsx`:

```typescript
useEffect(() => {
  const currentPlatform = platform();
  const isMac = currentPlatform === "macos";
  setIsMacOS(isMac);

  // Windows/Linux 直接跳过权限检查
  if (!isMac) {
    onComplete();
    return;
  }

  // macOS 才检查权限
  // ...
}, []);
```

## 后端实现

### 跨平台输入模拟

`src-tauri/src/input.rs`:

```rust
pub fn send_text(text: &str, paste_method: PasteMethod) -> Result<()> {
    match paste_method {
        PasteMethod::CtrlV => {
            // Windows: Ctrl+V
            // macOS: Cmd+V
            // Linux: Ctrl+V
            #[cfg(target_os = "windows")]
            let modifier = Key::Control;
            #[cfg(target_os = "macos")]
            let modifier = Key::Meta;
            #[cfg(target_os = "linux")]
            let modifier = Key::Control;

            enigo.key(modifier, Direction::Press)?;
            enigo.key(Key::Unicode('v'), Direction::Click)?;
            enigo.key(modifier, Direction::Release)?;
        }
        PasteMethod::Direct => {
            // 直接输入文本（可能在某些 Windows 应用中失败）
            enigo.text(text)?;
        }
        // ...
    }
}
```

### 权限初始化

`src-tauri/src/lib.rs`:

```rust
fn initialize_core_logic(app_handle: &AppHandle) {
    // 注意: Enigo 不在这里初始化
    // 前端在 onboarding 完成后调用 initialize_enigo 命令
    // 避免在 macOS 上过早触发权限对话框
}
```

## 用户体验设计

### macOS 用户体验

1. **首次启动**: 显示 onboarding 界面
2. **权限请求**:
   - 麦克风: 点击按钮 → 系统对话框 → 立即授权
   - 辅助功能: 点击按钮 → 打开系统设置 → 手动勾选 → 返回应用验证
3. **权限授予后**: 自动进入主界面

### Windows 用户体验

1. **首次启动**: 直接进入主界面（跳过 onboarding）
2. **麦克风权限**: 首次使用转录时，系统自动弹出权限对话框
3. **无辅助功能权限**: 不显示相关提示

### Linux 用户体验

类似 Windows，根据桌面环境可能有细微差异。

## 测试清单

### macOS 测试

- [ ] 未授权辅助功能时，应用显示权限请求界面
- [ ] 点击"授权辅助功能"按钮，系统设置正确打开
- [ ] 授权后，应用能检测到权限变化
- [ ] 授权后，`initialize_enigo` 命令成功
- [ ] 授权后，转录文本能正确输入到其他应用
- [ ] 未授权麦克风时，请求对话框正确显示
- [ ] 授权麦克风后，能正常录音

### Windows 测试

- [ ] 首次启动直接进入主界面（无权限 UI）
- [ ] 首次使用转录时，系统显示麦克风权限对话框
- [ ] 授权麦克风后，能正常录音
- [ ] 转录文本能通过 Ctrl+V 输入到记事本
- [ ] 转录文本能通过 Ctrl+V 输入到 Word
- [ ] 转录文本能通过 Ctrl+V 输入到 Chrome
- [ ] 在管理员权限应用中测试（可能失败，预期行为）

### Linux 测试

- [ ] 首次启动直接进入主界面
- [ ] 能正常录音（可能需要 PulseAudio/PipeWire）
- [ ] 转录文本能输入到其他应用（X11）
- [ ] Wayland 下行为符合预期（可能有限制）

## 故障排查

### macOS: 权限对话框不出现

**问题**: 点击"授权"按钮后，系统设置没有打开。

**原因**:
- 权限 API 调用失败
- macOS 版本不兼容（需要 macOS 10.14+）

**解决方案**:
```bash
# 手动打开系统设置
open "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
```

### macOS: 辅助功能授权后仍无法输入

**问题**: 已授权辅助功能，但转录文本无法输入。

**原因**:
- `initialize_enigo` 命令未被调用
- Enigo 初始化失败

**解决方案**:
```rust
// 检查日志
log::info!("Enigo initialized: {}", enigo.is_some());

// 手动重试初始化
commands.initializeEnigo();
```

### Windows: 无法输入到管理员应用

**问题**: 转录文本无法输入到以管理员身份运行的应用。

**原因**: Windows UIPI (User Interface Privilege Isolation) 限制。

**解决方案**:
- **方案 A**: 提示用户关闭管理员应用或以普通权限运行
- **方案 B**: 以管理员身份运行 KeVoiceInput（不推荐）
- **方案 C**: 使用剪贴板方式（推荐，已实现）

### Windows: 麦克风无法访问

**问题**: 无法录音，提示麦克风访问被拒绝。

**解决方案**:
1. 打开"设置 → 隐私 → 麦克风"
2. 确保"允许应用访问麦克风"已开启
3. 确保 KeVoiceInput 在列表中已授权

## 未来改进

### 权限状态监听

当前实现使用轮询检查权限状态，未来可以考虑：
- 使用系统事件监听权限变化（如果 API 支持）
- 减少轮询频率以节省资源

### Linux Wayland 支持

Wayland 下的输入模拟受限，可能需要：
- 使用 D-Bus 接口
- 请求用户安装特定的桌面扩展
- 提供替代方案（如剪贴板）

### Windows UIAccess

对于需要输入到管理员应用的高级用户：
- 提供签名版本
- 安装到 Program Files
- 在清单中声明 UIAccess
- 需要代码签名证书

## 总结

✅ **权限管理已完全跨平台适配**

- macOS: 完整的权限请求和检查流程
- Windows: 自动权限处理，无需额外 UI
- Linux: 基本支持，依赖桌面环境

**关键策略**:
- 条件导入 macOS 特定 API，避免在其他平台报错
- 前端根据平台显示/隐藏权限 UI
- 后端使用跨平台库（enigo）处理输入模拟
- 提供多种输入方式（Direct, Ctrl+V, Shift+Insert）以适应不同场景

**Windows 适配状态**: ✅ 已完全支持，无需用户手动管理权限

## 相关文件

### 前端
- `src/components/AccessibilityPermissions.tsx` - 权限提示组件
- `src/components/onboarding/AccessibilityOnboarding.tsx` - Onboarding 流程

### 后端
- `src-tauri/src/commands/mod.rs` - `initialize_enigo` 命令
- `src-tauri/src/input.rs` - 输入模拟实现
- `src-tauri/src/lib.rs` - 应用初始化

### 依赖
- `tauri-plugin-macos-permissions` - macOS 权限管理
- `enigo` - 跨平台输入模拟

## 相关文档

- [macOS 辅助功能文档](https://developer.apple.com/documentation/accessibility)
- [Windows SendInput API](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput)
- [enigo 文档](https://docs.rs/enigo/)
