# SeACo-Paraformer API 支持状态

## 当前状态总结

✅ **`model_eb` 已经在所有 API 中支持**
✅ **代码已经同时支持标准 Paraformer 和 SeACo-Paraformer 两种模式**

## API 支持详情

### 1. C++ API

**配置结构**: `sherpa-onnx/csrc/offline-paraformer-model-config.h`

```cpp
struct OfflineParaformerModelConfig {
  std::string model;      // model.onnx 路径
  std::string model_eb;   // model_eb.onnx 路径（可选，SeACo-Paraformer）
  // ...
};
```

**使用方式**:
```cpp
OfflineRecognizerConfig config;
config.model_config.paraformer.model = "model.onnx";
config.model_config.paraformer.model_eb = "model_eb.onnx";  // 可选
```

### 2. Python API

**接口**: `sherpa-onnx/python/sherpa_onnx/offline_recognizer.py`

```python
recognizer = sherpa_onnx.OfflineRecognizer.from_paraformer(
    paraformer="model.onnx",
    paraformer_eb="model_eb.onnx",  # 可选参数
    tokens="tokens.txt",
    # ...
)
```

**文档说明** (Line 442-444):
```python
paraformer_eb:
    Path to ``model_eb.onnx`` for SeACo-Paraformer (embedding model for
    hotwords). If empty, regular Paraformer is used.
```

### 3. C API

**接口**: `sherpa-onnx/c-api/c-api.h`

```c
struct SherpaOnnxOfflineParaformerModelConfig {
  const char *model;
  const char *model_eb;  // For SeACo-Paraformer
  // ...
};
```

### 4. 命令行接口

**选项**: `--paraformer-eb`

```bash
# 标准 Paraformer（不使用 model_eb）
./sherpa-onnx-offline \
  --tokens=tokens.txt \
  --paraformer=model.onnx \
  audio.wav

# SeACo-Paraformer（使用 model_eb）
./sherpa-onnx-offline \
  --tokens=tokens.txt \
  --paraformer=model.onnx \
  --paraformer-eb=model_eb.onnx \
  --hotwords-file=hotwords.txt \
  audio.wav
```

## 两种模式的支持逻辑

### 模式 1: 标准 Paraformer（不使用 model_eb）

**条件**: `model_eb` 为空或未提供

**代码路径**:
1. **模型加载** (`offline-paraformer-model.cc:44-48`):
   ```cpp
   // Load embedding model if provided (for SeACo-Paraformer)
   if (!config_.paraformer.model_eb.empty()) {
     // 不会执行，因为 model_eb 为空
   }
   ```

2. **推理调用** (`offline-recognizer-paraformer-impl.h:191-198`):
   ```cpp
   if (model_->HasEmbeddingModel() && !hotwords_ids_.empty()) {
     // 不会执行，因为 HasEmbeddingModel() 返回 false
   } else {
     t = model_->Forward(std::move(x), std::move(x_length));
     // 使用两参数版本，标准 Paraformer
   }
   ```

3. **主模型推理** (`offline-paraformer-model.cc:67-146`):
   ```cpp
   std::vector<Ort::Value> Forward(Ort::Value features,
                                   Ort::Value features_length) {
     // 检查模型是否需要 bias_embed
     bool needs_bias_embed = false;
     for (size_t i = 0; i < input_names_.size(); ++i) {
       if (input_names_[i] == "bias_embed") {
         needs_bias_embed = true;
         break;
       }
     }
     
     if (needs_bias_embed) {
       // 如果模型需要但未提供 model_eb，创建空的 bias_embed
       // Shape: (batch_size, 0, embedding_dim)
     } else {
       // 标准 Paraformer，使用两输入
       std::array<Ort::Value, 2> inputs = {...};
     }
   }
   ```

**特点**:
- ✅ 不加载 `model_eb.onnx`
- ✅ 使用标准的 `Forward(features, lengths)` 两参数版本
- ✅ 如果主模型需要 `bias_embed` 输入，自动创建空的 `bias_embed` (shape: `(batch_size, 0, embedding_dim)`)

### 模式 2: SeACo-Paraformer（使用 model_eb）

**条件**: `model_eb` 不为空

**代码路径**:
1. **模型加载** (`offline-paraformer-model.cc:44-48`):
   ```cpp
   if (!config_.paraformer.model_eb.empty()) {
     auto eb_buf = ReadFile(config_.paraformer.model_eb);
     InitEmbedding(eb_buf.data(), eb_buf.size());
     // 加载 model_eb.onnx
   }
   ```

2. **热词处理** (`offline-recognizer-paraformer-impl.h:191-198`):
   ```cpp
   if (model_->HasEmbeddingModel() && !hotwords_ids_.empty()) {
     Ort::Value bias_embed = GenerateBiasEmbed(n);
     t = model_->Forward(std::move(x), std::move(x_length),
                        std::move(bias_embed));
     // 使用三参数版本，带 bias_embed
   }
   ```

3. **Bias Embedding 生成** (`offline-recognizer-paraformer-impl.h:357-415`):
   ```cpp
   Ort::Value GenerateBiasEmbed(int32_t batch_size) const {
     // 1. 填充热词到 (N+1, 10)
     // 2. 调用 model_->ForwardEmbedding()
     // 3. 提取 embeddings
     // 4. 返回 (batch_size, num_hotwords, 512)
   }
   ```

**特点**:
- ✅ 加载 `model_eb.onnx`
- ✅ 如果有热词，生成 `bias_embed` 并使用三参数版本
- ✅ 如果没有热词，创建空的 `bias_embed` (shape: `(batch_size, 0, embedding_dim)`)

## 验证逻辑

**位置**: `sherpa-onnx/csrc/offline-recognizer.cc:84-93`

```cpp
// For SeACo-Paraformer, hotwords can be used with greedy_search
bool is_seaco_paraformer = !model_config.paraformer.model.empty() &&
                            !model_config.paraformer.model_eb.empty();

if (!hotwords_file.empty() && decoding_method != "modified_beam_search" &&
    !is_seaco_paraformer) {
  // 错误：标准 Paraformer 不支持热词 + greedy_search
  SHERPA_ONNX_LOGE("Please use --decoding-method=modified_beam_search ...");
}
```

**逻辑**:
- 如果提供了 `hotwords_file` 且不是 SeACo-Paraformer，必须使用 `modified_beam_search`
- SeACo-Paraformer 可以使用 `greedy_search` + 热词

## 配置验证

**位置**: `sherpa-onnx/csrc/offline-paraformer-model-config.cc:35-48`

```cpp
bool OfflineParaformerModelConfig::Validate() const {
  if (EndsWith(model, ".onnx")) {
    if (!FileExists(model)) {
      return false;
    }
    // Validate embedding model if provided (for SeACo-Paraformer)
    if (!model_eb.empty()) {
      if (!FileExists(model_eb)) {
        SHERPA_ONNX_LOGE("Paraformer embedding model '%s' does not exist",
                         model_eb.c_str());
        return false;
      }
    }
    return true;
  }
  // ...
}
```

**验证规则**:
- ✅ `model.onnx` 必须存在
- ✅ 如果提供了 `model_eb`，必须存在
- ✅ `model_eb` 可以为空（标准 Paraformer）

## 使用示例

### 示例 1: 标准 Paraformer

```cpp
// C++
OfflineRecognizerConfig config;
config.model_config.paraformer.model = "model.onnx";
// model_eb 不设置或设置为空字符串
config.model_config.tokens = "tokens.txt";

auto recognizer = std::make_unique<OfflineRecognizer>(config);
```

```python
# Python
recognizer = sherpa_onnx.OfflineRecognizer.from_paraformer(
    paraformer="model.onnx",
    # paraformer_eb 不提供或提供空字符串
    tokens="tokens.txt",
)
```

```bash
# 命令行
./sherpa-onnx-offline \
  --tokens=tokens.txt \
  --paraformer=model.onnx \
  audio.wav
```

### 示例 2: SeACo-Paraformer（无热词）

```cpp
// C++
OfflineRecognizerConfig config;
config.model_config.paraformer.model = "model.onnx";
config.model_config.paraformer.model_eb = "model_eb.onnx";  // 提供 model_eb
config.model_config.tokens = "tokens.txt";
// hotwords_file 不设置

auto recognizer = std::make_unique<OfflineRecognizer>(config);
```

```python
# Python
recognizer = sherpa_onnx.OfflineRecognizer.from_paraformer(
    paraformer="model.onnx",
    paraformer_eb="model_eb.onnx",  # 提供 model_eb
    tokens="tokens.txt",
)
```

```bash
# 命令行
./sherpa-onnx-offline \
  --tokens=tokens.txt \
  --paraformer=model.onnx \
  --paraformer-eb=model_eb.onnx \
  audio.wav
```

### 示例 3: SeACo-Paraformer（有热词）

```cpp
// C++
OfflineRecognizerConfig config;
config.model_config.paraformer.model = "model.onnx";
config.model_config.paraformer.model_eb = "model_eb.onnx";
config.model_config.tokens = "tokens.txt";
config.hotwords_file = "hotwords.txt";  // 提供热词文件

auto recognizer = std::make_unique<OfflineRecognizer>(config);
```

```python
# Python
recognizer = sherpa_onnx.OfflineRecognizer.from_paraformer(
    paraformer="model.onnx",
    paraformer_eb="model_eb.onnx",
    tokens="tokens.txt",
    hotwords_file="hotwords.txt",  # 提供热词文件
)
```

```bash
# 命令行
./sherpa-onnx-offline \
  --tokens=tokens.txt \
  --paraformer=model.onnx \
  --paraformer-eb=model_eb.onnx \
  --hotwords-file=hotwords.txt \
  audio.wav
```

## 总结

### ✅ 已实现的功能

1. **`model_eb` 已在所有 API 中支持**:
   - C++ API ✅
   - Python API ✅
   - C API ✅
   - 命令行接口 ✅

2. **两种模式都已支持**:
   - 标准 Paraformer（不使用 `model_eb`）✅
   - SeACo-Paraformer（使用 `model_eb`）✅

3. **自动模式切换**:
   - 根据 `model_eb` 是否为空自动选择模式 ✅
   - 如果模型需要 `bias_embed` 但未提供 `model_eb`，自动创建空的 `bias_embed` ✅

### 📝 代码位置总结

| 功能 | 文件 | 行号 |
|------|------|------|
| 配置定义 | `offline-paraformer-model-config.h` | 27-29 |
| 配置注册 | `offline-paraformer-model-config.cc` | 25-27 |
| 配置验证 | `offline-paraformer-model-config.cc` | 42-48 |
| 模型加载 | `offline-paraformer-model.cc` | 44-48, 60-64 |
| 推理流程 | `offline-recognizer-paraformer-impl.h` | 191-198 |
| Python 绑定 | `offline_recognizer.py` | 442-449 |
| C API | `c-api.h` | 422 |

## 结论

**当前实现已经完全支持两种模式，无需额外修改。** `model_eb` 是一个可选参数：
- 如果为空或未提供 → 使用标准 Paraformer
- 如果提供 → 使用 SeACo-Paraformer

代码会根据 `model_eb` 的存在与否自动选择正确的模式。
