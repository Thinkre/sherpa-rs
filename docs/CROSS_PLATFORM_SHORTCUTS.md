# 跨平台快捷键和输入处理

## 概述

KeVoiceInput 使用全局快捷键触发语音输入，并通过模拟键盘输入将转录文本插入到其他应用。本文档说明跨平台的快捷键和输入处理实现。

## 平台默认快捷键

| 平台 | 转录快捷键 | 取消快捷键 | 说明 |
|------|----------|----------|------|
| macOS | `Option+Space` | `Escape` | Option 键相当于 Alt |
| Windows | `Ctrl+Space` | `Escape` | 标准 Windows 快捷键 |
| Linux | `Ctrl+Space` | `Escape` | 与 Windows 一致 |

### 修饰键对应关系

| 键名 | macOS | Windows | Linux |
|------|-------|---------|-------|
| Command/Super | `⌘ (Cmd)` | `⊞ (Win)` | `Super` |
| Control | `⌃ (Ctrl)` | `Ctrl` | `Ctrl` |
| Option/Alt | `⌥ (Option)` | `Alt` | `Alt` |
| Shift | `⇧ (Shift)` | `Shift` | `Shift` |

## 代码实现

### 1. 默认快捷键配置

`src-tauri/src/settings.rs:736-744`:

```rust
pub fn get_default_settings() -> AppSettings {
    #[cfg(target_os = "windows")]
    let default_shortcut = "ctrl+space";

    #[cfg(target_os = "macos")]
    let default_shortcut = "option+space";

    #[cfg(target_os = "linux")]
    let default_shortcut = "ctrl+space";

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    let default_shortcut = "alt+space"; // 其他平台后备

    let mut bindings = HashMap::new();
    bindings.insert(
        "transcribe".to_string(),
        ShortcutBinding {
            id: "transcribe".to_string(),
            name: "Transcribe".to_string(),
            description: "Converts your speech into text.".to_string(),
            default_binding: default_shortcut.to_string(),
            current_binding: default_shortcut.to_string(),
        },
    );
    // ...
}
```

### 2. 快捷键注册

使用 `tauri-plugin-global-shortcut` 处理全局快捷键：

`src-tauri/src/shortcut.rs`:

```rust
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub fn init_shortcuts(app: &AppHandle) {
    let user_settings = settings::load_or_create_app_settings(app);

    for (id, binding) in user_settings.bindings {
        if let Err(e) = register_shortcut(app, binding) {
            error!("Failed to register shortcut {}: {}", id, e);
        }
    }
}

fn register_shortcut(app: &AppHandle, binding: ShortcutBinding) -> Result<(), String> {
    let shortcut = binding.current_binding.parse::<Shortcut>()
        .map_err(|e| format!("Invalid shortcut: {}", e))?;

    app.global_shortcut()
        .on_shortcut(shortcut, move |app, _shortcut, _event| {
            // 触发操作
            execute_action(app, &binding.id);
        })
        .map_err(|e| format!("Failed to register: {}", e))?;

    Ok(())
}
```

### 3. 输入模拟 - Ctrl+V / Cmd+V

`src-tauri/src/input.rs:25-52`:

```rust
/// 发送 Ctrl+V (Windows/Linux) 或 Cmd+V (macOS) 粘贴命令
pub fn send_paste_ctrl_v(enigo: &mut Enigo) -> Result<(), String> {
    // 平台特定的键定义
    #[cfg(target_os = "macos")]
    let (modifier_key, v_key_code) = (Key::Meta, Key::Other(9));

    #[cfg(target_os = "windows")]
    let (modifier_key, v_key_code) = (Key::Control, Key::Other(0x56)); // VK_V

    #[cfg(target_os = "linux")]
    let (modifier_key, v_key_code) = (Key::Control, Key::Unicode('v'));

    // 按下修饰键 + V
    enigo.key(modifier_key, enigo::Direction::Press)?;
    enigo.key(v_key_code, enigo::Direction::Click)?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    enigo.key(modifier_key, enigo::Direction::Release)?;

    Ok(())
}
```

**关键点**:
- Windows 使用虚拟键码 `0x56` (VK_V) 确保在任何键盘布局下都能工作
- macOS 使用键码 `9` (对应 V 键的物理位置)
- Linux 使用 Unicode 字符 `'v'`

### 4. 输入模拟 - Shift+Insert

Windows 和 Linux 特定的粘贴方式：

```rust
pub fn send_paste_shift_insert(enigo: &mut Enigo) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let insert_key_code = Key::Other(0x2D); // VK_INSERT

    #[cfg(not(target_os = "windows"))]
    let insert_key_code = Key::Other(0x76); // XK_Insert

    // 按下 Shift + Insert
    enigo.key(Key::Shift, enigo::Direction::Press)?;
    enigo.key(insert_key_code, enigo::Direction::Click)?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    enigo.key(Key::Shift, enigo::Direction::Release)?;

    Ok(())
}
```

**用途**:
- 终端应用（Windows Command Prompt, PowerShell）
- 某些不支持 Ctrl+V 的旧版软件

### 5. 输入模拟 - Ctrl+C / Cmd+C

读取选中文本时使用：

```rust
pub fn send_copy_ctrl_c(enigo: &mut Enigo) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let (modifier_key, c_key_code) = (Key::Meta, Key::Unicode('c'));

    #[cfg(target_os = "windows")]
    let (modifier_key, c_key_code) = (Key::Control, Key::Unicode('c'));

    #[cfg(target_os = "linux")]
    let (modifier_key, c_key_code) = (Key::Control, Key::Unicode('c'));

    // 按下修饰键 + C
    enigo.key(modifier_key, enigo::Direction::Press)?;
    enigo.key(c_key_code, enigo::Direction::Click)?;

    std::thread::sleep(std::time::Duration::from_millis(50));

    enigo.key(modifier_key, enigo::Direction::Release)?;

    Ok(())
}
```

## 输入方法配置

用户可以选择不同的输入方法：

### PasteMethod 枚举

`src-tauri/src/settings.rs:206-214`:

```rust
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum PasteMethod {
    CtrlV,        // Ctrl+V (Windows/Linux) / Cmd+V (macOS)
    Direct,       // 直接输入文本
    None,         // 不输入
    ShiftInsert,  // Shift+Insert (Windows/Linux)
    CtrlShiftV,   // Ctrl+Shift+V / Cmd+Shift+V
}

impl Default for PasteMethod {
    fn default() -> Self {
        // macOS 和 Windows 默认使用 CtrlV，Linux 使用 Direct
        #[cfg(target_os = "linux")]
        return PasteMethod::Direct;

        #[cfg(not(target_os = "linux"))]
        return PasteMethod::CtrlV;
    }
}
```

### 方法适用场景

| 方法 | macOS | Windows | Linux | 适用场景 |
|------|-------|---------|-------|---------|
| **CtrlV** | ✅ Cmd+V | ✅ Ctrl+V | ✅ Ctrl+V | 大多数应用 |
| **Direct** | ⚠️ 有限 | ⚠️ 有限 | ✅ 推荐 | X11 下最佳，Wayland 受限 |
| **ShiftInsert** | ❌ 不支持 | ✅ 终端 | ✅ 终端 | 命令提示符、PowerShell |
| **CtrlShiftV** | ✅ 终端 | ✅ 终端 | ✅ 终端 | 终端应用（无格式粘贴） |
| **None** | ✅ | ✅ | ✅ | 仅复制到剪贴板 |

## 虚拟键码参考

### Windows 虚拟键码 (VK_*)

| 键 | 虚拟键码 | 十六进制 |
|----|---------|---------|
| V | 86 | 0x56 |
| C | 67 | 0x43 |
| Insert | 45 | 0x2D |
| Escape | 27 | 0x1B |
| Space | 32 | 0x20 |

完整列表: https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes

### macOS 键码

| 键 | 键码 | 说明 |
|----|------|------|
| V | 9 | 物理位置 |
| C | 8 | 物理位置 |
| Space | 49 | 空格键 |
| Escape | 53 | Esc 键 |

### Linux X11 键码 (keysym)

| 键 | Keysym | 十六进制 |
|----|--------|---------|
| v | XK_v | 0x0076 |
| c | XK_c | 0x0063 |
| Insert | XK_Insert | 0xFF63 |
| Escape | XK_Escape | 0xFF1B |

## 用户自定义快捷键

### 前端接口

`src/components/settings/ShortcutSettings.tsx`:

```typescript
import { commands } from "@/bindings";

const changeShortcut = async (id: string, newBinding: string) => {
  try {
    const result = await commands.changeBinding(id, newBinding);
    if (result.success) {
      toast.success("Shortcut updated successfully");
    } else {
      toast.error(result.error || "Failed to update shortcut");
    }
  } catch (error) {
    toast.error("Failed to update shortcut");
  }
};
```

### 快捷键格式

快捷键字符串格式由 `tauri-plugin-global-shortcut` 定义：

**单个键**:
- `"a"`, `"1"`, `"f12"`

**修饰键 + 键**:
- `"ctrl+a"`, `"shift+f5"`, `"alt+space"`

**多个修饰键**:
- `"ctrl+shift+a"`, `"ctrl+alt+delete"`

**平台特定修饰键**:
- `"commandorcontrol+c"` - macOS 使用 Cmd，其他平台使用 Ctrl
- `"option+space"` - macOS 专用
- `"super+d"` - Linux Super 键 (Windows 键)

### 验证快捷键

`src-tauri/src/shortcut.rs`:

```rust
fn validate_shortcut_string(shortcut: &str) -> Result<(), String> {
    // 尝试解析快捷键
    shortcut.parse::<Shortcut>()
        .map_err(|e| format!("Invalid shortcut format: {}", e))?;

    // 检查是否被系统占用
    // ...

    Ok(())
}
```

## 键盘布局兼容性

### 问题

不同键盘布局（QWERTY, AZERTY, DVORAK, 俄文等）的键位不同，使用字符 `'v'` 可能无法正确粘贴。

### 解决方案

**Windows**: 使用虚拟键码 (VK_V = 0x56) 而不是字符
```rust
#[cfg(target_os = "windows")]
let v_key_code = Key::Other(0x56); // VK_V，物理位置，不受布局影响
```

**macOS**: 使用键码 (9) 而不是字符
```rust
#[cfg(target_os = "macos")]
let v_key_code = Key::Other(9); // 键码 9 是 V 键的物理位置
```

**Linux**: 使用 Unicode 字符（可能在某些布局下需要回退到键码）
```rust
#[cfg(target_os = "linux")]
let v_key_code = Key::Unicode('v');
```

## 特殊场景处理

### 1. Wayland 限制 (Linux)

**问题**: Wayland 下输入模拟受限，需要额外权限或不支持。

**检测方法**:
```rust
fn is_wayland() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_ok()
}
```

**解决方案**:
- 自动回退到剪贴板方式 (`PasteMethod::None`)
- 提示用户使用 X11 或安装特定工具

### 2. 管理员权限应用 (Windows)

**问题**: 普通权限应用无法向管理员权限应用发送输入。

**解决方案**:
- 推荐使用剪贴板 + 粘贴方式
- 或提示用户以管理员身份运行 KeVoiceInput（不推荐）

### 3. 受保护的系统界面

某些界面不接受模拟输入：
- Windows UAC 提示
- macOS 密码输入框
- Linux 锁屏界面

**处理**: 用户需要手动粘贴。

## 测试清单

### 快捷键测试

**macOS**:
- [ ] Option+Space 触发转录
- [ ] Escape 取消录音
- [ ] 自定义快捷键生效
- [ ] 与系统快捷键冲突检测

**Windows**:
- [ ] Ctrl+Space 触发转录
- [ ] Escape 取消录音
- [ ] 自定义快捷键生效
- [ ] 不同键盘布局下快捷键有效

**Linux**:
- [ ] Ctrl+Space 触发转录
- [ ] X11 下快捷键正常工作
- [ ] Wayland 下快捷键正常工作

### 输入模拟测试

**macOS**:
- [ ] Cmd+V 粘贴到 TextEdit
- [ ] Cmd+V 粘贴到 Chrome
- [ ] Cmd+V 粘贴到 VS Code
- [ ] Cmd+Shift+V 粘贴到 Terminal

**Windows**:
- [ ] Ctrl+V 粘贴到记事本
- [ ] Ctrl+V 粘贴到 Word
- [ ] Ctrl+V 粘贴到 Chrome
- [ ] Shift+Insert 粘贴到 Command Prompt
- [ ] Shift+Insert 粘贴到 PowerShell
- [ ] 不同键盘布局（QWERTY, AZERTY, 俄文）

**Linux**:
- [ ] X11: Ctrl+V 粘贴到 gedit
- [ ] X11: Ctrl+V 粘贴到 Chrome
- [ ] X11: Direct 输入到终端
- [ ] Wayland: 检测并回退到剪贴板

## 性能优化

### 延迟控制

输入模拟之间的延迟影响用户体验：

```rust
// 快速操作: 50ms
std::thread::sleep(std::time::Duration::from_millis(50));

// 标准操作: 100ms
std::thread::sleep(std::time::Duration::from_millis(100));

// 慢速应用（Excel, Word）: 200ms
std::thread::sleep(std::time::Duration::from_millis(200));
```

**可配置**: 未来可以让用户调整延迟设置。

### 错误恢复

输入模拟失败时的处理：

```rust
pub fn send_text_with_retry(text: &str, paste_method: PasteMethod, max_retries: u32) -> Result<(), String> {
    for attempt in 0..max_retries {
        match send_text(text, paste_method) {
            Ok(()) => return Ok(()),
            Err(e) if attempt < max_retries - 1 => {
                log::warn!("Input attempt {} failed: {}, retrying...", attempt + 1, e);
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}
```

## 故障排查

### macOS: 快捷键不响应

**原因**:
- 辅助功能权限未授予
- 快捷键与系统冲突

**解决**:
```bash
# 检查辅助功能权限
tccutil reset Accessibility com.kevoiceinput.app

# 检查快捷键冲突
# 系统偏好设置 → 键盘 → 快捷键
```

### Windows: Ctrl+V 无法粘贴

**原因**:
- 目标应用以管理员身份运行
- 剪贴板被其他程序占用
- 目标应用不支持粘贴

**解决**:
1. 关闭管理员权限应用或以管理员身份运行 KeVoiceInput
2. 尝试其他输入方法（Shift+Insert）
3. 检查剪贴板内容

### Linux: Wayland 下无法输入

**原因**: Wayland 安全限制。

**解决**:
- 切换到 X11 会话
- 使用剪贴板方式
- 安装 `ydotool` 或类似工具（需要 root）

## 总结

✅ **快捷键和输入处理已完全跨平台**

**关键策略**:
- 使用条件编译 `#[cfg(target_os = "...")]` 处理平台差异
- Windows 使用虚拟键码，确保键盘布局兼容
- 提供多种输入方法，适应不同场景
- 智能检测平台限制（Wayland, UAC）并回退

**Windows 适配状态**: ✅ 已完全支持
- 默认快捷键: Ctrl+Space
- 输入方法: Ctrl+V, Shift+Insert, Direct
- 虚拟键码: VK_V (0x56), VK_INSERT (0x2D)
- 键盘布局: 完全兼容

## 相关文件

### 后端
- `src-tauri/src/shortcut.rs` - 快捷键注册和管理
- `src-tauri/src/input.rs` - 输入模拟实现
- `src-tauri/src/settings.rs` - 快捷键配置

### 前端
- `src/components/settings/ShortcutSettings.tsx` - 快捷键设置界面
- `src/stores/settingsStore.ts` - 设置状态管理

### 依赖
- `tauri-plugin-global-shortcut` - 全局快捷键支持
- `enigo` - 跨平台输入模拟

## 相关文档

- [enigo 文档](https://docs.rs/enigo/)
- [tauri-plugin-global-shortcut](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/global-shortcut)
- [Windows 虚拟键码](https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes)
- [macOS 键码](https://eastmanreference.com/complete-list-of-applescript-key-codes)
- [Linux keysym](https://www.x.org/releases/current/doc/xproto/x11protocol.html#keysym_encoding)
