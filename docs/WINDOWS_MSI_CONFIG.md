# Windows MSI 安装包配置指南

## 概述

KeVoiceInput 使用 WiX Toolset (Windows Installer XML) 创建 MSI 安装包。Tauri 2.x 自动集成 WiX，无需手动配置。

## 当前配置

### 基础配置

**位置**：`src-tauri/tauri.conf.json`

```json
{
  "productName": "KeVoiceInput",
  "version": "0.0.1",
  "identifier": "com.kevoiceinput.app",
  "bundle": {
    "active": true,
    "createUpdaterArtifacts": true,
    "targets": "all",
    "resources": ["resources/**/*"],
    "license": "MIT",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "windows": {
      "signCommand": "trusted-signing-cli -e https://eus.codesigning.azure.net/ -a CJ-Signing -c cjpais-dev -d KeVoiceInput %1"
    }
  }
}
```

### Cargo 元数据

**位置**：`src-tauri/Cargo.toml`

```toml
[package]
name = "kevoiceinput"
version = "0.0.1"
description = "KeVoiceInput - Premium Voice Input Application"
authors = ["cjpais"]
edition = "2021"
license = "MIT"
```

**作用**：
- `name`：二进制文件名（kevoiceinput.exe）
- `version`：安装包版本
- `description`：显示在程序列表中
- `license`：许可证类型

## MSI 配置选项

### 完整配置示例

可以在 `tauri.conf.json` 的 `bundle.windows` 中添加更多选项：

```json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": null,
      "tsp": false,
      "webviewInstallMode": {
        "type": "downloadBootstrapper"
      },
      "allowDowngrades": false,
      "wix": {
        "language": "en-US",
        "template": null,
        "fragmentPaths": [],
        "componentRefs": [],
        "componentGroupRefs": [],
        "featureGroupRefs": [],
        "featureRefs": [],
        "mergeRefs": [],
        "skipWebviewInstall": false,
        "license": null,
        "enableElevatedUpdateTask": false,
        "bannerPath": null,
        "dialogImagePath": null
      },
      "nsis": null,
      "signCommand": "trusted-signing-cli ..."
    }
  }
}
```

### 关键选项说明

#### 1. certificateThumbprint

**说明**：代码签名证书指纹

**用途**：使用 Windows 证书存储中的证书签名

**示例**：
```json
"certificateThumbprint": "ABC123456789..."
```

**当前状态**：使用 `signCommand` 代替

#### 2. digestAlgorithm

**说明**：签名哈希算法

**选项**：`sha1`、`sha256`（推荐）

**当前设置**：默认 `sha256`

#### 3. timestampUrl

**说明**：时间戳服务器 URL

**用途**：确保签名在证书过期后仍有效

**示例**：
```json
"timestampUrl": "http://timestamp.digicert.com"
```

#### 4. webviewInstallMode

**说明**：WebView2 安装方式

**选项**：
- `downloadBootstrapper`：下载引导程序（推荐，体积小）
- `embedBootstrapper`：嵌入引导程序（离线安装）
- `offlineInstaller`：嵌入完整安装包（体积大，完全离线）
- `fixedRuntime`：使用固定版本运行时
- `skip`：跳过安装（假设已安装）

**当前设置**：`downloadBootstrapper`

**建议**：
- 企业/离线环境：使用 `offlineInstaller`
- 互联网环境：使用 `downloadBootstrapper`（默认）

#### 5. allowDowngrades

**说明**：是否允许降级安装

**默认**：`false`

**用途**：允许用户安装旧版本

**注意**：通常保持 `false` 避免混乱

#### 6. wix 配置

##### 6.1 language

**说明**：安装程序界面语言

**选项**：
- `en-US`（英语）
- `zh-CN`（简体中文）
- `zh-TW`（繁体中文）
- `ja-JP`（日语）
- 等等

**示例**：
```json
"language": "zh-CN"
```

**多语言支持**：
```json
"language": ["en-US", "zh-CN", "ja-JP"]
```

##### 6.2 license

**说明**：许可证协议文件路径（RTF 格式）

**示例**：
```json
"license": "LICENSE.rtf"
```

**创建 RTF 许可证**：
1. 将 `LICENSE` 文件用 WordPad/Word 打开
2. 另存为 RTF 格式
3. 放置在 `src-tauri/` 目录

**当前状态**：未设置（跳过许可证页面）

##### 6.3 bannerPath

**说明**：安装程序顶部横幅图片

**尺寸**：493x58 像素

**格式**：BMP（推荐）、PNG

**示例**：
```json
"bannerPath": "assets/banner.bmp"
```

##### 6.4 dialogImagePath

**说明**：安装程序侧边栏图片

**尺寸**：493x312 像素

**格式**：BMP（推荐）、PNG

**示例**：
```json
"dialogImagePath": "assets/dialog.bmp"
```

##### 6.5 enableElevatedUpdateTask

**说明**：创建提升权限的更新任务

**用途**：允许自动更新无需 UAC 提示

**默认**：`false`

**示例**：
```json
"enableElevatedUpdateTask": true
```

**注意**：需要应用签名

## 推荐的增强配置

### 配置 1：基础中文支持

```json
{
  "bundle": {
    "windows": {
      "webviewInstallMode": {
        "type": "downloadBootstrapper"
      },
      "wix": {
        "language": "zh-CN"
      },
      "signCommand": "trusted-signing-cli ..."
    }
  }
}
```

### 配置 2：离线安装包

```json
{
  "bundle": {
    "windows": {
      "webviewInstallMode": {
        "type": "offlineInstaller"
      },
      "wix": {
        "language": ["zh-CN", "en-US"]
      }
    }
  }
}
```

**注意**：安装包会增加约 100MB

### 配置 3：企业级完整配置

```json
{
  "bundle": {
    "windows": {
      "webviewInstallMode": {
        "type": "offlineInstaller"
      },
      "allowDowngrades": false,
      "wix": {
        "language": ["zh-CN", "en-US"],
        "license": "LICENSE.rtf",
        "bannerPath": "assets/banner.bmp",
        "dialogImagePath": "assets/dialog.bmp",
        "enableElevatedUpdateTask": true
      },
      "signCommand": "signtool sign /tr http://timestamp.digicert.com /td sha256 /fd sha256 \"%1\""
    }
  }
}
```

## 安装包结构

### 默认安装位置

- **64位 Windows**：`C:\Program Files\KeVoiceInput\`
- **32位 Windows**：`C:\Program Files (x86)\KeVoiceInput\`

### 文件布局

```
C:\Program Files\KeVoiceInput\
├── kevoiceinput.exe         # 主程序
├── resources\                # 资源文件
│   ├── tray_idle.png
│   ├── models\
│   └── ...
├── WebView2\                 # WebView2 运行时（如果使用 fixedRuntime）
└── ...
```

### 注册表项

MSI 安装程序会创建以下注册表项：

**安装信息**：
```
HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\{GUID}
```

**自动启动**（如果启用）：
```
HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run
KeVoiceInput = "C:\Program Files\KeVoiceInput\kevoiceinput.exe"
```

**文件关联**（如果配置）：
```
HKEY_CLASSES_ROOT\.kevoice
```

## 代码签名

### 方法 1：Azure Trusted Signing CLI（当前使用）

**配置**：
```json
"signCommand": "trusted-signing-cli -e https://eus.codesigning.azure.net/ -a CJ-Signing -c cjpais-dev -d KeVoiceInput %1"
```

**优点**：
- 云端证书管理
- 无需本地证书
- 支持 CI/CD

### 方法 2：SignTool（本地证书）

**配置**：
```json
"signCommand": "signtool sign /f \"path\\to\\cert.pfx\" /p PASSWORD /tr http://timestamp.digicert.com /td sha256 /fd sha256 \"%1\""
```

**参数说明**：
- `/f`：证书文件路径
- `/p`：证书密码
- `/tr`：时间戳服务器
- `/td`：时间戳哈希算法
- `/fd`：文件哈希算法
- `%1`：待签名文件（Tauri 自动传入）

### 方法 3：无签名（开发测试）

**配置**：
```json
"signCommand": null
```

或直接移除 `signCommand` 字段。

**注意**：无签名安装包会触发 Windows SmartScreen 警告。

## 安装程序外观定制

### 创建横幅图片

**工具**：Photoshop、GIMP、Paint.NET

**尺寸**：493x58 像素

**格式**：BMP 24位

**设计建议**：
- 使用品牌颜色
- 包含应用名称和图标
- 简洁清晰

**转换命令**（使用 ImageMagick）：
```bash
convert banner.png -resize 493x58! -type TrueColor BMP3:banner.bmp
```

### 创建对话框图片

**尺寸**：493x312 像素

**格式**：BMP 24位

**设计建议**：
- 展示应用截图或功能亮点
- 品牌一致性

## 高级功能

### 自定义安装步骤

可以通过 WiX Fragment 添加自定义逻辑。

**步骤**：
1. 创建 WiX 片段文件（.wxs）
2. 在 `tauri.conf.json` 中引用：
   ```json
   "wix": {
     "fragmentPaths": ["custom-install.wxs"]
   }
   ```

**示例**（custom-install.wxs）：
```xml
<?xml version="1.0" encoding="utf-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Fragment>
    <CustomAction Id="CreateShortcut"
                  Directory="DesktopFolder"
                  ExeCommand="[INSTALLDIR]kevoiceinput.exe"
                  Return="ignore" />
    <InstallExecuteSequence>
      <Custom Action="CreateShortcut" After="InstallFinalize" />
    </InstallExecuteSequence>
  </Fragment>
</Wix>
```

### 桌面快捷方式

Tauri 默认不创建桌面快捷方式，只创建开始菜单项。

**添加桌面快捷方式**：使用自定义 WiX Fragment（见上）

### 文件关联

如果需要关联特定文件类型（例如 `.kevoice`）：

**配置**：
```json
"fileAssociations": [
  {
    "ext": "kevoice",
    "name": "KeVoiceInput Recording",
    "description": "KeVoiceInput Audio Recording",
    "mimeType": "audio/x-kevoice",
    "role": "Editor"
  }
]
```

**注意**：需要 Tauri 2.x+

## 构建 MSI

### 开发构建

```powershell
# 使用构建脚本
.\scripts\build-windows.ps1

# 或直接运行 Tauri CLI
bun run tauri build
```

### 清理构建

```powershell
# 清理旧构建
.\scripts\build-windows.ps1 -Clean

# 或手动清理
Remove-Item -Recurse -Force src-tauri\target\release\
```

### 构建输出

**位置**：`src-tauri\target\release\bundle\msi\`

**文件**：
- `KeVoiceInput_0.0.1_x64_en-US.msi` - 安装包
- `KeVoiceInput_0.0.1_x64_en-US.msi.zip` - Tauri Updater 归档
- `KeVoiceInput_0.0.1_x64_en-US.msi.zip.sig` - 签名文件

## 测试安装包

### 本地测试

```powershell
# 安装
msiexec /i "path\to\KeVoiceInput_0.0.1_x64_en-US.msi" /l*v install.log

# 静默安装
msiexec /i "path\to\KeVoiceInput_0.0.1_x64_en-US.msi" /quiet /l*v install.log

# 卸载
msiexec /x "path\to\KeVoiceInput_0.0.1_x64_en-US.msi" /l*v uninstall.log

# 静默卸载
msiexec /x "path\to\KeVoiceInput_0.0.1_x64_en-US.msi" /quiet /l*v uninstall.log
```

### 检查日志

```powershell
# 查看安装日志
notepad install.log

# 搜索错误
Select-String -Path install.log -Pattern "error" -CaseSensitive:$false
```

### 验证安装

**检查清单**：
- [ ] 应用出现在"开始"菜单
- [ ] 可以从开始菜单启动应用
- [ ] 应用出现在"设置 → 应用"列表
- [ ] 所有资源文件正确安装
- [ ] 托盘图标正常显示
- [ ] 快捷键正常工作
- [ ] 语音转录功能正常
- [ ] 卸载后完全移除文件

## 故障排查

### 问题 1：WiX 构建失败

**错误**：
```
error: failed to bundle project: error running wix candle
```

**原因**：WiX Toolset 未安装或版本不兼容。

**解决方案**：
```powershell
# Tauri 会自动下载 WiX
# 如果失败，手动安装
winget install WiXToolset.WiX

# 或从官网下载
# https://wixtoolset.org/releases/
```

### 问题 2：图标未显示

**原因**：icon.ico 文件缺失或格式错误。

**解决方案**：
```powershell
# 检查图标文件
ls src-tauri\icons\icon.ico

# 使用在线工具转换 PNG 为 ICO
# https://convertio.co/png-ico/
```

### 问题 3：WebView2 安装失败

**错误**：用户报告应用启动后显示 "WebView2 Runtime not found"。

**原因**：WebView2 未安装且网络不可用。

**解决方案**：
- 使用 `offlineInstaller` 模式打包
- 或提供 WebView2 独立安装程序

### 问题 4：签名失败

**错误**：
```
error: failed to sign: Command 'trusted-signing-cli' not found
```

**解决方案**：
```powershell
# 确保签名工具可用
where.exe trusted-signing-cli

# 或临时移除签名
# 在 tauri.conf.json 中注释掉 signCommand
```

### 问题 5：安装后应用无法启动

**原因**：缺少 DLL 依赖（如 sherpa-onnx）。

**解决方案**：
- 确保 `build-windows.ps1` 正确复制了所有 DLL
- 检查 `src-tauri\target\release\` 中是否有所有必需的 DLL
- 使用 Dependency Walker 检查依赖

## 最佳实践

### 1. 版本号管理

使用语义化版本（SemVer）：
```
Major.Minor.Patch
例如：1.0.0, 0.1.0, 0.0.2
```

确保三个配置文件版本一致：
- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

使用 `scripts/sync-version.sh` 统一更新。

### 2. 签名所有构建

生产环境必须签名：
- 避免 Windows SmartScreen 警告
- 建立用户信任
- 支持自动更新

### 3. 提供离线安装包

为企业用户提供 `offlineInstaller` 版本。

### 4. 测试多种场景

- 全新安装
- 升级安装
- 降级安装（如果允许）
- 静默安装
- 卸载后重装

### 5. 日志记录

安装时记录详细日志用于调试：
```powershell
msiexec /i "installer.msi" /l*v install.log
```

## Windows 商店分发（未来）

虽然当前使用 MSI，未来可以考虑 Microsoft Store：

**优点**：
- 自动更新
- 用户信任
- 更简单的安装

**要求**：
- MSIX 打包格式
- 应用签名
- 商店审核

Tauri 2.x 支持 MSIX 打包：
```json
"bundle": {
  "targets": ["msi", "msix"]
}
```

## 相关资源

- [Tauri Bundle 配置](https://tauri.app/v2/reference/config/#bundleconfig)
- [WiX Toolset 文档](https://wixtoolset.org/documentation/)
- [Windows Installer 指南](https://docs.microsoft.com/en-us/windows/win32/msi/windows-installer-portal)
- [SignTool 文档](https://docs.microsoft.com/en-us/windows/win32/seccrypto/signtool)

## 相关文档

- [WINDOWS_QUICKSTART.md](WINDOWS_QUICKSTART.md) - Windows 快速开始
- [BUILD_GUIDE.md](BUILD_GUIDE.md) - 构建指南
- [CI_CD_WINDOWS.md](CI_CD_WINDOWS.md) - CI/CD Windows 构建
- [WINDOWS_PORT.md](WINDOWS_PORT.md) - Windows 适配技术细节
