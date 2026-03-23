# 跨平台托盘图标实现

## 概述

KeVoiceInput 使用系统托盘图标提供快速访问和状态指示。托盘图标已完全实现跨平台支持（macOS、Windows、Linux），并自动适应系统主题。

## 平台差异

| 特性 | macOS | Windows | Linux |
|------|-------|---------|-------|
| 位置 | 菜单栏（顶部） | 系统托盘（右下角） | 取决于桌面环境 |
| 主题自适应 | ✅ 自动 | ✅ 自动 | ⚠️ 使用彩色图标 |
| 图标格式 | PNG（Template） | PNG/ICO | PNG |
| 右键菜单 | ✅ 支持 | ✅ 支持 | ✅ 支持 |
| 状态动画 | ✅ 三种状态 | ✅ 三种状态 | ✅ 三种状态 |

## 图标状态

### 三种图标状态

| 状态 | 说明 | 触发条件 |
|------|------|---------|
| **Idle** | 空闲状态 | 应用启动、转录完成 |
| **Recording** | 录音中 | 按下快捷键开始录音 |
| **Transcribing** | 转录中 | 录音结束，正在处理 |

### 图标文件

**位置**：`src-tauri/resources/`

#### macOS / Windows（主题自适应）

| 状态 | 深色主题（亮图标） | 浅色主题（暗图标） |
|------|-------------------|-------------------|
| Idle | `tray_idle.png` | `tray_idle_dark.png` |
| Recording | `tray_recording.png` | `tray_recording_dark.png` |
| Transcribing | `tray_transcribing.png` | `tray_transcribing_dark.png` |

#### Linux（彩色图标）

| 状态 | 文件 |
|------|------|
| Idle | `voice_input.png` |
| Recording | `recording.png` |
| Transcribing | `transcribing.png` |

**为什么 Linux 使用彩色图标？**
- 不同桌面环境对托盘图标的处理不一致
- 彩色图标在所有环境下都清晰可见
- 避免在浅色/深色主题切换时出现问题

## 代码实现

### 主题检测

**文件**：`src-tauri/src/tray.rs`

```rust
pub fn get_current_theme(app: &AppHandle) -> AppTheme {
    if cfg!(target_os = "linux") {
        // Linux 总是使用彩色主题
        AppTheme::Colored
    } else {
        // macOS/Windows 检测系统主题
        if let Some(main_window) = app.get_webview_window("main") {
            match main_window.theme().unwrap_or(Theme::Dark) {
                Theme::Light => AppTheme::Light,
                Theme::Dark => AppTheme::Dark,
                _ => AppTheme::Dark,
            }
        } else {
            AppTheme::Dark
        }
    }
}
```

### 图标路径选择

```rust
pub fn get_icon_path(theme: AppTheme, state: TrayIconState) -> &'static str {
    match (theme, state) {
        // 深色主题使用亮图标
        (AppTheme::Dark, TrayIconState::Idle) => "resources/tray_idle.png",
        (AppTheme::Dark, TrayIconState::Recording) => "resources/tray_recording.png",
        (AppTheme::Dark, TrayIconState::Transcribing) => "resources/tray_transcribing.png",

        // 浅色主题使用暗图标
        (AppTheme::Light, TrayIconState::Idle) => "resources/tray_idle_dark.png",
        (AppTheme::Light, TrayIconState::Recording) => "resources/tray_recording_dark.png",
        (AppTheme::Light, TrayIconState::Transcribing) => "resources/tray_transcribing_dark.png",

        // 彩色主题（Linux）
        (AppTheme::Colored, TrayIconState::Idle) => "resources/voice_input.png",
        (AppTheme::Colored, TrayIconState::Recording) => "resources/recording.png",
        (AppTheme::Colored, TrayIconState::Transcribing) => "resources/transcribing.png",
    }
}
```

### 图标切换

```rust
pub fn change_tray_icon(app: &AppHandle, icon: TrayIconState) {
    let tray = app.state::<TrayIcon>();
    let theme = get_current_theme(app);
    let icon_path = get_icon_path(theme, icon.clone());

    let _ = tray.set_icon(Some(
        Image::from_path(
            app.path()
                .resolve(icon_path, tauri::path::BaseDirectory::Resource)
                .expect("failed to resolve"),
        )
        .expect("failed to set icon"),
    ));

    // 同时更新托盘菜单
    update_tray_menu(app, &icon, None);
}
```

### Template Mode（macOS）

macOS 使用 **Template Mode** 让系统自动调整图标颜色：

```rust
let _ = tray.set_icon_as_template(true);
```

**效果**：
- macOS 会自动将图标转换为单色
- 系统主题切换时自动反转颜色
- 需要图标有透明背景

## 托盘菜单

### 菜单项（所有平台）

| 菜单项 | 快捷键 | 说明 |
|--------|--------|------|
| 🎤 KeVoiceInput v0.0.1 | - | 版本信息（不可点击） |
| **Show Home** | - | 显示主窗口 |
| **Cancel** | - | 取消录音（仅录音时显示） |
| **Copy Last Transcript** | - | 复制最后一次转录 |
| **Add Hotword** | - | 添加热词 |
| **Settings** | Cmd+, / Ctrl+, | 打开设置 |
| **Check Updates** | - | 检查更新 |
| **Quit** | Cmd+Q / Ctrl+Q | 退出应用 |

### 平台特定快捷键

```rust
#[cfg(target_os = "macos")]
let (settings_accelerator, quit_accelerator) = (Some("Cmd+,"), Some("Cmd+Q"));

#[cfg(not(target_os = "macos"))]
let (settings_accelerator, quit_accelerator) = (Some("Ctrl+,"), Some("Ctrl+Q"));
```

### 状态感知菜单

托盘菜单根据应用状态动态调整：

**Idle 状态**：标准菜单

**Recording/Transcribing 状态**：添加"Cancel"菜单项

## 国际化支持

托盘菜单支持 14 种语言，使用 `tray_i18n.rs` 提供翻译：

```rust
let strings = get_tray_translations(Some(locale.to_string()));

let show_home_i = MenuItem::with_id(
    app,
    "show_home",
    &strings.show_home,  // 翻译后的文本
    true,
    None::<&str>,
).expect("failed to create show home item");
```

**支持语言**：
- 简体中文、繁体中文、英语、日语、韩语、西班牙语、法语、德语、意大利语、葡萄牙语、俄语、阿拉伯语、土耳其语、越南语

## Windows 特定注意事项

### 图标尺寸

Windows 托盘图标推荐尺寸：
- **16x16** 像素（标准 DPI）
- **32x32** 像素（高 DPI）
- **48x48** 像素（超高 DPI）

当前使用的 PNG 图标会自动缩放。

### ICO 格式（可选）

虽然 PNG 可用，但 ICO 格式在 Windows 上更原生：

```rust
// 可选：使用 ICO 文件
#[cfg(target_os = "windows")]
let icon_path = "resources/tray_idle.ico";

#[cfg(not(target_os = "windows"))]
let icon_path = "resources/tray_idle.png";
```

**当前实现**：统一使用 PNG，Windows 自动处理。

### 高 DPI 支持

Windows 10+ 自动缩放托盘图标，无需额外配置。

### 托盘图标持久性

Windows 托盘图标会在以下情况下暂时消失：
- 资源管理器（explorer.exe）重启
- 系统重启

Tauri 会自动重新创建图标，无需手动处理。

## Linux 桌面环境兼容性

### 测试的桌面环境

| 环境 | 托盘图标 | 菜单 | 主题切换 |
|------|---------|------|---------|
| **GNOME** | ✅ 需要扩展 | ✅ | ✅ |
| **KDE Plasma** | ✅ 原生支持 | ✅ | ✅ |
| **XFCE** | ✅ 原生支持 | ✅ | ✅ |
| **Cinnamon** | ✅ 原生支持 | ✅ | ✅ |
| **MATE** | ✅ 原生支持 | ✅ | ✅ |

**GNOME 注意事项**：
- GNOME 3+ 默认不显示托盘图标
- 需要安装扩展：[AppIndicator Support](https://extensions.gnome.org/extension/615/appindicator-support/)

### Ubuntu 22.04+ 默认配置

Ubuntu 22.04+ 使用 GNOME，需要：
```bash
# 安装 gnome-shell-extension-appindicator
sudo apt install gnome-shell-extension-appindicator

# 启用扩展
gnome-extensions enable ubuntu-appindicators@ubuntu.com
```

## 图标设计指南

### 通用原则

1. **简洁清晰**：16x16 像素下也能识别
2. **透明背景**：PNG 格式带 Alpha 通道
3. **单色为主**：macOS Template Mode 需要
4. **状态明显**：三种状态应易于区分

### macOS 图标要求

- **尺寸**：推荐 22x22 像素（视网膜屏 44x44）
- **颜色**：黑色或白色，透明背景
- **格式**：PNG with Alpha
- **Template Mode**：图标会被系统自动着色

### Windows 图标要求

- **尺寸**：16x16、32x32、48x48（多尺寸）
- **颜色**：全彩或单色
- **格式**：PNG 或 ICO
- **对比度**：确保在浅色和深色背景下都清晰

### Linux 图标要求

- **尺寸**：建议 48x48 像素
- **颜色**：推荐使用彩色（避免主题问题）
- **格式**：PNG
- **兼容性**：在各桌面环境测试

## 故障排查

### macOS: 图标不显示

**原因**：
1. 权限问题
2. 图标文件路径错误
3. Template Mode 失效

**解决方案**：
```bash
# 检查资源文件
ls -la src-tauri/resources/tray*.png

# 重启托盘（杀死菜单栏进程）
killall SystemUIServer

# 检查应用日志
tail -f ~/Library/Logs/KeVoiceInput/app.log
```

### Windows: 图标显示为默认图标

**原因**：
1. 图标文件未打包到安装程序
2. 图标尺寸不合适
3. 资源路径错误

**解决方案**：
```powershell
# 检查打包的资源
cd src-tauri/target/release/bundle/msi
# 解压 MSI 检查 resources/ 目录

# 验证图标文件
Get-ChildItem -Path "src-tauri\resources\tray*.png"
```

### Linux: GNOME 不显示托盘图标

**原因**：GNOME 默认禁用托盘图标。

**解决方案**：
```bash
# 安装 AppIndicator 扩展
sudo apt install gnome-shell-extension-appindicator

# 启用扩展
gnome-extensions enable ubuntu-appindicators@ubuntu.com

# 重启 GNOME Shell
Alt+F2 → 输入 r → Enter
```

### 主题切换后图标不更新

**问题**：系统主题切换后，图标仍显示旧主题。

**解决方案**：
- 这是已知限制，需要重启应用
- 未来可以监听系统主题变化事件并自动更新

## 性能优化

### 图标缓存

Tauri 会缓存加载的图标，频繁切换状态不会重复加载：

```rust
// 图标只在状态改变时加载
pub fn change_tray_icon(app: &AppHandle, icon: TrayIconState) {
    // ... 加载并设置图标
}
```

### 图标文件大小

优化建议：
- PNG 文件使用 `pngquant` 压缩
- 移除不必要的元数据
- 避免过大文件（推荐 < 10KB）

当前图标大小：
```
tray_idle.png:         ~2KB  ✅ 优秀
tray_recording.png:    ~2KB  ✅ 优秀
tray_transcribing.png: ~2KB  ✅ 优秀
voice_input.png:       ~240KB ⚠️ 可优化
```

## 测试清单

发布前验证：

- [ ] **macOS**：深色模式下图标清晰
- [ ] **macOS**：浅色模式下图标清晰
- [ ] **macOS**：三种状态图标正确切换
- [ ] **macOS**：右键菜单正常工作
- [ ] **macOS**：快捷键有效（Cmd+,、Cmd+Q）
- [ ] **Windows**：系统托盘图标显示
- [ ] **Windows**：三种状态图标正确切换
- [ ] **Windows**：右键菜单正常工作
- [ ] **Windows**：快捷键有效（Ctrl+,、Ctrl+Q）
- [ ] **Linux (KDE)**：图标显示且彩色
- [ ] **Linux (GNOME + 扩展)**：图标显示
- [ ] **Linux**：菜单正常工作

## 未来改进

### 动画图标

可以实现录音和转录状态的动画效果：
- Recording：脉冲动画
- Transcribing：旋转动画

技术方案：定时器 + 多帧图标切换

### 图标气泡提示

Windows/Linux 支持托盘气泡提示：
```rust
tray.set_tooltip(Some("KeVoiceInput - 录音中..."));
```

### 自定义图标颜色

允许用户选择图标颜色主题（类似 IDE 主题）。

## 总结

✅ **托盘图标已完全跨平台**

**关键特性**：
- macOS/Windows 自动主题适应
- Linux 使用彩色图标避免兼容性问题
- 三种状态图标清晰区分
- 国际化菜单支持 14 种语言
- Template Mode 确保 macOS 原生外观

**Windows 适配状态**: ✅ 已完全支持
- 图标正确显示在系统托盘
- 菜单快捷键使用 Ctrl
- PNG 图标自动缩放适应 DPI
- 所有功能与 macOS 一致

## 相关文件

### 后端
- `src-tauri/src/tray.rs` - 托盘图标逻辑
- `src-tauri/src/tray_i18n.rs` - 托盘菜单国际化
- `src-tauri/src/app_menu.rs` - 应用菜单处理

### 资源
- `src-tauri/resources/tray*.png` - 托盘图标文件
- `src-tauri/resources/voice_input.png` - Linux 彩色图标
- `src-tauri/resources/recording.png` - Linux 录音图标
- `src-tauri/resources/transcribing.png` - Linux 转录图标

### 配置
- `src-tauri/tauri.conf.json` - Tauri 配置（资源打包）

## 相关文档

- [Tauri Tray Icon API](https://tauri.app/v2/reference/javascript/api/namespacecore/#trayicon)
- [macOS Human Interface Guidelines - Menu Bar Extras](https://developer.apple.com/design/human-interface-guidelines/menu-bar-extras)
- [Windows Notification Area Guidelines](https://docs.microsoft.com/en-us/windows/win32/shell/notification-area)
