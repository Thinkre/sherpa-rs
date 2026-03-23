# KeVoiceInput 仓库结构与命名规范

用于 [GitHub Thinkre/KeVoiceInput](https://github.com/Thinkre/KeVoiceInput) 的发布物与模型路径、命名规范，便于后续支持 Windows、iOS、Android、Linux 等多平台。

## 一、应用发布物（App Releases）

### 1.1 命名格式

```
KeVoiceInput-<version>-<platform>-<arch>.<ext>
```

| 字段       | 说明     | 示例 |
|------------|----------|------|
| `version`  | 语义化版本号 | `0.0.1`, `1.2.0` |
| `platform` | 平台标识（小写） | `macos`, `windows`, `linux`, `ios`, `android` |
| `arch`     | 架构（小写） | `aarch64`, `x86_64`, `armv7`, `i686`, `universal` |
| `ext`      | 安装包/压缩格式 | 见下表 |

### 1.2 各平台约定

| 平台 (platform) | 常见架构 (arch) | 安装包扩展名 | 说明 |
|-----------------|-----------------|--------------|------|
| macos           | aarch64, x86_64 | .dmg         | macOS 安装镜像 |
| macos           | aarch64, x86_64 | .app.tar.gz  | 应用更新包（Tauri updater） |
| windows         | x86_64, i686    | .msi, .exe   | Windows 安装包 |
| linux           | x86_64, aarch64 | .deb, .AppImage, .rpm | Linux 安装包 |
| ios             | aarch64, universal | .ipa       | iOS 安装包 |
| android         | aarch64, armv7, x86_64 | .apk, .aab | Android 安装包 |

### 1.3 在 GitHub 上的组织方式

- **GitHub Releases**：每个版本一个 Release，tag 为 `v<version>`（如 `v0.0.1`）。
- Release 的附件使用上述命名，例如：
  - `KeVoiceInput-0.0.1-macos-aarch64.dmg`
  - `KeVoiceInput-0.0.1-macos-aarch64.app.tar.gz`
  - `KeVoiceInput-0.0.1-macos-aarch64.app.tar.gz.sig`
  - `KeVoiceInput-0.0.1-windows-x86_64.msi`（后续）
  - `KeVoiceInput-0.0.1-linux-x86_64.AppImage`（后续）

### 1.4 更新检查（Updater）

- `latest.json` 的 `platforms` 键与 Tauri 约定一致：`darwin-aarch64`, `darwin-x86_64`, `win32-x86_64`, `linux-x86_64` 等。
- 更新包 URL 应指向规范命名的文件，例如：
  `https://github.com/Thinkre/KeVoiceInput/releases/download/v0.0.1/KeVoiceInput-0.0.1-macos-aarch64.app.tar.gz`

---

## 二、模型文件（Models）

### 2.1 命名格式

```
<model_id>.<ext>
或
<model_id>-<简短版本或日期>.<ext>
```

- `model_id`：与 `default_models.toml` / `models.toml` 中的 `id` 一致，或可读的英文标识（小写、连字符）。
- 示例：`zipformer-zh-en.tar.bz2`, `KeSeaCoParaformer.tar.bz2`, `ggml-small.bin`。

### 2.2 在 GitHub 上的组织方式

- **方式 A**：使用单独 Release（如 tag `models` 或 `models-v1`），附件为各模型压缩包。
- **方式 B**：在仓库中建目录（需配合 Git LFS 或外部存储），例如：
  - `models/whisper/small/ggml-small.bin`
  - `models/sherpa-onnx/zipformer-zh-en/`（目录则打包为 `.tar.bz2` 上传）

推荐：大文件用 **GitHub Release 附件** 或 **Git LFS + 仓库内路径**；`default_models.toml` 中的 `url` 指向上述地址。

### 2.3 模型下载 URL 示例

- Release 附件：  
  `https://github.com/Thinkre/KeVoiceInput/releases/download/models/zipformer-zh-en.tar.bz2`
- 或固定 base URL：  
  `https://github.com/Thinkre/KeVoiceInput/releases/latest/download/<filename>`

---

## 三、路径与 URL 汇总

| 类型     | 路径/URL 示例 |
|----------|-------------------------------|
| 应用 DMG | `releases/download/v0.0.1/KeVoiceInput-0.0.1-macos-aarch64.dmg` |
| 应用更新包 | `releases/download/v0.0.1/KeVoiceInput-0.0.1-macos-aarch64.app.tar.gz` |
| 更新签名 | `releases/download/v0.0.1/KeVoiceInput-0.0.1-macos-aarch64.app.tar.gz.sig` |
| 更新配置 | 可托管 `latest.json` 于 Release 或 CDN |
| 模型     | `releases/download/models/<model_id>.tar.bz2` 或仓库内路径 |

---

## 四、本地脚本与 CI 建议

- `scripts/release-artifacts.sh`：从构建输出收集文件，重命名为上述规范并输出到统一目录（如 `release-out/`）。
- `scripts/upload-release-to-github.sh`：使用 `gh release create` / `gh release upload` 上传到 `Thinkre/KeVoiceInput`。
- CI（如 GitHub Actions）：在 tag 推送时构建各平台产物，按规范命名并上传到对应 Release。

---

## 五、快速上传步骤

### 应用发布（上传到 GitHub Release）

1. 构建并收集产物（规范命名）：
   ```bash
   bun run tauri:build
   ./scripts/release-artifacts.sh
   ```
2. 上传到 [Thinkre/KeVoiceInput](https://github.com/Thinkre/KeVoiceInput)（需安装并登录 [gh](https://cli.github.com/)）：
   ```bash
   ./scripts/upload-release-to-github.sh
   ```
   或指定版本与目录：`./scripts/upload-release-to-github.sh v0.0.2 release-out`
3. 生成并提交 **latest.json**（应用内更新检查会从该文件拉取新版本 URL）：
   ```bash
   ./scripts/generate-latest-json.sh    # 默认用 tauri.conf 的 version；或传入版本如 0.0.2
   git add latest.json && git commit -m "chore: latest.json for vX.Y.Z" && git push
   ```
   `tauri.conf.json` 中 updater 的 endpoint 已指向：  
   `https://raw.githubusercontent.com/Thinkre/KeVoiceInput/main/latest.json`

### 模型上传

1. 将模型压缩包放入 `release-out/models/`，命名示例：`zipformer-zh-en.tar.bz2`、`KeSeaCoParaformer.tar.bz2`。
2. 上传到 Release `models`：
   ```bash
   ./scripts/upload-release-to-github.sh --models-only
   ```
   或指定目录：`./scripts/upload-release-to-github.sh --models-only /path/to/models`
