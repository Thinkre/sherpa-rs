# DMG 测试清单

在将 DMG 发布给其他用户前，使用此清单进行测试。

## 构建后立即验证

```bash
# 1. 挂载 DMG
hdiutil attach src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg

# 2. 检查文件列表
ls -la /Volumes/KeVoiceInput/

# 应该看到：
# - KeVoiceInput.app
# - Applications (symlink)
# - Install.command
# - manual-install.sh
# - README.txt

# 3. 验证 README 内容
cat /Volumes/KeVoiceInput/README.txt

# 4. 检查动态库
ls -la /Volumes/KeVoiceInput/KeVoiceInput.app/Contents/Frameworks/

# 应该看到 4 个 .dylib 文件：
# - libcargs.dylib
# - libonnxruntime.1.17.1.dylib
# - libsherpa-onnx-c-api.dylib
# - libsherpa-onnx-cxx-api.dylib

# 5. 卸载
hdiutil detach /Volumes/KeVoiceInput
```

## 本机测试

### 测试 1：Install.command 双击

- [ ] 双击 `Install.command`
- [ ] 可能被 Gatekeeper 阻止（正常）
- [ ] 右键 → 打开（可能有效）

### 测试 2：Terminal 命令

```bash
# 挂载 DMG
hdiutil attach src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg

# 运行安装脚本
/Volumes/KeVoiceInput/Install.command

# 应该能够正常执行
```

### 测试 3：手动安装

- [ ] 拖拽 app 到 Applications
- [ ] 运行 `xattr -cr /Applications/KeVoiceInput.app`
- [ ] 右键打开应用
- [ ] 应用能正常启动

### 测试 4：README 可读性

- [ ] 双击 README.txt
- [ ] 在 TextEdit 中正常显示
- [ ] 格式清晰易读
- [ ] 所有说明都正确

## 其他 Mac 设备测试

### 环境准备

在**干净的 Mac**上测试（没有 Xcode、没有开发工具）：
- 使用不同的 macOS 版本（Catalina, Big Sur, Monterey, Ventura, Sonoma）
- 使用不同的 Mac 型号（Intel vs Apple Silicon）

### 测试场景

#### 场景 1：从浏览器下载

- [ ] 从 GitHub Release 下载 DMG
- [ ] 文件会被标记为"来自互联网"
- [ ] 双击打开 DMG
- [ ] 尝试所有安装方法

#### 场景 2：从 AirDrop 接收

- [ ] 通过 AirDrop 发送 DMG
- [ ] 接收后打开
- [ ] 测试安装

#### 场景 3：从 USB 复制

- [ ] 复制 DMG 到 USB
- [ ] 在另一台 Mac 上从 USB 打开
- [ ] 测试安装

### 详细测试步骤

#### 测试方法 1：README 指引

1. [ ] 打开 DMG
2. [ ] 双击 README.txt
3. [ ] 按照 METHOD 1 操作（Install.command）
4. [ ] 如果被阻止，按照 TROUBLESHOOTING 解决
5. [ ] 验证应用能启动

#### 测试方法 2：手动安装

1. [ ] 打开 DMG
2. [ ] 拖拽 app 到 Applications
3. [ ] 打开终端
4. [ ] 运行 `xattr -cr /Applications/KeVoiceInput.app`
5. [ ] 右键打开应用
6. [ ] 验证应用能启动

#### 测试方法 3：Terminal 安装

1. [ ] 打开 DMG
2. [ ] 打开终端
3. [ ] 运行 `/Volumes/KeVoiceInput/Install.command`
4. [ ] 按提示完成安装
5. [ ] 验证应用能启动

#### 测试方法 4：manual-install.sh

1. [ ] 打开 DMG
2. [ ] 打开终端
3. [ ] 运行 `/Volumes/KeVoiceInput/manual-install.sh`
4. [ ] 验证安装完成
5. [ ] 验证应用能启动

## 应用功能测试

安装完成后：

### 首次启动

- [ ] 应用能够启动（不闪退）
- [ ] 提示授予辅助功能权限
- [ ] 提示授予麦克风权限

### 基本功能

- [ ] 能够打开设置
- [ ] 能够查看模型列表
- [ ] 能够下载模型
- [ ] 能够选择音频设备
- [ ] 能够启动录音
- [ ] 能够进行转录

### 动态库验证

```bash
# 检查应用能找到动态库
otool -L /Applications/KeVoiceInput.app/Contents/MacOS/KeVoiceInput | grep -E "(sherpa|onnx)"

# 应该看到 @rpath/ 路径，不应该有 /Users/... 绝对路径
```

## 问题记录

如果测试中发现问题，记录以下信息：

### 系统信息
```bash
sw_vers
# macOS 版本

uname -m
# 架构 (x86_64 或 arm64)

echo $SHELL
# Shell 类型
```

### 错误信息
- 完整错误消息
- 终端输出
- 系统日志（Console.app）

### 重现步骤
1. 操作步骤
2. 预期结果
3. 实际结果

## 成功标准

所有测试项通过才能发布：

- [x] DMG 包含所有必要文件
- [x] 动态库正确打包
- [x] README.txt 内容正确
- [ ] 至少一种安装方法在干净的 Mac 上能成功
- [ ] 应用安装后能正常启动
- [ ] 应用基本功能正常工作
- [ ] 没有硬编码的绝对路径

## 发布前最终检查

```bash
# 1. 验证 DMG 大小合理（约 30-35MB）
du -h src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg

# 2. 计算 SHA256 校验和
shasum -a 256 src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg

# 3. 测试 DMG 可以正常挂载和卸载
hdiutil attach src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg
hdiutil detach /Volumes/KeVoiceInput

# 4. 在 Release 说明中包含：
#    - 安装方法（指向 README.txt）
#    - 故障排查（指向文档）
#    - 系统要求
#    - SHA256 校验和
```

## 用户反馈收集

发布后追踪：
- GitHub Issues 中的安装问题
- 用户成功/失败的反馈
- 常见问题模式
- 改进建议
