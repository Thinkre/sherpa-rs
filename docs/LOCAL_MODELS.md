# KeVoiceInput 本地模型列表

本文档整理了 KeVoiceInput 支持的所有本地语音识别模型，包括模型信息、调用接口、配置参数和下载地址。

## 目录
- [Whisper 系列模型](#whisper-系列模型)
- [Transducer 系列模型](#transducer-系列模型)
- [FireRedAsr 系列模型](#fireredasr-系列模型)
- [Paraformer 系列模型](#paraformer-系列模型)
- [配置参数详解](#配置参数详解)

---

## Whisper 系列模型

基于 OpenAI Whisper 的语音识别模型，使用 whisper.cpp 进行推理。

### 1. Whisper Small

**模型信息**
- **模型 ID**: `small`
- **模型名称**: Whisper Small
- **描述**: 快速且相当准确
- **文件名**: `ggml-small.bin`
- **大小**: 487 MB
- **准确度评分**: 0.60
- **速度评分**: 0.85

**下载地址**（应用内使用 Hugging Face / hf-mirror 下载）
- 主：`https://hf-mirror.com/ggerganov/whisper.cpp/resolve/main/ggml-small.bin`
- 备用：`https://blob.handy.computer/ggml-small.bin`
- 仓库（可选其他量化）：<https://hf-mirror.com/ggerganov/whisper.cpp>

**调用接口**
- 引擎类型: `EngineType::Whisper`
- 识别器: `WhisperRecognizer` (whisper.cpp)

---

### 2. Whisper Medium

**模型信息**
- **模型 ID**: `medium`
- **模型名称**: Whisper Medium
- **描述**: 准确度好，速度中等
- **文件名**: `whisper-medium-q4_1.bin`
- **大小**: 492 MB
- **准确度评分**: 0.75
- **速度评分**: 0.60

**下载地址**（应用内使用 Hugging Face / hf-mirror 下载）
- 主：`https://hf-mirror.com/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin`
- 备用：`https://blob.handy.computer/whisper-medium-q4_1.bin`
- 仓库（可选其他量化）：<https://hf-mirror.com/ggerganov/whisper.cpp>

**调用接口**
- 引擎类型: `EngineType::Whisper`
- 识别器: `WhisperRecognizer` (whisper.cpp)

---

### 3. Whisper Turbo

**模型信息**
- **模型 ID**: `turbo`
- **模型名称**: Whisper Turbo
- **描述**: 准确度和速度平衡
- **文件名**: `ggml-large-v3-turbo.bin`
- **大小**: 1600 MB
- **准确度评分**: 0.80
- **速度评分**: 0.40

**下载地址**（应用内使用 Hugging Face / hf-mirror 下载）
- 主：`https://hf-mirror.com/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin`
- 备用：`https://blob.handy.computer/ggml-large-v3-turbo.bin`
- 仓库：<https://hf-mirror.com/ggerganov/whisper.cpp>

**调用接口**
- 引擎类型: `EngineType::Whisper`
- 识别器: `WhisperRecognizer` (whisper.cpp)

---

### 4. Whisper Large

**模型信息**
- **模型 ID**: `large`
- **模型名称**: Whisper Large
- **描述**: 准确度高，但速度慢
- **文件名**: `ggml-large-v3-q5_0.bin`
- **大小**: 1100 MB
- **准确度评分**: 0.85
- **速度评分**: 0.30

**下载地址**（应用内使用 Hugging Face / hf-mirror 下载）
- 主：`https://hf-mirror.com/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-q5_0.bin`
- 备用：`https://blob.handy.computer/ggml-large-v3-q5_0.bin`
- 仓库：<https://hf-mirror.com/ggerganov/whisper.cpp>

**调用接口**
- 引擎类型: `EngineType::Whisper`
- 识别器: `WhisperRecognizer` (whisper.cpp)

---

## Transducer 系列模型

基于 RNN-T (Transducer) 架构的模型，支持热词功能，使用 sherpa-onnx 进行推理。

### 5. Zipformer 双语 (中英)

**模型信息**
- **模型 ID**: `zipformer-zh-en`
- **模型名称**: Zipformer 双语
- **描述**: 支持中文和英文。支持热词功能，识别准确度高。
- **目录名**: `zipformer-zh-en`
- **大小**: 320 MB
- **准确度评分**: 0.93
- **速度评分**: 0.70
- **建模单元**: cjkchar+bpe

**下载地址**
```
https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-zipformer-zh-en-2023-11-22.tar.bz2
```

**模型文件结构**
```
zipformer-zh-en/
├── encoder-epoch-20-avg-1-chunk-16-left-128.int8.onnx  (248 MB)
├── decoder-epoch-20-avg-1-chunk-16-left-128.int8.onnx  (4.9 MB)
├── joiner-epoch-20-avg-1-chunk-16-left-128.int8.onnx   (3.9 MB)
└── tokens.txt
```

**调用接口**
- 引擎类型: `EngineType::Transducer`
- 识别器: `TransducerRecognizer` (sherpa-onnx)
- 支持热词: ✅

**配置参数**
```rust
TransducerConfig {
    encoder: "encoder-epoch-20-avg-1-chunk-16-left-128.int8.onnx",
    decoder: "decoder-epoch-20-avg-1-chunk-16-left-128.int8.onnx",
    joiner: "joiner-epoch-20-avg-1-chunk-16-left-128.int8.onnx",
    tokens: "tokens.txt",
    num_threads: 1,
    sample_rate: 16000,
    feature_dim: 80,
    decoding_method: "greedy_search",
    hotwords_file: "",      // 热词文件路径（可选）
    hotwords_score: 2.0,    // 热词分数
    modeling_unit: "cjkchar+bpe",
    bpe_vocab: "",
    blank_penalty: 0.0,
    model_type: "",
    debug: false,
    provider: "cpu",        // 或 "coreml"
}
```

---

### 6. Conformer 中文

**模型信息**
- **模型 ID**: `conformer-zh-stateless2`
- **模型名称**: Conformer 中文
- **描述**: 中文识别模型，使用 cjkchar 建模单元，对中文热词支持更好。
- **目录名**: `conformer-zh-stateless2`
- **大小**: 50 MB
- **准确度评分**: 0.90
- **速度评分**: 0.80
- **建模单元**: cjkchar

**下载地址**
```
https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-conformer-zh-stateless2-2023-05-23.tar.bz2
```

**模型文件结构**
```
conformer-zh-stateless2/
├── encoder-epoch-*.onnx
├── decoder-epoch-*.onnx
├── joiner-epoch-*.onnx
└── tokens.txt
```

**调用接口**
- 引擎类型: `EngineType::Transducer`
- 识别器: `TransducerRecognizer` (sherpa-onnx)
- 支持热词: ✅

**配置参数**
```rust
TransducerConfig {
    encoder: "encoder-epoch-*.onnx",
    decoder: "decoder-epoch-*.onnx",
    joiner: "joiner-epoch-*.onnx",
    tokens: "tokens.txt",
    num_threads: 1,
    sample_rate: 16000,
    feature_dim: 80,
    decoding_method: "greedy_search",
    hotwords_file: "",      // 热词文件路径（可选）
    hotwords_score: 2.0,    // 热词分数
    modeling_unit: "cjkchar",
    bpe_vocab: "",
    blank_penalty: 0.0,
    model_type: "",
    debug: false,
    provider: "cpu",        // 或 "coreml"
}
```

---

## FireRedAsr 系列模型

基于 Conformer 架构的方言模型，支持中文普通话和多种方言，使用 sherpa-onnx 进行推理。

### 7. FireRedAsr Large 中英双语

**模型信息**
- **模型 ID**: `fire-red-asr-large-zh-en`
- **模型名称**: FireRedAsr Large 中英双语
- **描述**: 支持中文（普通话、四川话、天津话、河南话等方言）和英文。识别准确度高。
- **目录名**: `sherpa-onnx-fire-red-asr-large-zh_en-2025-02-16`
- **大小**: 1700 MB
- **准确度评分**: 0.95
- **速度评分**: 0.60

**下载地址**
```
https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-fire-red-asr-large-zh_en-2025-02-16.tar.bz2
```

**模型文件结构**
```
sherpa-onnx-fire-red-asr-large-zh_en-2025-02-16/
├── encoder-epoch-99-avg-1.int8.onnx   (425 MB)
├── decoder-epoch-99-avg-1.int8.onnx   (1.3 GB)
├── joiner-epoch-99-avg-1.int8.onnx
└── tokens.txt
```

**调用接口**
- 引擎类型: `EngineType::FireRedAsr`
- 识别器: `SherpaOnnxOfflineRecognizer` (sherpa-onnx C API)
- 支持热词: ❌
- 支持方言: ✅ (普通话、四川话、天津话、河南话等)

**配置参数**
```rust
SherpaOnnxOfflineRecognizerConfig {
    model_config: {
        transducer: {
            encoder: "encoder-epoch-99-avg-1.int8.onnx",
            decoder: "decoder-epoch-99-avg-1.int8.onnx",
            joiner: "joiner-epoch-99-avg-1.int8.onnx",
        },
        tokens: "tokens.txt",
        num_threads: 1,
        debug: false,
        provider: "cpu",
        model_type: "",
        modeling_unit: "",
        bpe_vocab: "",
    },
    feat_config: {
        sample_rate: 16000,
        feature_dim: 80,
    },
    decoding_method: "greedy_search",
    max_active_paths: 4,
    hotwords_file: null,
    hotwords_score: 0.0,
    blank_penalty: 0.0,
}
```

---

## Paraformer 系列模型

基于 Paraformer 架构的模型，支持中文语音识别。SeACo Paraformer 变体支持热词功能。

### 8. Paraformer 模型（自定义导入）

**模型信息**
- **引擎类型**: `EngineType::Paraformer`
- **识别器**: `ParaformerRecognizer` (sherpa-onnx)
- **支持热词**: ❌
- **导入方式**: 用户通过"导入本地模型"功能添加

**必需文件**
```
<model-directory>/
├── model.onnx (或 model.int8.onnx)
└── tokens.txt
```

**调用接口**
```rust
ParaformerConfig {
    model: "model.onnx",
    tokens: "tokens.txt",
    model_eb: None,         // Paraformer 不使用
    hotwords_file: None,
    hotwords_score: 0.0,
    provider: Some("cpu"),  // 或 "coreml"
    num_threads: Some(1),
    debug: false,
}
```

---

### 9. SeACo Paraformer 模型（自定义导入）

**模型信息**
- **模型 ID**: `custom-xxx` (自动生成)
- **模型名称**: KeSeACoParaformer
- **引擎类型**: `EngineType::SeacoParaformer`
- **识别器**: `ParaformerRecognizer` (sherpa-onnx)
- **支持热词**: ✅
- **导入方式**: 用户通过"导入本地模型"功能添加

**必需文件**
```
<model-directory>/
├── model.onnx         (主模型，必需)
├── model_eb.onnx      (嵌入模型，热词支持，必需)
└── tokens.txt 或 tokens.json  (词表，必需)

可选文件：
├── am.mvn             (声学模型均值方差归一化，可选)
└── config.yaml        (配置文件，可选)
```

**调用接口**
```rust
ParaformerConfig {
    model: "model.onnx",
    tokens: "tokens.txt",      // 或 "tokens.json"
    model_eb: Some("model_eb.onnx"),  // SeACo Paraformer 必需
    hotwords_file: Some("hotwords.txt"),  // 热词文件（可选）
    hotwords_score: 2.0,       // 热词分数
    provider: Some("cpu"),     // 或 "coreml"
    num_threads: Some(1),
    debug: false,
}
```

**热词文件格式** (`hotwords.txt`)
```
每行一个热词
可以包含中文
可以包含英文
可以包含数字
```

---

## 配置参数详解

### 通用参数

所有模型都支持以下通用参数：

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `num_threads` | `i32` | `1` | 推理线程数 |
| `debug` | `bool` | `false` | 是否输出调试信息 |
| `provider` | `String` | `"cpu"` | 推理后端: `"cpu"`, `"coreml"` (macOS) |
| `sample_rate` | `i32` | `16000` | 音频采样率 (Hz) |

---

### Whisper 特定参数

Whisper 模型使用 whisper.cpp 进行推理，无需额外配置参数。

**代码位置**: `src-tauri/src/managers/transcription.rs:700-850`

---

### Transducer 特定参数

用于 Zipformer 和 Conformer 模型。

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `encoder` | `String` | - | Encoder 模型路径 |
| `decoder` | `String` | - | Decoder 模型路径 |
| `joiner` | `String` | - | Joiner 模型路径 |
| `tokens` | `String` | - | 词表文件路径 |
| `feature_dim` | `i32` | `80` | 特征维度 |
| `decoding_method` | `String` | `"greedy_search"` | 解码方法 |
| `hotwords_file` | `String` | `""` | 热词文件路径 |
| `hotwords_score` | `f32` | `2.0` | 热词权重分数 |
| `modeling_unit` | `String` | - | 建模单元: `"cjkchar"`, `"bpe"`, `"cjkchar+bpe"` |
| `bpe_vocab` | `String` | `""` | BPE 词表路径 |
| `blank_penalty` | `f32` | `0.0` | 空白符惩罚 |
| `max_active_paths` | `i32` | `4` | 最大激活路径数 |

**代码位置**: `src-tauri/src/managers/transcription.rs:1100-1250`

---

### Paraformer / SeACo Paraformer 特定参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `model` | `String` | - | 主模型路径 (`model.onnx`) |
| `tokens` | `String` | - | 词表文件路径 (`tokens.txt` 或 `tokens.json`) |
| `model_eb` | `Option<String>` | `None` | 嵌入模型路径 (仅 SeACo Paraformer) |
| `hotwords_file` | `Option<String>` | `None` | 热词文件路径 (仅 SeACo Paraformer) |
| `hotwords_score` | `f32` | `0.0` | 热词权重分数 (仅 SeACo Paraformer) |

**代码位置**:
- Paraformer: `src-tauri/src/managers/transcription.rs:900-1100`
- SeACo Paraformer: `src-tauri/src/managers/transcription.rs:1400-1600`

---

### FireRedAsr 特定参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `encoder` | `String` | - | Encoder 模型路径 |
| `decoder` | `String` | - | Decoder 模型路径 |
| `joiner` | `String` | - | Joiner 模型路径 |
| `tokens` | `String` | - | 词表文件路径 |
| `feature_dim` | `i32` | `80` | 特征维度 |
| `decoding_method` | `String` | `"greedy_search"` | 解码方法 |
| `max_active_paths` | `i32` | `4` | 最大激活路径数 |

**代码位置**: `src-tauri/src/managers/transcription.rs:1250-1400`

---

## 依赖库版本

### ONNX Runtime
- **当前版本**: `1.23.2`
- **架构**: Universal (x86_64 + arm64)
- **路径**: `src-tauri/target/release/libonnxruntime.1.23.2.dylib`

### sherpa-onnx
- **位置**: `vendor/sherpa-onnx/`
- **版本**: 编译时依赖 onnxruntime 1.23.2
- **库文件**:
  - `libsherpa-onnx-c-api.dylib`
  - `libsherpa-onnx-cxx-api.dylib`

### whisper.cpp
- **位置**: `vendor/whisper.cpp/`
- **绑定**: `whisper-rs` crate
- **推理引擎**: CoreML (macOS) 或 CPU

---

## 模型存储位置

**用户模型目录**:
```
~/Library/Application Support/com.kevoiceinput.app/models/
```

**模型文件结构**:
```
models/
├── ggml-small.bin                              (Whisper Small)
├── whisper-medium-q4_1.bin                     (Whisper Medium)
├── ggml-large-v3-turbo.bin                     (Whisper Turbo)
├── ggml-large-v3-q5_0.bin                      (Whisper Large)
├── zipformer-zh-en/                            (Zipformer 双语)
│   ├── encoder-epoch-20-avg-1-chunk-16-left-128.int8.onnx
│   ├── decoder-epoch-20-avg-1-chunk-16-left-128.int8.onnx
│   ├── joiner-epoch-20-avg-1-chunk-16-left-128.int8.onnx
│   └── tokens.txt
├── conformer-zh-stateless2/                    (Conformer 中文)
│   ├── encoder-epoch-*.onnx
│   ├── decoder-epoch-*.onnx
│   ├── joiner-epoch-*.onnx
│   └── tokens.txt
└── sherpa-onnx-fire-red-asr-large-zh_en-2025-02-16/  (FireRedAsr Large)
    ├── encoder-epoch-99-avg-1.int8.onnx
    ├── decoder-epoch-99-avg-1.int8.onnx
    ├── joiner-epoch-99-avg-1.int8.onnx
    └── tokens.txt
```

---

## 相关文件

- **模型管理器**: `src-tauri/src/managers/model.rs`
- **转录管理器**: `src-tauri/src/managers/transcription.rs`
- **Paraformer 配置**: `vendor/sherpa-rs/crates/sherpa-rs/src/paraformer.rs`
- **Transducer 配置**: `vendor/sherpa-rs/crates/sherpa-rs/src/transducer.rs`
- **ONNX Runtime 升级脚本**: `upgrade-onnxruntime-fast.sh`

---

## 测试命令

### 查看模型目录
```bash
ls -la ~/Library/Application\ Support/com.kevoiceinput.app/models/
```

### 查看模型大小
```bash
du -sh ~/Library/Application\ Support/com.kevoiceinput.app/models/*
```

### 查看应用日志
```bash
tail -f ~/Library/Logs/com.kevoiceinput.app/main.log | grep -E "model|engine|transcription"
```

---

## 添加新模型

### 预定义模型（需修改代码）

1. 在 `src-tauri/src/managers/model.rs` 的 `new()` 方法中添加模型定义
2. 指定引擎类型: `EngineType::Whisper`, `EngineType::Transducer`, `EngineType::Paraformer`, `EngineType::FireRedAsr`, 或 `EngineType::SeacoParaformer`
3. 提供下载 URL 和模型元数据
4. 重新编译应用

### 自定义模型（无需修改代码）

1. 打开 KeVoiceInput 应用
2. 进入"模型"页面
3. 点击"导入本地模型"
4. 选择模型文件夹
5. 应用会自动识别模型类型（Paraformer 或 SeACo Paraformer）

**模型识别规则**:
- 如果有 `model_eb.onnx` + `model.onnx` + `tokens.txt/json` → SeACo Paraformer
- 如果有 `model.onnx` + `tokens.txt` → Paraformer
- 其他格式暂不支持自动导入

---

## 版本信息

- **文档版本**: 1.0
- **应用版本**: 0.0.1
- **更新日期**: 2026-02-27
- **ONNX Runtime**: 1.23.2

---

## 注意事项

1. **onnxruntime 版本**:
   - 必须使用 1.23.2 版本
   - sherpa-onnx 库编译时依赖此版本
   - 版本不匹配会导致应用崩溃 (SIGSEGV)

2. **模型文件完整性**:
   - Transducer 模型需要 3 个 ONNX 文件 (encoder, decoder, joiner)
   - Paraformer 需要 model.onnx 和 tokens.txt
   - SeACo Paraformer 额外需要 model_eb.onnx

3. **热词功能**:
   - 支持热词: Transducer, SeACo Paraformer
   - 不支持热词: Whisper, Paraformer, FireRedAsr

4. **方言支持**:
   - FireRedAsr Large 支持: 普通话、四川话、天津话、河南话等
   - 其他模型仅支持普通话

5. **推理后端**:
   - macOS: 可使用 CoreML 或 CPU
   - 其他平台: 仅 CPU

---

## 故障排查

### 模型下载失败
- 检查网络连接
- 尝试使用代理: `export https_proxy=http://127.0.0.1:7897`
- 使用镜像地址: `upgrade-onnxruntime-fast.sh`

### 模型加载失败
- 检查模型文件完整性
- 查看日志: `~/Library/Logs/com.kevoiceinput.app/main.log`
- 确认 onnxruntime 版本为 1.23.2

### 应用崩溃
- 确认 onnxruntime 版本匹配
- 运行: `otool -L /Applications/KeVoiceInput.app/Contents/Frameworks/libsherpa-onnx-c-api.dylib`
- 重新构建应用: `./scripts/tauri-build-wrapper.sh build`

---

**文档完成！** 🎉
