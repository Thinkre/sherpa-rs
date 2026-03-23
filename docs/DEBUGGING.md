# KeVoiceInput 调试指南

本文档提供常见问题的调试方法和故障排查步骤。

## 快速诊断清单

### 应用无法启动

1. **检查系统要求**
   ```bash
   # macOS 版本
   sw_vers

   # 处理器架构
   uname -m
   ```

2. **检查应用完整性**
   ```bash
   # 验证 app 结构
   ls -la /Applications/KeVoiceInput.app/Contents/

   # 检查动态库
   ls -la /Applications/KeVoiceInput.app/Contents/Frameworks/
   ```

3. **查看崩溃日志**
   ```bash
   # macOS
   ls -t ~/Library/Logs/DiagnosticReports/kevoiceinput* | head -1 | xargs cat

   # 或使用 Console.app
   open -a Console
   ```

### 模型加载失败

1. **验证模型文件**
   ```bash
   # 查看所有模型
   ls -la ~/Library/Application\ Support/com.kevoiceinput.app/models/

   # 检查特定模型目录
   ls -lh ~/Library/Application\ Support/com.kevoiceinput.app/models/<model_dir>/
   ```

2. **检查文件完整性**
   - Transducer: `encoder`, `decoder`, `joiner`, `tokens.txt`
   - Paraformer: `model.onnx`, `tokens.txt`
   - SeACo Paraformer: `model.onnx`, `model_eb.onnx`, `tokens.txt`

3. **查看错误信息**
   - 打开"设置" → "通用" → 启用"调试模式"
   - 尝试加载模型，查看错误提示

### 录音无声音或质量差

1. **检查音频设备**
   - 打开"设置" → "音频" → 查看输入设备列表
   - 确认麦克风已选择且未静音
   - 测试系统麦克风是否工作正常

2. **检查 VAD 设置**
   - VAD 阈值过高可能导致无法检测到语音
   - 尝试降低阈值或关闭 VAD

3. **查看录音文件**
   ```bash
   # 保存的录音位置
   ls -lt ~/Library/Application\ Support/com.kevoiceinput.app/recordings/ | head

   # 播放检查
   afplay ~/Library/Application\ Support/com.kevoiceinput.app/recordings/latest.wav
   ```

## 模型特定问题

### Transducer (Zipformer) 崩溃

**症状**: 点击 Zipformer 模型后应用崩溃

**可能原因**:
1. 动态库缺失或版本不匹配
2. 模型文件损坏
3. 内存不足

**调试步骤**:

1. 确认动态库完整
   ```bash
   ls -la /Applications/KeVoiceInput.app/Contents/Frameworks/
   # 应该看到:
   # - libcargs.dylib
   # - libonnxruntime.1.17.1.dylib
   # - libsherpa-onnx-c-api.dylib
   # - libsherpa-onnx-cxx-api.dylib
   ```

2. 检查库依赖
   ```bash
   otool -L /Applications/KeVoiceInput.app/Contents/MacOS/kevoiceinput
   otool -L /Applications/KeVoiceInput.app/Contents/Frameworks/*.dylib
   ```

3. 测试其他模型
   - 如果 Paraformer 也崩溃 → 动态库问题
   - 如果仅 Zipformer 崩溃 → 模型或 Transducer 特定问题

4. 查看崩溃日志中的关键信息
   ```bash
   ls -t ~/Library/Logs/DiagnosticReports/kevoiceinput* | head -1 | \
     xargs cat | grep -A 10 "Exception\|Faulting\|sherpa\|onnx"
   ```

### SeACo Paraformer 热词不工作

**症状**: 热词未提高识别准确度

**检查项**:

1. 确认使用 SeACo 模型
   ```bash
   # 检查是否有 model_eb.onnx
   ls -la ~/Library/Application\ Support/com.kevoiceinput.app/models/<seaco_model>/model_eb.onnx
   ```

2. 检查热词配置
   - 打开"设置" → "热词"
   - 确认热词已添加且启用
   - 检查热词格式（每行一个）

3. 查看调试日志
   - 启用调试模式
   - 加载模型时应显示 "Loaded model_eb.onnx"
   - 转录时应显示 "Applied hotwords: X"

### Whisper 模型速度慢

**症状**: 转录耗时很长

**优化建议**:

1. **使用更小的模型**
   - Large → Turbo → Medium → Small
   - 准确度和速度的权衡

2. **硬件加速**
   - macOS: 确保使用 Metal 加速
   - 检查 GPU 是否被识别

3. **减少音频长度**
   - 使用 VAD 自动分段
   - 避免一次处理超长录音

## 构建问题

### DMG 构建后应用闪退

**原因**: 动态库未正确打包

**解决方法**:

```bash
# 1. 删除旧应用
rm -rf /Applications/KeVoiceInput.app

# 2. 重新打包动态库
./scripts/copy-dylibs.sh src-tauri/target/release/bundle/macos/KeVoiceInput.app

# 3. 重新创建 DMG
./scripts/create-dmg.sh \
  src-tauri/target/release/bundle/macos/KeVoiceInput.app \
  src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg

# 4. 重新安装
hdiutil attach src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg
/Volumes/KeVoiceInput/Install.command
```

### 构建时找不到 sherpa-onnx

**错误信息**: `Cannot find libsherpa-onnx-c-api`

**解决方法**:

```bash
# 1. 设置环境变量
export SHERPA_LIB_PATH=/path/to/sherpa-onnx/install/lib

# 2. 验证库文件存在
ls -la $SHERPA_LIB_PATH/

# 3. 重新构建
cd src-tauri && cargo clean && cd ..
bun run tauri:build
```

### 前端编译错误

**常见问题**:

1. **依赖版本冲突**
   ```bash
   # 清理并重新安装
   rm -rf node_modules
   bun install
   ```

2. **TypeScript 类型错误**
   ```bash
   # 重新生成 bindings
   cd src-tauri
   cargo build
   cd ..
   # bindings.ts 应自动更新
   ```

3. **Vite 端口占用**
   ```bash
   # 使用清理脚本
   bun run dev:clean
   ```

## 日志和诊断

### 启用调试模式

1. 打开应用
2. "设置" → "通用" → 启用"调试模式"
3. 查看控制台输出（开发模式）或日志文件

### 收集诊断信息

```bash
#!/bin/bash
# save-debug-info.sh

OUTPUT_FILE="/tmp/kevoiceinput-debug-$(date +%Y%m%d-%H%M%S).txt"

echo "=== System Info ===" >> "$OUTPUT_FILE"
sw_vers >> "$OUTPUT_FILE"
uname -m >> "$OUTPUT_FILE"

echo -e "\n=== App Structure ===" >> "$OUTPUT_FILE"
ls -laR /Applications/KeVoiceInput.app/Contents/ >> "$OUTPUT_FILE"

echo -e "\n=== Dynamic Libraries ===" >> "$OUTPUT_FILE"
otool -L /Applications/KeVoiceInput.app/Contents/MacOS/kevoiceinput >> "$OUTPUT_FILE"
otool -L /Applications/KeVoiceInput.app/Contents/Frameworks/*.dylib >> "$OUTPUT_FILE"

echo -e "\n=== Models ===" >> "$OUTPUT_FILE"
ls -laR ~/Library/Application\ Support/com.kevoiceinput.app/models/ >> "$OUTPUT_FILE"

echo -e "\n=== Recent Crash Logs ===" >> "$OUTPUT_FILE"
ls -t ~/Library/Logs/DiagnosticReports/kevoiceinput* 2>/dev/null | head -3 | \
  while read file; do
    echo "--- $file ---" >> "$OUTPUT_FILE"
    cat "$file" >> "$OUTPUT_FILE"
  done

echo "Debug info saved to: $OUTPUT_FILE"
```

### 查看实时日志

**开发模式**:
```bash
bun run tauri:dev
# 控制台会显示所有日志
```

**生产模式**:
```bash
# macOS Console.app
open -a Console
# 搜索 "kevoiceinput"

# 或命令行
log stream --predicate 'process == "kevoiceinput"' --level debug
```

## 性能分析

### CPU 使用率过高

1. **检查模型大小**
   - 使用更小的模型
   - 减少线程数

2. **VAD 优化**
   - 调整 VAD 检测频率
   - 增加静音阈值

3. **后台处理**
   - 关闭不需要的 LLM 后处理
   - 禁用自动标点符号

### 内存占用过高

1. **模型优化**
   - 不要同时加载多个大模型
   - 切换模型后释放旧模型

2. **历史记录清理**
   - 定期删除旧的转录记录
   - 限制保存的录音文件数量

3. **检查内存泄漏**
   ```bash
   # 使用 Activity Monitor 或 Instruments
   open -a "Activity Monitor"
   ```

## 常见错误代码

| 错误代码 | 含义 | 解决方法 |
|---------|------|---------|
| `MODEL_NOT_FOUND` | 模型文件不存在 | 重新下载模型 |
| `MODEL_LOAD_FAILED` | 模型加载失败 | 检查文件完整性 |
| `AUDIO_DEVICE_ERROR` | 音频设备错误 | 检查麦克风权限和连接 |
| `TRANSCRIPTION_FAILED` | 转录失败 | 查看详细错误信息 |
| `PERMISSION_DENIED` | 权限被拒绝 | 授予辅助功能/麦克风权限 |

## 获取帮助

如果问题仍未解决：

1. **搜索 Issues**: [GitHub Issues](https://github.com/yourusername/KeVoiceInput/issues)
2. **创建 Issue**: 提供诊断信息和错误日志
3. **讨论区**: [GitHub Discussions](https://github.com/yourusername/KeVoiceInput/discussions)

**提交 Issue 时请包含**:
- 操作系统和版本
- 应用版本
- 复现步骤
- 错误信息和日志
- 使用的模型
- 诊断信息（运行 `save-debug-info.sh`）
