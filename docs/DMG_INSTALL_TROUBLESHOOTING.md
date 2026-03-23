# DMG 安装问题排查指南

## 常见问题

### 问题 1：macOS Gatekeeper 安全提示 ⚠️

**症状**：双击 `Install.command` 时显示：
> Apple 无法验证 "Install.command" 是否包含可能危害 Mac 安全或泄漏隐私的恶意软件

**快速解决方案**：

方案 A - 右键打开（最简单，无需终端）：
1. **右键点击** `Install.command`
2. 选择「打开」
3. 在弹窗中再点「打开」

方案 B - 使用终端：
```bash
/Volumes/KeVoiceInput/Install.command
```

方案 C - 手动安装：
1. 拖拽 `KeVoiceInput.app` 到 `Applications`
2. 打开终端：`xattr -cr /Applications/KeVoiceInput.app`
3. 右键点击应用 → 打开

**详细解决方案**：参见 [GATEKEEPER_WORKAROUND.md](GATEKEEPER_WORKAROUND.md)

---

### 问题 2：双击安装脚本路径错误

### 症状

在其他 Mac 设备上双击 DMG 中的安装脚本时，终端显示类似以下错误：

```
zsh: no such file or directory: Volumes/KeVoiceInput/安装到应用程序.command
```

### 原因

1. **中文文件名问题**：某些 Mac 系统配置下，终端无法正确解析路径中的中文字符
2. **路径缺少前缀**：错误信息显示路径缺少开头的 `/`（`Volumes/...` 而不是 `/Volumes/...`）

### 解决方案

#### 方案 1：使用新版 DMG（推荐）✅

从 2026-02-22 开始，构建的 DMG 使用英文文件名 `Install.command`，避免了中文路径问题。

**如何获取新版 DMG**：
```bash
# 重新构建
bun run tauri:build

# 新的 DMG 会包含 Install.command 而不是 安装到应用程序.command
```

#### 方案 2：手动安装（适用于旧版 DMG）

如果使用的是旧版 DMG（带中文文件名），可以手动安装：

1. **打开 DMG**：双击打开 `KeVoiceInput_0.0.1_aarch64.dmg`

2. **拖拽安装**：
   - 将 `KeVoiceInput.app` 拖到 `Applications` 文件夹

3. **移除隔离属性**（重要！）：
   打开终端，运行以下命令：
   ```bash
   # 移除隔离属性，允许应用直接启动
   xattr -cr /Applications/KeVoiceInput.app
   ```

4. **首次启动**：
   - 在 Finder 中找到应用
   - **右键点击** → 选择"打开"
   - 在弹出的对话框中点击"打开"

#### 方案 3：使用终端手动运行安装脚本

如果需要使用安装脚本但双击失败，可以通过终端手动运行：

```bash
# 1. 打开终端
# 2. 运行以下命令（注意使用正确的完整路径）

/Volumes/KeVoiceInput/安装到应用程序.command

# 或者先 cd 到目录
cd /Volumes/KeVoiceInput
./安装到应用程序.command
```

### 验证 DMG 版本

检查你的 DMG 使用的是哪个版本的安装脚本：

```bash
# 挂载 DMG
hdiutil attach KeVoiceInput_0.0.1_aarch64.dmg

# 查看内容
ls -la /Volumes/KeVoiceInput/

# 如果看到 "Install.command" → 新版（英文文件名）
# 如果看到 "安装到应用程序.command" → 旧版（中文文件名）

# 卸载 DMG
hdiutil detach /Volumes/KeVoiceInput
```

## 为什么更改为英文文件名？

虽然 macOS 完全支持中文文件名，但在某些情况下：

1. **终端环境问题**：不同的 shell 配置（bash、zsh）和 locale 设置可能导致路径解析问题
2. **`.command` 文件特殊性**：`.command` 文件双击时由 Terminal.app 执行，路径传递可能受到环境变量影响
3. **兼容性考虑**：英文文件名在所有 Mac 系统上都能保证正常工作

## 开发者注意事项

如果你要分发自己构建的 DMG：

1. **确保使用最新构建脚本**：
   ```bash
   # scripts/create-dmg.sh 现在使用 Install.command
   bun run tauri:build
   ```

2. **验证 DMG 内容**：
   ```bash
   hdiutil attach src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg
   ls -la /Volumes/KeVoiceInput/
   # 应该看到 Install.command
   ```

3. **测试安装脚本**：
   在干净的 Mac 环境中测试双击 `Install.command` 是否正常工作

## 相关文件

- 新版安装脚本：`scripts/install-app.command`
- DMG 创建脚本：`scripts/create-dmg.sh`
- 构建包装脚本：`scripts/tauri-build-wrapper.sh`

## 更新日志

- **2026-02-22**：将安装脚本文件名从 `安装到应用程序.command` 改为 `Install.command`
- **原因**：解决某些 Mac 设备上中文路径解析问题
- **影响**：所有新构建的 DMG 都使用英文文件名
