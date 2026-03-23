# macOS Gatekeeper 安全提示解决方案

## 问题描述

在其他 Mac 设备上双击 `Install.command` 时，macOS 显示：

> Apple 无法验证 "Install.command" 是否包含可能危害 Mac 安全或泄漏隐私的恶意软件

这是 macOS Gatekeeper 的安全机制，会阻止执行未签名或来源不明的脚本。

## 为什么会出现这个问题？

1. **未进行代码签名**：`.command` 脚本没有使用 Apple Developer 证书签名
2. **隔离属性**：从互联网下载的文件会被标记为"隔离"，macOS 会额外审查
3. **Gatekeeper 策略**：macOS Catalina (10.15) 之后加强了安全检查

## 解决方案（按推荐顺序）

### 方案 1：右键 → 打开（最简单，无需终端）✅

1. **右键点击** `Install.command`
2. 选择「打开」
3. 在弹窗中点击「打开」

**优点**：无需打开终端，系统自带方式，安全可控。

---

### 方案 2：使用终端直接运行

打开终端，运行：

```bash
/Volumes/KeVoiceInput/Install.command
```

**步骤**：打开 DMG → 打开终端（⌘+Space，输入 "Terminal"）→ 粘贴命令回车

---

### 方案 3：手动拖拽安装（最安全）🔒

不使用脚本，直接手动安装：

```bash
# 1. 拖拽应用到 Applications 文件夹

# 2. 打开终端，移除隔离属性
xattr -cr /Applications/KeVoiceInput.app

# 3. 首次启动：右键点击应用 → 打开 → 点击"打开"按钮
```

**优点**：
- 最传统最安全的方式
- 用户完全控制每一步
- 不依赖脚本

---

### 方案 4：移除隔离属性后再双击

先移除脚本的隔离属性，然后双击：

```bash
# 1. 打开终端
# 2. 移除脚本的隔离属性
xattr -cr /Volumes/KeVoiceInput/Install.command

# 3. 双击 Install.command
```

**优点**：
- 仍然可以使用自动安装脚本
- 只需要一次操作

---

### 方案 5：临时禁用 Gatekeeper（不推荐）⚠️

**警告**：这会降低系统安全性，不推荐使用！

```bash
# 禁用 Gatekeeper
sudo spctl --master-disable

# 安装完成后，记得重新启用
sudo spctl --master-enable
```

---

## DMG 中包含的文件

新版 DMG 包含以下文件来帮助用户：

1. **Install.command**：自动安装脚本（推荐，但可能被 Gatekeeper 阻止）
2. **manual-install.sh**：备用安装脚本（可从终端运行）
3. **README.txt**：详细安装说明
4. **KeVoiceInput.app**：应用本体
5. **Applications**：应用程序文件夹快捷方式

## 使用 README.txt

DMG 中包含 `README.txt` 文件，双击即可查看完整安装说明，包括：
- 自动安装方法
- 手动安装方法
- 故障排查步骤
- Terminal 命令行说明

## 开发者解决方案

要彻底解决「其他 Mac 上双击 Install.command 被阻止」的问题，需要**对安装脚本签名**并**对 DMG 公证**。公证后，用户在其他 Mac 上下载并打开 DMG 时，不再出现“无法验证是否包含恶意软件”的提示。

### 构建时自动签名安装脚本

在创建 DMG 时设置环境变量 `MACOS_INSTALLER_SIGNING_IDENTITY`，`scripts/create-dmg.sh` 会自动对 `Install.command` 和 `manual-install.sh` 进行签名（需配合 DMG 公证使用）：

```bash
export MACOS_INSTALLER_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)"
bun run tauri:build
# 或单独创建 DMG 时：
# ./scripts/create-dmg.sh path/to/KeVoiceInput.app path/to/output.dmg
```

### 手动对脚本进行代码签名

```bash
# 1. 获取 Apple Developer 证书
# 2. 签名脚本
codesign --sign "Developer ID Application: Your Name (TEAM_ID)" \
         --timestamp \
         --force \
         scripts/install-app.command

# 3. 验证签名
codesign -vvv scripts/install-app.command
```

### 对整个 DMG 进行公证

```bash
# 1. 签名 DMG
codesign --sign "Developer ID Application: Your Name (TEAM_ID)" \
         --timestamp \
         KeVoiceInput_0.0.1_aarch64.dmg

# 2. 提交公证
xcrun notarytool submit KeVoiceInput_0.0.1_aarch64.dmg \
      --apple-id "your@email.com" \
      --team-id "TEAM_ID" \
      --password "app-specific-password" \
      --wait

# 3. 装订公证票据
xcrun stapler staple KeVoiceInput_0.0.1_aarch64.dmg
```

### 所需材料

- Apple Developer Program 会员资格（$99/年）
- Developer ID Application 证书
- App-specific password（App 专用密码）

## 用户反馈和文档

建议在分发 DMG 时附带以下说明：

1. **README.txt 在 DMG 中**：打开 DMG 就能看到详细说明
2. **GitHub Release 说明**：在 Release 页面添加安装指南
3. **首次使用文档**：在应用内或网站上提供详细教程

## 自动化解决方案

可以修改 `tauri.conf.json` 配置 Tauri 的签名流程：

```json
{
  "bundle": {
    "macOS": {
      "signingIdentity": "Developer ID Application: Your Name (TEAM_ID)",
      "entitlements": "Entitlements.plist",
      "hardenedRuntime": true,
      "notarize": {
        "appleId": "your@email.com",
        "appleIdPassword": "@keychain:AC_PASSWORD",
        "teamId": "TEAM_ID"
      }
    }
  }
}
```

这样每次 `bun run tauri:build` 都会自动签名和公证。若希望 DMG 内的 `Install.command` 也被签名（避免其他 Mac 上出现安全提示），在构建前设置：

```bash
export MACOS_INSTALLER_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)"
```

## 常见问题

### Q: 为什么不默认进行代码签名？

A: 代码签名需要：
- Apple Developer Program 会员（$99/年）
- 实名认证
- 证书配置

对于开源项目或个人开发，这个门槛较高。

### Q: 用户觉得手动安装太麻烦怎么办？

A: 提供清晰的说明：
1. DMG 中包含 README.txt
2. Release 页面添加安装视频或 GIF
3. 在应用首页提供 FAQ

### Q: 能否绕过 Gatekeeper？

A: 不建议。应该：
1. 提供简单的 Terminal 命令
2. 或使用手动拖拽安装
3. 长期方案是进行代码签名

## 测试清单

在不同环境测试 DMG：

- [ ] 在构建机器上测试
- [ ] 在没有开发工具的 Mac 上测试
- [ ] 在不同 macOS 版本上测试（Catalina, Big Sur, Monterey, Ventura, Sonoma）
- [ ] 从"下载"文件夹打开（会被标记隔离）
- [ ] 使用不同的 shell（bash, zsh）
- [ ] 测试 README.txt 的可读性

## 参考资料

- [Apple Code Signing Guide](https://developer.apple.com/library/archive/documentation/Security/Conceptual/CodeSigningGuide/)
- [Notarizing macOS Software](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
- [Gatekeeper and Runtime Protection](https://support.apple.com/guide/security/gatekeeper-and-runtime-protection-sec5599b66df/)
