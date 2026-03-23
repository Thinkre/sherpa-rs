# Zipformer 双语模型崩溃问题修复指南

## 问题症状

### 症状 1：选择 zipformer 模型后应用闪退
- 在模型列表中点击 zipformer 双语模型
- 应用显示"kevoiceinput 意外退出"
- 崩溃类型：`SIGSEGV` (段错误)
- 崩溃地址：`0x18` (空指针解引用)

### 症状 2：本地模型列表不显示未下载的模型
- "本地模型"标签页只显示已下载的模型
- 无法看到可下载的模型列表
- 需要切换到"全部"标签才能看到未下载的模型

## 问题原因分析

### 问题 1：Zipformer 崩溃

根据崩溃日志分析，崩溃发生在加载 Transducer 模型时：

```
Exception Type: EXC_BAD_ACCESS (SIGSEGV)
Exception Codes: KERN_INVALID_ADDRESS at 0x0000000000000018
Faulting Thread: 7
```

可能的原因：

1. **模型文件不完整或损坏**
   - zipformer 模型需要 4 个必需文件：
     - `encoder-epoch-34-avg-19.onnx` (或 `.int8.onnx`)
     - `decoder-epoch-34-avg-19.onnx`
     - `joiner-epoch-34-avg-19.onnx` (或 `.int8.onnx`)
     - `tokens.txt`
   - 如果任何文件缺失或损坏，会导致崩溃

2. **sherpa-rs 库初始化问题**
   - TransducerRecognizer 初始化时访问空指针
   - 可能是 sherpa-onnx C++ 库的问题

3. **热词文件问题**
   - 如果配置了热词但 `bpe.vocab` 文件不存在
   - 可能导致初始化失败

4. **内存不足**
   - Transducer 模型较大，可能需要更多内存

### 问题 2：模型列表显示

前端代码在 `ModelsPage.tsx:743-745` 有错误的过滤逻辑：

```typescript
// 旧代码：只显示已下载的模型
if (!model.is_downloaded) {
  return false;
}
```

这导致用户无法在"本地模型"标签页看到可下载的模型。

## 修复方案

### 修复 1：模型列表显示问题 ✅ (已修复)

**修改文件**：`src/components/pages/ModelsPage.tsx`

**修改内容**：
```typescript
// 新代码：显示所有非 API 的模型（包括未下载的）
const filteredLocalModels = models.filter((model) => {
  // API 模型不在本地模型列表中显示
  if (model.engine_type === "Api") {
    return false;
  }

  // 显示所有本地引擎模型（已下载和未下载）
  // ... 搜索过滤逻辑 ...
  return true;
});
```

**效果**：
- "本地模型"标签页现在显示所有本地引擎模型
- 未下载的模型显示"下载"按钮
- 已下载的模型显示"删除"按钮和可选择状态

### 修复 2：Transducer 模型加载错误处理 ✅ (已修复)

**修改文件**：`src-tauri/src/managers/transcription.rs`

**修改内容**：

1. 添加模型文件存在性验证：
```rust
// 在创建 recognizer 前验证所有文件
if !encoder_file.exists() {
    return Err(anyhow::anyhow!("Encoder file does not exist: {:?}", encoder_file));
}
// ... 其他文件验证 ...
```

2. 增强错误消息：
```rust
let error_msg = format!(
    "Failed to load transducer model {}: {}. \
    This may indicate:\n\
    1. Model files are corrupted - try re-downloading\n\
    2. Incompatible model format - ensure it's a transducer/zipformer model\n\
    3. Insufficient memory - try closing other applications\n\
    Model files: encoder={:?}, decoder={:?}, joiner={:?}, tokens={:?}",
    model_id, e, encoder_file, decoder_file, joiner_file, tokens_file
);
```

**效果**：
- 模型加载失败时不会崩溃应用
- 提供详细的错误信息帮助诊断问题
- 发出事件通知前端显示错误

### 修复 3：模型文件验证（用户操作）

如果 zipformer 模型仍然崩溃，按以下步骤排查：

#### 步骤 1：检查模型文件完整性

```bash
# 找到模型目录
cd ~/Library/Application\ Support/com.kevoiceinput.app/models/

# 查找 zipformer 模型（ID 可能是 custom-xxx 或具体模型名）
ls -la */

# 检查模型目录内容（应该有 4 个必需文件）
ls -la <模型目录>/

# 应该看到：
# encoder-epoch-34-avg-19.onnx (或 .int8.onnx)
# decoder-epoch-34-avg-19.onnx
# joiner-epoch-34-avg-19.onnx (或 .int8.onnx)
# tokens.txt
# 可选：bbpe.model, bpe.vocab
```

#### 步骤 2：验证文件大小

```bash
cd <模型目录>

# 检查文件大小（不应该是 0 或异常小）
du -h *

# 典型大小：
# encoder: 100-300MB
# decoder: 1-10MB
# joiner: 1-10MB
# tokens.txt: 几百 KB
```

#### 步骤 3：重新下载模型

如果文件损坏或不完整：

1. 删除旧模型：
   ```bash
   rm -rf ~/Library/Application\ Support/com.kevoiceinput.app/models/<模型目录>
   ```

2. 在应用中重新下载模型

3. 或手动导入完整的模型文件夹

#### 步骤 4：查看崩溃日志

```bash
# 查看最新崩溃日志
ls -t ~/Library/Logs/DiagnosticReports/kevoiceinput* | head -1 | xargs cat

# 查找关键信息：
# - Exception Type
# - Faulting Thread
# - 栈回溯中的 sherpa 或 onnx 相关函数
```

#### 步骤 5：测试模型加载

创建测试脚本验证模型：

```bash
# 进入项目目录
cd /path/to/KeVoiceInput

# 运行 sherpa 测试二进制
cd src-tauri
cargo run --bin test_sherpa_api -- \
  --model-dir ~/Library/Application\ Support/com.kevoiceinput.app/models/<模型目录> \
  --model-type transducer
```

## 临时解决方案

如果 zipformer 模型持续崩溃，使用其他模型：

### 推荐替代模型

1. **Paraformer 模型**（中文）
   - 更稳定，内存占用更小
   - 速度快，准确率高
   - 不支持热词（使用 SeACo Paraformer 可支持）

2. **Whisper Small/Base**（多语言）
   - OpenAI Whisper 模型
   - 非常稳定，兼容性好
   - 支持多种语言

3. **SeACo Paraformer**（中文 + 热词）
   - 支持热词功能
   - 比 zipformer 更稳定

4. **FireRedAsr**（中文 + 方言）
   - 支持中文方言
   - 稳定性好

## 预防措施

### 开发者

1. **添加模型文件校验**：
   - 下载完成后验证文件完整性
   - 计算和验证文件 checksum

2. **改进错误处理**：
   - Catch sherpa-rs 的所有异常
   - 不要让 C++ 异常传播到 Rust

3. **添加模型测试**：
   - 在加载模型前进行快速测试
   - 验证模型可以正常初始化

4. **内存限制检查**：
   - 检测可用内存
   - 如果内存不足，提示用户

### 用户

1. **下载完整模型**：
   - 等待下载完全完成
   - 不要中途取消下载

2. **检查磁盘空间**：
   - 确保有足够空间存储模型
   - 模型大小通常在 100MB-1GB

3. **关闭其他应用**：
   - 加载大模型时释放内存
   - 避免同时运行多个大型应用

4. **使用稳定模型**：
   - 优先使用测试过的模型
   - 避免使用实验性或自定义模型

## 已知限制

1. **Transducer 模型内存占用大**
   - zipformer 模型可能需要 1-2GB 内存
   - 在低内存设备上可能不稳定

2. **sherpa-rs 绑定问题**
   - sherpa-rs 是 sherpa-onnx 的 Rust 绑定
   - C++ 异常可能导致 Rust panic
   - 建议使用最新版本的 sherpa-rs

3. **macOS 特定问题**
   - 动态库加载问题
   - 需要正确配置 @rpath

## 报告问题

如果问题仍未解决，请报告 Issue 并提供：

1. **系统信息**：
   ```bash
   sw_vers  # macOS 版本
   uname -m  # 架构
   ```

2. **模型信息**：
   - 模型名称和 ID
   - 模型来源（内置 / 下载 / 导入）
   - 模型目录内容：`ls -lh <模型目录>`

3. **崩溃日志**：
   ```bash
   ls -t ~/Library/Logs/DiagnosticReports/kevoiceinput* | head -1 | xargs cat
   ```

4. **应用日志**（如果有）：
   ```bash
   cat ~/Library/Application\ Support/com.kevoiceinput.app/*.log
   ```

5. **重现步骤**：
   - 详细说明操作步骤
   - 是否每次都崩溃

## 参考资料

- [Transducer 模型说明](https://k2-fsa.github.io/sherpa/onnx/pretrained_models/transducer/)
- [sherpa-onnx 文档](https://k2-fsa.github.io/sherpa/onnx/)
- [sherpa-rs GitHub](https://github.com/thewh1teagle/sherpa-rs)
- [CRASH_TROUBLESHOOTING.md](CRASH_TROUBLESHOOTING.md)

## 更新日志

- **2026-02-22**: 修复模型列表显示问题和添加 Transducer 错误处理
- **问题仍在调查中**: zipformer 模型空指针崩溃的根本原因
