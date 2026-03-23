# Zipformer 模型崩溃快速调试指南

## 问题：之前可以用，现在不能用了

### 立即检查清单

#### 1. 确认当前的应用版本和DMG
```bash
# 检查当前安装的应用
ls -la /Applications/KeVoiceInput.app/

# 检查DMG构建时间
ls -lh src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg
```

**关键问题**：使用的是哪个版本的应用？
- 之前能用的版本
- 最新构建的版本（2026-02-22）

#### 2. 检查动态库是否正确
```bash
# 检查已安装应用的动态库
ls -la /Applications/KeVoiceInput.app/Contents/Frameworks/

# 应该看到 4 个 .dylib 文件
# 如果缺失，这就是问题所在！
```

#### 3. 检查模型文件
```bash
# 查看所有模型
ls -la ~/Library/Application\ Support/com.kevoiceinput.app/models/

# 找到 zipformer 模型目录（可能是 custom-xxx 开头）
# 检查模型文件完整性
ls -lh ~/Library/Application\ Support/com.kevoiceinput.app/models/<zipformer_dir>/
```

### 可能的原因

#### 原因 1：使用了没有正确打包动态库的 DMG ❗

**症状**：
- 应用启动后立即崩溃
- `/Applications/KeVoiceInput.app/Contents/Frameworks/` 目录不存在或为空

**解决**：
```bash
# 删除当前应用
rm -rf /Applications/KeVoiceInput.app

# 使用最新的 DMG 重新安装（包含正确的动态库）
hdiutil attach src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg
/Volumes/KeVoiceInput/Install.command
```

#### 原因 2：前端代码变更导致的问题

**检查点**：
- ModelsPage.tsx 的修改是否正确
- 模型选择逻辑是否有变化

**验证**：
```bash
# 重新构建前端
cd /Users/thinkre/Desktop/projects/KeVoiceInput
bun run build

# 重新构建应用
bun run tauri:build
```

#### 原因 3：后端 Rust 代码的问题

**检查点**：
- transcription.rs 的修改
- 是否影响了 Transducer 初始化

**验证**：
```bash
cd src-tauri
cargo build --release 2>&1 | grep -E "(error|warning.*zipformer|warning.*transducer)"
```

### 快速测试步骤

#### 步骤 1：测试模型列表显示

1. 打开应用
2. 进入"模型"页面
3. 切换到"本地模型"标签

**预期结果**：
- 应该显示所有本地引擎模型（包括未下载的）
- zipformer 模型应该在列表中

**如果看不到 zipformer**：
- 说明模型列表过滤有问题
- 检查 ModelsPage.tsx 的修改

#### 步骤 2：测试模型加载

1. 点击一个简单的模型（如 Paraformer）
2. 观察是否能正常加载

**如果 Paraformer 也崩溃**：
- 说明不是 zipformer 特定问题
- 可能是动态库问题

**如果 Paraformer 正常**：
- 说明是 zipformer 特定问题
- 继续调试 Transducer 加载逻辑

#### 步骤 3：查看错误日志

```bash
# 实时查看日志（如果应用有日志文件）
tail -f ~/Library/Application\ Support/com.kevoiceinput.app/*.log

# 或者使用 Console.app 查看实时日志
open -a Console

# 查找 kevoiceinput 相关日志
```

#### 步骤 4：查看崩溃报告

```bash
# 最新的崩溃日志
ls -t ~/Library/Logs/DiagnosticReports/kevoiceinput* | head -1 | xargs cat

# 关键信息：
# - Exception Type（异常类型）
# - Faulting Thread（崩溃线程）
# - 包含 "sherpa" 或 "onnx" 的栈帧
```

### 对比测试

如果之前能用的版本还在，进行对比：

```bash
# 备份当前版本
cp -R /Applications/KeVoiceInput.app /Applications/KeVoiceInput.app.new

# 恢复之前的版本（如果有备份）
cp -R /path/to/old/KeVoiceInput.app /Applications/

# 测试旧版本
# 1. 打开应用
# 2. 加载 zipformer 模型
# 3. 观察是否正常

# 对比差异
diff -r /Applications/KeVoiceInput.app /Applications/KeVoiceInput.app.new
```

### 回滚方案

如果需要回滚到之前能用的状态：

#### 回滚前端代码

```bash
# 检查 ModelsPage.tsx 的修改
cat src/components/pages/ModelsPage.tsx | grep -A 20 "filteredLocalModels"

# 如果需要回滚，手动恢复之前的过滤逻辑
```

#### 回滚后端代码

```bash
# 检查 transcription.rs 的修改
cat src-tauri/src/managers/transcription.rs | grep -A 30 "Transducer] Creating recognizer"

# 应该看到原始的错误处理逻辑（.map_err）
```

### 收集调试信息

如果问题仍未解决，收集以下信息：

```bash
# 1. 系统信息
sw_vers > /tmp/debug_info.txt
uname -m >> /tmp/debug_info.txt

# 2. 应用结构
ls -laR /Applications/KeVoiceInput.app/Contents/ >> /tmp/debug_info.txt

# 3. 动态库依赖
otool -L /Applications/KeVoiceInput.app/Contents/MacOS/kevoiceinput >> /tmp/debug_info.txt
otool -L /Applications/KeVoiceInput.app/Contents/Frameworks/*.dylib >> /tmp/debug_info.txt

# 4. 模型目录
ls -laR ~/Library/Application\ Support/com.kevoiceinput.app/models/ >> /tmp/debug_info.txt

# 5. 最新崩溃日志
ls -t ~/Library/Logs/DiagnosticReports/kevoiceinput* | head -1 | xargs cat >> /tmp/debug_info.txt

echo "调试信息已保存到 /tmp/debug_info.txt"
```

### 最可能的问题

根据"之前可以用，现在不能用"的描述，最可能的原因是：

1. **动态库打包问题** (90% 可能)
   - 新构建的 DMG 没有正确打包动态库
   - 或动态库路径配置错误

2. **前端代码变更** (5% 可能)
   - ModelsPage.tsx 的过滤逻辑可能影响了模型选择
   - 但这不应该导致崩溃，只是显示问题

3. **后端代码变更** (5% 可能)
   - transcription.rs 的修改可能引入了问题
   - 但我们刚才已经回滚了

### 快速修复方案

最快的解决方案：

```bash
# 1. 确保使用正确构建的 DMG
cd /Users/thinkre/Desktop/projects/KeVoiceInput

# 2. 重新打包动态库
./scripts/copy-dylibs.sh src-tauri/target/release/bundle/macos/KeVoiceInput.app

# 3. 重新创建 DMG
./scripts/create-dmg.sh \
  src-tauri/target/release/bundle/macos/KeVoiceInput.app \
  src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg

# 4. 重新安装
rm -rf /Applications/KeVoiceInput.app
hdiutil attach src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg
/Volumes/KeVoiceInput/Install.command

# 5. 测试
open /Applications/KeVoiceInput.app
```

### 需要回答的关键问题

1. **什么时候开始不能用的？**
   - 安装了新的 DMG 之后？
   - 系统更新之后？
   - 模型更新之后？

2. **具体的错误是什么？**
   - 应用直接崩溃？
   - 点击模型后崩溃？
   - 有错误提示吗？

3. **其他模型能用吗？**
   - Paraformer 能加载吗？
   - Whisper 能加载吗？
   - 只有 zipformer 不能用？

4. **动态库完整吗？**
   - `/Applications/KeVoiceInput.app/Contents/Frameworks/` 里有 4 个 .dylib 文件吗？

请提供这些信息，我可以更精确地定位问题！
