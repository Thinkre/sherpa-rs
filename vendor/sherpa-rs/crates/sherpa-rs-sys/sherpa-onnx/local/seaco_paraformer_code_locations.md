# SeACo-Paraformer 实现代码位置

## 核心实现文件

### 1. 主模型接口和实现

#### 模型接口定义
- **文件**: `sherpa-onnx/csrc/offline-paraformer-model.h`
- **功能**: 
  - 定义 `OfflineParaformerModel` 类
  - 声明 `Forward()` 方法（支持 `bias_embed` 参数）
  - 声明 `ForwardEmbedding()` 方法（Embedding 模型推理）
  - 声明 `HasEmbeddingModel()` 方法

#### 模型实现
- **文件**: `sherpa-onnx/csrc/offline-paraformer-model.cc`
- **关键函数**:
  - `Impl::Init()`: 加载主模型和 Embedding 模型（`model_eb.onnx`）
  - `Impl::InitEmbedding()`: 初始化 Embedding 模型会话
  - `Impl::Forward()`: 主模型推理（两个重载版本，一个支持 `bias_embed`）
  - `Impl::ForwardEmbedding()`: Embedding 模型推理
  - `Impl::HasEmbeddingModel()`: 检查是否加载了 Embedding 模型

**关键代码位置**:
```cpp
// 加载 Embedding 模型
Line 44-48: 加载 model_eb.onnx
Line 60-64: 从 Manager 加载 model_eb.onnx

// 主模型推理（支持 bias_embed）
Line 67-146: Forward() 方法（检查是否需要 bias_embed）
Line 148-184: Forward() 方法（接收 bias_embed 参数）

// Embedding 模型推理
Line 198-208: ForwardEmbedding() 方法
```

### 2. 推理流程实现（核心）

#### 主要实现文件
- **文件**: `sherpa-onnx/csrc/offline-recognizer-paraformer-impl.h`
- **功能**: 完整的推理流程，包括热词处理

**关键类和函数**:

1. **`OfflineRecognizerParaformerImpl` 类**:
   - 继承自 `OfflineRecognizerImpl`
   - 实现完整的 SeACo-Paraformer 推理流程

2. **热词初始化**:
   - `InitHotwords()` (Line 276-294): 从文件加载热词
   - `InitHotwordsFromManager()` (Line 296-317): 从 Manager 加载热词
   - `EncodeHotwordsForSeaco()` (Line 319-355): 将热词编码为 token IDs

3. **Bias Embedding 生成**:
   - `GenerateBiasEmbed()` (Line 357-415): 生成 bias_embed 张量
     - 填充热词到固定长度
     - 调用 Embedding 模型
     - 提取 embeddings
   - `CreateEmptyBiasEmbed()` (Line 417-429): 创建空的 bias_embed
   - `TransposeAndIndex()` (Line 431-480): 从 embedding 输出中提取和索引

4. **推理流程**:
   - `DecodeStreams()` (Line 136-219): 主推理流程
     - Line 191-198: 检查是否有热词和 Embedding 模型
     - Line 193-195: 生成 bias_embed 并调用主模型

**关键代码位置**:
```cpp
// 热词处理
Line 275-294: InitHotwords() - 初始化热词
Line 319-355: EncodeHotwordsForSeaco() - 热词编码
Line 357-415: GenerateBiasEmbed() - 生成 bias_embed

// Embedding 提取
Line 431-480: TransposeAndIndex() - 提取和索引 embeddings

// 主推理流程
Line 191-198: 检查并使用热词
```

### 3. 解码器实现

- **文件**: `sherpa-onnx/csrc/offline-paraformer-greedy-search-decoder.cc`
- **功能**: 贪心搜索解码器
- **关键函数**: `Decode()` - 从 log_probs 解码出 token 序列

### 4. 配置相关

#### 模型配置
- **文件**: `sherpa-onnx/csrc/offline-paraformer-model-config.h`
- **文件**: `sherpa-onnx/csrc/offline-paraformer-model-config.cc`
- **功能**: 
  - 定义 `model_eb` 配置项（Embedding 模型路径）
  - 验证配置

#### 识别器配置
- **文件**: `sherpa-onnx/csrc/offline-recognizer.cc`
- **关键代码** (Line 84-93):
  ```cpp
  // 检查是否为 SeACo-Paraformer
  bool is_seaco_paraformer = !model_config.paraformer.model.empty() &&
                              !model_config.paraformer.model_eb.empty();
  ```

### 5. Python 绑定

- **文件**: `sherpa-onnx/python/sherpa_onnx/offline_recognizer.py`
- **功能**: Python API 接口
- **关键代码** (Line 443-449): 文档说明 `model_eb` 参数

- **文件**: `sherpa-onnx/python/csrc/offline-paraformer-model-config.cc`
- **功能**: Python 绑定实现
- **关键代码** (Line 20): 绑定 `model_eb` 属性

## 代码结构图

```
sherpa-onnx/
├── csrc/
│   ├── offline-paraformer-model.h          # 模型接口定义
│   ├── offline-paraformer-model.cc          # 模型实现（加载、推理）
│   ├── offline-paraformer-model-config.h    # 配置定义
│   ├── offline-paraformer-model-config.cc   # 配置实现
│   ├── offline-recognizer-paraformer-impl.h # ⭐ 核心推理流程实现
│   ├── offline-paraformer-greedy-search-decoder.cc  # 解码器
│   └── offline-recognizer.cc                # 识别器配置检查
│
└── python/
    ├── sherpa_onnx/
    │   └── offline_recognizer.py           # Python API
    └── csrc/
        └── offline-paraformer-model-config.cc  # Python 绑定
```

## 关键实现流程

### 1. 初始化流程

```
OfflineRecognizerParaformerImpl::InitHotwords()
  ↓
EncodeHotwordsForSeaco()
  ↓
hotwords_ids_ (存储编码后的热词)
```

### 2. 推理流程

```
DecodeStreams()
  ↓
GenerateBiasEmbed()
  ├─ 填充热词到 (N+1, 10)
  ├─ 调用 model_->ForwardEmbedding()
  └─ TransposeAndIndex() 提取 embeddings
  ↓
model_->Forward(features, lengths, bias_embed)
  ├─ 调用 model_eb.onnx (Embedding 模型)
  └─ 调用 model.onnx (主模型，带 bias_embed)
  ↓
decoder_->Decode(log_probs, token_num)
```

## 关键数据结构

### 1. 热词存储
```cpp
// 位置: offline-recognizer-paraformer-impl.h:488
std::vector<std::vector<int32_t>> hotwords_ids_;
// 每个热词是一个 token ID 序列
```

### 2. Bias Embedding 形状
```cpp
// 输入到主模型
shape: (batch_size, num_hotwords, 512)
dtype: float32
```

### 3. Embedding 模型输出
```cpp
// Embedding 模型输出
shape: (10, num_hotwords + 1, 512)
dtype: float32
// 格式: (T, N, D) - time-first
```

## 关键函数调用链

### 热词处理
```
InitHotwords()
  → EncodeHotwordsForSeaco()
    → SplitUtf8() [text-utils.h]
    → symbol_table_[ch] [symbol-table.h]
```

### Bias Embedding 生成
```
GenerateBiasEmbed()
  → model_->ForwardEmbedding()
    → embedding_sess_->Run() [ONNX Runtime]
  → TransposeAndIndex()
    → 内存操作提取 embeddings
```

### 主模型推理
```
DecodeStreams()
  → ApplyLFR() [LFR 变换]
  → ApplyCMVN() [CMVN 归一化]
  → PadSequence() [序列填充]
  → GenerateBiasEmbed() [生成 bias_embed]
  → model_->Forward(features, lengths, bias_embed)
    → sess_->Run() [ONNX Runtime]
  → decoder_->Decode()
    → 贪心搜索解码
```

## 相关工具函数

### 文本处理
- **文件**: `sherpa-onnx/csrc/text-utils.h/cc`
- **函数**: `SplitUtf8()` - UTF-8 字符分割

### 词表管理
- **文件**: `sherpa-onnx/csrc/symbol-table.h/cc`
- **功能**: Token ID 和字符之间的映射

### 序列填充
- **文件**: `sherpa-onnx/csrc/pad-sequence.h`
- **功能**: 将变长序列填充到相同长度

## 测试文件

- **文件**: `test_seaco_hotwords.sh`
- **功能**: SeACo-Paraformer 热词功能测试脚本

## 文档文件

- `scripts/seaco_paraformer_inference_flow.md` - 推理流程文档
- `scripts/seaco_paraformer_technical_overview.md` - 技术概览
- `scripts/seaco_paraformer_implementation_analysis.md` - 实现分析
- `scripts/seaco_paraformer_comparison.md` - 与 FunASR 对比

## 快速定位

### 如果想了解：
1. **如何加载 Embedding 模型**: `offline-paraformer-model.cc:44-48`
2. **如何生成 bias_embed**: `offline-recognizer-paraformer-impl.h:357-415`
3. **如何提取 embeddings**: `offline-recognizer-paraformer-impl.h:431-480`
4. **如何调用主模型**: `offline-recognizer-paraformer-impl.h:191-198`
5. **如何编码热词**: `offline-recognizer-paraformer-impl.h:319-355`

## 总结

**最核心的实现文件**:
1. ⭐ **`sherpa-onnx/csrc/offline-recognizer-paraformer-impl.h`** - 完整的推理流程，包括热词处理
2. **`sherpa-onnx/csrc/offline-paraformer-model.cc`** - 模型加载和推理接口
3. **`sherpa-onnx/csrc/offline-paraformer-model.h`** - 模型接口定义

这三个文件包含了 SeACo-Paraformer 的完整实现。
