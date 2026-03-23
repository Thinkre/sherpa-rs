# DMG 安装说明

## 打开 DMG 后的界面

现在打开 DMG 文件后，你会看到一个专业的安装界面：

```
┌─────────────────────────────────────────┐
│  KeVoiceInput                      ⚫ ⚪ 🔴 │
├─────────────────────────────────────────┤
│                                         │
│  ┌──────────┐        ┌──────────┐      │
│  │          │        │          │      │
│  │   📱     │   →    │   📁     │      │
│  │ KeVoice  │        │  Apps    │      │
│  │  Input   │        │          │      │
│  └──────────┘        └──────────┘      │
│                                         │
│           ┌──────────────┐             │
│           │  📝 一键安装  │             │
│           └──────────────┘             │
│                                         │
└─────────────────────────────────────────┘
```

## 安装步骤

### 方法 1：一键自动安装（推荐）⚡

1. **双击 DMG 文件**
   - 文件名：`KeVoiceInput_0.0.1_aarch64.dmg`
   - 位置：`src-tauri/target/release/bundle/dmg/`

2. **双击"安装到应用程序.command"**
   - 在 DMG 窗口底部中间位置
   - 会打开终端窗口自动完成安装

3. **按提示操作**
   - 选择安装位置（推荐选择 1）
   - 如需要输入密码，输入你的 macOS 密码
   - 自动移除隔离属性，可直接启动

4. **完成！**
   - 脚本会询问是否立即启动
   - 或从 Launchpad 搜索 "KeVoice" 启动

### 方法 2：手动拖拽安装

1. **双击 DMG 文件**

2. **拖动图标**
   - 将左上方的 KeVoiceInput.app 图标
   - 拖动到右上方的 Applications 文件夹

3. **等待复制完成**
   - macOS 会自动复制应用到 /Applications
   - 复制完成后可以弹出 DMG

4. **首次启动需要放行**
   - 右键点击应用，选择"打开"
   - 在弹出对话框中点击"打开"

## 首次启动

首次启动时，macOS 可能会显示安全提示：

```
"KeVoiceInput" 是从互联网下载的应用程序。
您确定要打开它吗？
```

点击"打开"即可。

如果看到"无法打开"的提示：
1. 右键点击应用
2. 选择"打开"
3. 在弹出的对话框中点击"打开"

## 卸载

要卸载应用：
1. 打开 Finder
2. 进入 Applications 文件夹
3. 将 KeVoiceInput.app 拖到废纸篓
4. 清空废纸篓

用户数据位置（可选删除）：
```
~/Library/Application Support/com.kevoiceinput.app/
```

## 常见问题

### Q: DMG 打开后只看到一个图标，没有 Applications 文件夹？

A: 这是旧版本的问题。请重新构建：
```bash
bun run tauri:build
```

新版本会自动创建专业的安装界面。

### Q: 拖动后应用在哪里？

A: 应用被安装到了 `/Applications/KeVoiceInput.app`。
你可以在 Finder 的 Applications 文件夹中找到它。

### Q: 可以安装到其他位置吗？

A: 技术上可以，但不推荐。macOS 应用通常应该安装在 /Applications。
如果安装到其他位置，某些功能可能无法正常工作。

### Q: 需要管理员权限吗？

A: 安装到 /Applications 通常不需要管理员权限。
但如果系统要求密码，请输入你的 macOS 用户密码。

## DMG 技术细节

- **格式**: UDZO (压缩)
- **文件系统**: APFS
- **大小**: ~32-34MB（压缩后）
- **内容**: 
  - KeVoiceInput.app (包含所有动态库)
  - Applications 符号链接 (指向 /Applications)

## 构建自己的 DMG

如果你想自定义 DMG：

1. 修改 `scripts/create-dmg.sh`
2. 可以调整：
   - 窗口大小和位置
   - 图标大小和位置
   - 背景图片（需要额外设置）
   - 卷标名称

3. 重新构建：
```bash
bun run tauri:build
```

## 相关文档

- [QUICK_BUILD.md](../QUICK_BUILD.md) - 快速构建指南
- [BUILD_GUIDE.md](BUILD_GUIDE.md) - 完整构建指南
