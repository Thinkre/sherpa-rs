# SeACo-Paraformer 实现对比：sherpa-onnx vs FunASR

## 概述

本文档对比了 sherpa-onnx 中的 SeACo-Paraformer 实现和 FunASR 中的 ContextualParaformer 实现，分析它们的相同点和差异。

## 核心结论

**基本一致**：sherpa-onnx 的实现遵循了 FunASR 的 ContextualParaformer 实现逻辑，核心算法和数据处理流程相同，但在实现细节上有一些差异。

## 详细对比

### 1. 热词处理流程对比

#### 1.1 热词编码

**FunASR (`proc_hotword`)**:
```python
def proc_hotword(self, hotwords):
    hotwords = hotwords.split(" ")  # 按空格分割
    hotwords_length = [len(i) - 1 for i in hotwords]  # 字符串长度减1
    hotwords_length.append(0)  # 添加 <s> token 的长度
    
    def word_map(word):
        hotwords = []
        for c in word:
            if c not in self.vocab.keys():
                hotwords.append(8403)  # <unk> token
            else:
                hotwords.append(self.vocab[c])
        return np.array(hotwords)
    
    hotword_int = [word_map(i) for i in hotwords]
    hotword_int.append(np.array([1]))  # <s> token ID = 1
    hotwords = pad_list(hotword_int, pad_value=0, max_len=10)
    
    return hotwords, hotwords_length
```

**sherpa-onnx (`EncodeHotwordsForSeaco` + `GenerateBiasEmbed`)**:
```cpp
// 1. 读取热词文件，按行分割
// 2. 对每个热词：
std::vector<std::string> chars = SplitUtf8(line);  // UTF-8 字符分割
std::vector<int32_t> ids;
for (const auto &ch : chars) {
    if (!symbol_table_.Contains(ch)) {
        // 跳过未知字符的热词
        has_unknown = true;
        break;
    }
    ids.push_back(symbol_table_[ch]);
}
hotwords_ids_.push_back(ids);

// 3. 在 GenerateBiasEmbed 中：
int32_t sos_id = symbol_table_["<s>"];
const int32_t max_len = 10;
for (size_t i = 0; i < hotwords_ids_.size(); ++i) {
    const auto &ids = hotwords_ids_[i];
    lengths[i] = std::max(0, static_cast<int32_t>(ids.size()) - 1);  // token IDs 长度减1
    // 填充到 max_len
}
// 添加 <s> token
padded_ids[num_hotwords * max_len] = sos_id;
lengths[num_hotwords] = 0;
```

**对比**:
- ✅ **相同点**: 
  - 都使用 `max_len=10` 进行填充
  - 都计算 `len(ids) - 1` 作为有效长度
  - 都添加 `<s>` token 作为最后一项
  - 都使用 `0` 作为 padding 值
  
- ⚠️ **差异点**:
  - **FunASR**: 使用字符串长度 `len(i) - 1`（对于中文字符，字符串长度 = 字符数）
  - **sherpa-onnx**: 使用 token ID 序列长度 `ids.size() - 1`
  - **FunASR**: 未知字符替换为 `<unk>` (8403)
  - **sherpa-onnx**: 包含未知字符的热词被完全跳过

**影响**: 对于纯中文字符串，两者结果相同；但如果热词包含多字节字符或特殊处理，可能会有细微差异。

#### 1.2 Embedding 模型调用

**FunASR (`eb_infer`)**:
```python
def eb_infer(self, hotwords, hotwords_length):
    # hotwords shape: (N+1, 10), dtype=int32
    outputs = self.ort_infer_eb([hotwords.astype(np.int32)])
    return outputs
```

**sherpa-onnx (`ForwardEmbedding`)**:
```cpp
std::vector<Ort::Value> ForwardEmbedding(Ort::Value input_ids) {
    // input_ids shape: (N+1, 10), dtype=int32
    std::array<Ort::Value, 1> inputs = {std::move(input_ids)};
    return embedding_sess_->Run({}, embedding_input_names_ptr_.data(),
                                inputs.data(), inputs.size(),
                                embedding_output_names_ptr_.data(),
                                embedding_output_names_ptr_.size());
}
```

**对比**:
- ✅ **完全相同**: 输入形状 `(N+1, 10)`，dtype=int32

#### 1.3 Embedding 提取和索引

**FunASR**:
```python
[bias_embed] = self.eb_infer(hotwords, hotwords_length)
# bias_embed shape: (10, N+1, 512) - time-first format
bias_embed = bias_embed.transpose(1, 0, 2)  # -> (N+1, 10, 512)
_ind = np.arange(0, len(hotwords)).tolist()  # [0, 1, 2, ..., N]
bias_embed = bias_embed[_ind, hotwords_length.tolist()]  # -> (N+1, 512)
# 提取每个热词在对应长度位置的 embedding
# 移除最后一个 <s> token
bias_embed = bias_embed[:-1]  # -> (N, 512)
```

**sherpa-onnx (`TransposeAndIndex`)**:
```cpp
// embeddings shape: (10, N+1, 512) - time-first format
int64_t T = shape[0];  // max_len = 10
int64_t N = shape[1];  // num_hotwords + 1
int64_t D = shape[2];  // embedding_dim = 512

// 直接从 (T, N, D) 布局提取
for (int64_t i = 0; i < num_valid_hotwords; ++i) {
    int32_t len = lengths[i];
    // 提取 embeddings[len, i, :] from (T, N, D) layout
    const float *src = data + len * N * D + i * D;
    float *dst = result_data.data() + i * D;
    std::copy(src, src + D, dst);
}
// 结果: (num_hotwords, 512)
```

**对比**:
- ✅ **逻辑相同**: 都从 embedding 矩阵中提取每个热词在有效长度位置的 embedding
- ⚠️ **实现方式不同**:
  - **FunASR**: 先转置为 `(N+1, 10, 512)`，然后使用索引 `[i, lengths[i]]` 提取
  - **sherpa-onnx**: 直接从 `(10, N+1, 512)` 布局计算内存偏移提取
  
**数学等价性**:
```
FunASR: bias_embed[i, lengths[i], :]  (转置后)
sherpa-onnx: embeddings[lengths[i], i, :]  (原始布局)

两者等价，因为:
bias_embed[i, lengths[i], :] = embeddings.transpose(1,0,2)[i, lengths[i], :]
                            = embeddings[lengths[i], i, :]
```

#### 1.4 Batch 扩展

**FunASR**:
```python
bias_embed = np.expand_dims(bias_embed, axis=0)  # (N, 512) -> (1, N, 512)
bias_embed = np.repeat(bias_embed, feats.shape[0], axis=0)  # -> (batch_size, N, 512)
```

**sherpa-onnx (`TransposeAndIndex`)**:
```cpp
// result_data shape: (num_hotwords, 512)
std::array<int64_t, 3> output_shape{batch_size, num_valid_hotwords, D};
Ort::Value result = Ort::Value::CreateTensor<float>(...);

// 复制到 batch 的每个样本
for (int32_t b = 0; b < batch_size; ++b) {
    std::copy(result_data.begin(), result_data.end(),
              result_ptr + b * num_hotwords * D);
}
```

**对比**:
- ✅ **完全相同**: 都将单个样本的 bias_embed 复制到整个 batch

### 2. 主模型推理对比

#### 2.1 模型输入

**FunASR (`bb_infer`)**:
```python
def bb_infer(self, feats, feats_len, bias_embed):
    outputs = self.ort_infer_bb([feats, feats_len, bias_embed])
    return outputs
```

**sherpa-onnx (`Forward`)**:
```cpp
std::vector<Ort::Value> Forward(Ort::Value features,
                                Ort::Value features_length,
                                Ort::Value bias_embed) {
    // 根据 input_names 确定输入顺序
    std::vector<Ort::Value> inputs;
    for (size_t i = 0; i < input_names_.size(); ++i) {
        const std::string &name = input_names_[i];
        if (name == "speech" || name == "features") {
            inputs.push_back(std::move(features_clone));
        } else if (name == "speech_lengths" || ...) {
            inputs.push_back(std::move(features_length_clone));
        } else if (name == "bias_embed") {
            inputs.push_back(std::move(bias_embed));
        }
    }
    return sess_->Run(...);
}
```

**对比**:
- ✅ **输入相同**: `feats`, `feats_len`, `bias_embed`
- ⚠️ **差异**: 
  - **FunASR**: 固定顺序 `[feats, feats_len, bias_embed]`
  - **sherpa-onnx**: 根据模型的 `input_names` 动态确定顺序（更灵活）

### 3. 特征提取对比

#### 3.1 LFR + CMVN

**FunASR**:
```python
def extract_feat(self, waveform_list):
    feats, feats_len = [], []
    for waveform in waveform_list:
        speech, _ = self.frontend.fbank(waveform)  # Mel 特征
        feat, feat_len = self.frontend.lfr_cmvn(speech)  # LFR + CMVN
        feats.append(feat)
        feats_len.append(feat_len)
    feats = self.pad_feats(feats, np.max(feats_len))
    return feats, feats_len
```

**sherpa-onnx**:
```cpp
// 1. GetFrames() - Mel 特征提取
std::vector<float> f = ss[i]->GetFrames();

// 2. ApplyLFR()
f = ApplyLFR(f);  // (num_frames, 80) -> (out_frames, 560)

// 3. ApplyCMVN()
ApplyCMVN(&f);  // 原地归一化

// 4. Padding
Ort::Value x = PadSequence(...);
```

**对比**:
- ✅ **流程相同**: Mel → LFR → CMVN → Padding
- ✅ **参数相同**: LFR window_size=7, shift=6

### 4. 解码对比

**FunASR**:
```python
def decode_one(self, am_score, valid_token_num):
    yseq = am_score.argmax(axis=-1)  # 贪心搜索
    yseq = np.array([1] + yseq.tolist() + [2])  # 添加 sos/eos
    token_int = hyp.yseq[1:last_pos].tolist()
    token_int = list(filter(lambda x: x not in (0, 2), token_int))  # 移除 blank 和 eos
    token = self.converter.ids2tokens(token_int)
    token = token[: valid_token_num - self.pred_bias]
    return token
```

**sherpa-onnx**:
```cpp
for (int32_t k = 0; k != num_tokens; ++k) {
    auto max_idx = std::distance(p, std::max_element(p, p + vocab_size));
    if (max_idx == eos_id_) {
        break;  // 遇到结束符停止
    }
    results[i].tokens.push_back(max_idx);
    p += vocab_size;
}
```

**对比**:
- ✅ **算法相同**: 都是贪心搜索（argmax）
- ⚠️ **差异**:
  - **FunASR**: 显式添加 sos/eos，然后移除
  - **sherpa-onnx**: 遇到 eos 直接停止，不添加额外的 sos/eos
  - **FunASR**: 使用 `pred_bias` 调整 token 数量
  - **sherpa-onnx**: 直接使用 `token_num`

## 关键差异总结

| 方面 | FunASR | sherpa-onnx | 影响 |
|------|--------|-------------|------|
| **热词长度计算** | 字符串长度 `len(i) - 1` | Token IDs 长度 `ids.size() - 1` | 对纯中文相同，多字节字符可能不同 |
| **未知字符处理** | 替换为 `<unk>` (8403) | 跳过整个热词 | sherpa-onnx 更严格 |
| **Embedding 提取** | 转置后索引 | 直接内存偏移 | 数学等价，实现不同 |
| **输入顺序** | 固定顺序 | 动态根据 input_names | sherpa-onnx 更灵活 |
| **解码处理** | 显式添加/移除 sos/eos | 遇到 eos 停止 | 结果相同，实现不同 |
| **pred_bias** | 支持 | 不支持 | FunASR 有额外调整 |

## 兼容性分析

### ✅ 完全兼容的部分

1. **模型文件**: 两者使用相同的 ONNX 模型（`model.onnx` 和 `model_eb.onnx`）
2. **数据格式**: 输入输出形状和数据类型完全一致
3. **核心算法**: Embedding 提取、索引逻辑数学等价
4. **特征提取**: LFR、CMVN 参数和处理流程相同

### ⚠️ 需要注意的差异

1. **热词文件格式**:
   - **FunASR**: 空格分隔的字符串，如 `"停滞 交易 情况"`
   - **sherpa-onnx**: 每行一个热词的文件，如：
     ```
     停滞
     交易
     情况
     ```

2. **未知字符处理**:
   - **FunASR**: 会保留热词，未知字符替换为 `<unk>`
   - **sherpa-onnx**: 会跳过包含未知字符的整个热词

3. **pred_bias**:
   - **FunASR**: 支持 `predictor_bias` 配置参数
   - **sherpa-onnx**: 未实现此功能

## 代码引用

sherpa-onnx 的实现中明确注释了遵循 FunASR 的实现：

```cpp
// Following FunASR's ContextualParaformer implementation:
// 1. For each hotword, create [token_ids] sequence
// 2. Calculate length for each hotword (len(ids) - 1, minimum 0)
// 3. Pad sequences to max_len=10
// 4. Call embedding model
// 5. Extract embeddings at the last valid position using lengths
```

## 结论

**sherpa-onnx 的 SeACo-Paraformer 实现与 FunASR 的 ContextualParaformer 在核心算法和数据处理流程上是一致的**。主要差异在于：

1. **实现语言**: Python vs C++
2. **代码组织**: 不同的代码结构和命名
3. **边界情况处理**: 未知字符、输入格式等细节差异
4. **灵活性**: sherpa-onnx 在输入顺序处理上更灵活

这些差异不会影响模型的正确性和性能，两者可以互换使用相同的模型文件，产生相同的识别结果（在相同输入和配置下）。
