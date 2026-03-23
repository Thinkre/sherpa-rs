# KeVoiceInput v0.0.1 Release Notes

## 安装说明 📦

### macOS 安装

下载 `KeVoiceInput_0.0.1_aarch64.dmg` (约 32MB)

**推荐安装方法**：

1. **打开 DMG**，双击查看 `README.txt` 获取完整说明

2. **快速安装**（如果遇到 macOS 安全提示）：
   ```bash
   # 打开终端，运行：
   /Volumes/KeVoiceInput/Install.command
   ```

3. **手动安装**（最可靠）：
   - 拖拽 `KeVoiceInput.app` 到 `Applications` 文件夹
   - 打开终端，运行：
     ```bash
     xattr -cr /Applications/KeVoiceInput.app
     ```
   - 右键点击应用 → 选择"打开"

**故障排查**：如果遇到问题，请参考：
- DMG 中的 `README.txt`
- [安装故障排查指南](../docs/DMG_INSTALL_TROUBLESHOOTING.md)
- [Gatekeeper 解决方案](../docs/GATEKEEPER_WORKAROUND.md)

### 系统要求

- macOS 10.13 或更高版本
- Apple Silicon (M1/M2/M3) 或 Intel
- 4GB RAM（推荐 8GB）
- 2GB 可用磁盘空间

### 首次启动

1. **授予权限**：
   - 辅助功能权限（用于输入转录结果）
   - 麦克风权限（用于录音）

2. **下载模型**：
   - 在"模型"页面选择一个推荐模型
   - 点击"下载"等待完成

3. **开始使用**：
   - 设置快捷键（或使用默认快捷键）
   - 按快捷键开始语音输入

---

## 新功能 ✨

### 核心功能
- ✅ 多引擎语音识别（Whisper、Sherpa-ONNX、Paraformer、SeACo Paraformer）
- ✅ 实时语音转录
- ✅ 模型管理（下载、删除、切换）
- ✅ 历史记录（自动保存、查看、搜索）
- ✅ 快捷键支持
- ✅ LLM 后处理（可选）
- ✅ 多语言界面（14 种语言）

### 高级功能
- ✅ 热词支持（SeACo Paraformer）
- ✅ 标点符号自动添加
- ✅ 自定义词汇纠正
- ✅ ITN（逆文本规范化）
- ✅ 智能文本过滤（移除填充词、处理口吃）
- ✅ 剪贴板集成
- ✅ 自动启动
- ✅ 托盘图标

---

## 已知问题 ⚠️

### macOS 安全提示

**问题**：双击 `Install.command` 时显示安全警告

**原因**：应用未进行 Apple 代码签名和公证

**解决**：
1. 使用终端运行安装脚本（见上方安装说明）
2. 或使用手动安装方法
3. 详见 DMG 中的 `README.txt`

### 首次启动可能较慢

**问题**：首次加载模型时需要较长时间

**原因**：模型文件较大，需要初始化

**解决**：耐心等待，后续启动会更快

---

## 校验和 🔐

```
SHA256: [在此处粘贴 DMG 的 SHA256 校验和]
```

验证下载文件：
```bash
shasum -a 256 KeVoiceInput_0.0.1_aarch64.dmg
```

---

## 技术栈 🛠

- **前端**: React 18 + TypeScript + Tailwind CSS
- **后端**: Rust + Tauri 2.9.1
- **语音识别**: Whisper, Sherpa-ONNX, Paraformer
- **音频处理**: cpal, rodio, rubato

---

## 反馈和支持 💬

- **Bug 报告**: [GitHub Issues](https://github.com/thinkre/KeVoiceInput/issues)
- **功能请求**: [GitHub Discussions](https://github.com/thinkre/KeVoiceInput/discussions)
- **文档**: [项目 README](../README.md)

---

## 更新日志 📝

### v0.0.1 (2026-02-22)

#### 添加
- 初始发布
- 多引擎语音识别支持
- 模型管理系统
- 历史记录功能
- 14 种语言界面
- LLM 后处理集成

#### 修复
- DMG 安装脚本文件名改为英文（`Install.command`）以避免终端路径问题
- 添加 README.txt 提供详细安装说明
- 添加 manual-install.sh 作为备用安装方法
- 优化 Finder 视图布局

#### 已知限制
- 应用未进行 Apple 代码签名（需要 Apple Developer Program）
- 首次加载模型较慢
- 部分功能仅在 macOS 上可用

---

## 开发者信息 👨‍💻

- **作者**: thinkre
- **许可证**: MIT
- **源码**: https://github.com/thinkre/KeVoiceInput
- **致谢**: Tauri, Sherpa-ONNX, Whisper, FunASR

---

## 下一步计划 🚀

- [ ] 代码签名和公证
- [ ] Windows 支持
- [ ] Linux 支持
- [ ] 更多语音识别引擎
- [ ] 云端同步
- [ ] 插件系统

---

**感谢使用 KeVoiceInput！**

如果觉得有用，请在 GitHub 上给我们一个 ⭐️
