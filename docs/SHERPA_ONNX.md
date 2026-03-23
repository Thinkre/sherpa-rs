# sherpa-onnx 集成指南

本文档详细说明 KeVoiceInput 如何集成 sherpa-onnx 进行设备端语音识别。

## 概述

sherpa-onnx 是新一代 Kaldi (k2-fsa) 的设备端语音识别库，支持多种模型格式和平台。KeVoiceInput 通过 vendored sherpa-rs 集成 sherpa-onnx，实现高质量的本地语音识别。

## 架构

```
KeVoiceInput App
    ↓
sherpa-rs (Rust bindings)
    ↓
sherpa-onnx (C++ library)
    ↓
ONNX Runtime
    ↓
模型文件 (.onnx)
```

## Vendored sherpa-rs

### 来源

- **上游仓库**: [thewh1teagle/sherpa-rs](https://github.com/thewh1teagle/sherpa-rs)
- **本地路径**: `vendor/sherpa-rs/`

### 为什么使用 Vendored 版本？

1. **定制修改**: 添加了 SeACo Paraformer 的 `model_eb.onnx` 支持
2. **版本控制**: 确保与 KeVoiceInput 兼容的特定版本
3. **快速迭代**: 无需等待上游合并即可添加新功能

### 修改内容

- 添加 `model_eb.onnx` 参数支持 (SeACo Paraformer 热词嵌入模型)
- 优化错误处理和日志输出
- 针对 KeVoiceInput 的性能优化

## 支持的模型类型

### 1. Transducer (Zipformer/Conformer)

**特点**:
- 流式识别，低延迟
- 支持热词
- 适合实时转录

**模型文件**:
```
model_directory/
├── encoder-epoch-99-avg-1.onnx  # 编码器
├── decoder-epoch-99-avg-1.onnx  # 解码器
├── joiner-epoch-99-avg-1.onnx   # 连接器
└── tokens.txt                    # 词表
```

**用途**: 中英文双语、中文专用模型

### 2. Paraformer

**特点**:
- 非流式识别，高准确度
- 快速推理
- 适合离线转录

**模型文件**:
```
model_directory/
├── model.onnx   # Paraformer 模型
└── tokens.txt   # 词表
```

**用途**: 中文识别

### 3. SeACo Paraformer

**特点**:
- 基于 Paraformer，增加热词支持
- 使用 `model_eb.onnx` 实现热词嵌入

**模型文件**:
```
model_directory/
├── model.onnx       # 标准 Paraformer 模型
├── model_eb.onnx    # 热词嵌入模型 (SeACo 专用)
└── tokens.txt       # 词表
```

**识别机制**:
- 应用检测 `model_eb.onnx` 存在自动启用热词功能
- 运行时动态加载热词列表
- 通过嵌入向量提高特定词汇识别率

**用途**: 中文识别 + 热词功能

## 环境配置

### 构建时配置

#### 1. 设置 SHERPA_LIB_PATH

指向本地编译的 sherpa-onnx 库：

```bash
# macOS/Linux
export SHERPA_LIB_PATH=/path/to/sherpa-onnx/install/lib

# Windows
set SHERPA_LIB_PATH=C:\path\to\sherpa-onnx\install\lib
```

#### 2. 编译 sherpa-onnx

```bash
git clone https://github.com/k2-fsa/sherpa-onnx.git
cd sherpa-onnx

mkdir build && cd build
cmake -DCMAKE_BUILD_TYPE=Release \
      -DCMAKE_INSTALL_PREFIX=../install \
      -DSHERPA_ONNX_ENABLE_PYTHON=OFF \
      -DSHERPA_ONNX_ENABLE_TESTS=OFF \
      -DSHERPA_ONNX_ENABLE_CHECK=OFF \
      -DBUILD_SHARED_LIBS=ON \
      ..

make -j$(nproc)
make install
```

#### 3. 配置 Cargo.toml

vendored sherpa-rs 的 `build.rs` 会读取 `SHERPA_LIB_PATH`:

```rust
// vendor/sherpa-rs/build.rs
if let Ok(sherpa_lib_path) = env::var("SHERPA_LIB_PATH") {
    println!("cargo:rustc-link-search=native={}", sherpa_lib_path);
}
```

### 运行时配置

#### macOS

动态库需要打包到 `.app` 中：

```bash
# 复制动态库
cp $SHERPA_LIB_PATH/*.dylib KeVoiceInput.app/Contents/Frameworks/

# 调整库路径
install_name_tool -change \
  @rpath/libsherpa-onnx-c-api.dylib \
  @loader_path/../Frameworks/libsherpa-onnx-c-api.dylib \
  KeVoiceInput.app/Contents/MacOS/kevoiceinput
```

**自动化**: `scripts/post-bundle.sh` 和 `scripts/copy-dylibs.sh` 处理

**开发模式**:
```bash
export DYLD_LIBRARY_PATH=$SHERPA_LIB_PATH:$DYLD_LIBRARY_PATH
bun run tauri:dev
```

#### Linux

```bash
export LD_LIBRARY_PATH=$SHERPA_LIB_PATH:$LD_LIBRARY_PATH
```

#### Windows

将 `.dll` 文件放在可执行文件同目录或 `PATH` 中。

## 在代码中使用

### TranscriptionManager

`managers/transcription.rs` 负责加载模型和转录：

#### 加载 Transducer 模型

```rust
let recognizer = sherpa_rs::OnlineRecognizer::new(
    sherpa_rs::OnlineRecognizerConfig {
        model_config: sherpa_rs::OnlineModelConfig {
            transducer: sherpa_rs::OnlineTransducerModelConfig {
                encoder: encoder_path,
                decoder: decoder_path,
                joiner: joiner_path,
            },
            tokens: tokens_path,
            num_threads: 4,
        },
        decoding_method: "modified_beam_search",
        ..Default::default()
    }
)?;
```

#### 加载 Paraformer 模型

```rust
let recognizer = sherpa_rs::OfflineRecognizer::new(
    sherpa_rs::OfflineRecognizerConfig {
        model_config: sherpa_rs::OfflineModelConfig {
            paraformer: sherpa_rs::OfflineParaformerModelConfig {
                model: model_path,
            },
            tokens: tokens_path,
            num_threads: 4,
        },
        ..Default::default()
    }
)?;
```

#### 加载 SeACo Paraformer 模型

```rust
let recognizer = sherpa_rs::OfflineRecognizer::new(
    sherpa_rs::OfflineRecognizerConfig {
        model_config: sherpa_rs::OfflineModelConfig {
            paraformer: sherpa_rs::OfflineParaformerModelConfig {
                model: model_path,
                model_eb: model_eb_path,  // SeACo 热词嵌入模型
            },
            tokens: tokens_path,
            num_threads: 4,
        },
        hotwords_file: hotwords_file_path,  // 热词列表
        hotwords_score: 1.5,  // 热词权重
        ..Default::default()
    }
)?;
```

### 转录流程

#### 流式转录 (Transducer)

```rust
let stream = recognizer.create_stream()?;
stream.accept_waveform(sample_rate, &audio_samples);
while recognizer.is_ready(&stream) {
    recognizer.decode_stream(&stream);
}
let result = recognizer.get_result(&stream);
```

#### 非流式转录 (Paraformer/SeACo)

```rust
let stream = recognizer.create_stream()?;
stream.accept_waveform(sample_rate, &audio_samples);
recognizer.decode_stream(&stream);
let result = stream.result();
```

## 故障排查

### 常见问题

#### 1. "Cannot find libsherpa-onnx-c-api"

**原因**: 动态库路径未设置

**解决**:
- 检查 `SHERPA_LIB_PATH` 是否设置
- 检查库文件是否存在
- macOS: 检查 `DYLD_LIBRARY_PATH` 或 `.app/Contents/Frameworks/`
- Linux: 检查 `LD_LIBRARY_PATH`

#### 2. "Model file not found"

**原因**: 模型文件路径错误

**解决**:
- 检查模型目录结构
- 确认所有必需文件存在
- 检查文件权限

#### 3. "Segmentation fault" 或崩溃

**原因**: 库版本不兼容或内存问题

**解决**:
- 确保 sherpa-onnx 和 ONNX Runtime 版本兼容
- 检查模型文件是否损坏
- 增加线程数或减少并发

#### 4. SeACo Paraformer 不识别热词

**原因**: `model_eb.onnx` 未正确加载

**解决**:
- 确认 `model_eb.onnx` 文件存在
- 检查 `hotwords_file` 路径正确
- 验证 sherpa-rs 版本支持 `model_eb` 参数

### 调试工具

#### 测试 sherpa-onnx API

```bash
cd src-tauri
cargo run --bin test_sherpa_api
```

#### 检查动态库依赖

```bash
# macOS
otool -L /Applications/KeVoiceInput.app/Contents/MacOS/kevoiceinput

# Linux
ldd ./kevoiceinput

# Windows
dumpbin /dependents kevoiceinput.exe
```

## 性能优化

### 线程数配置

```rust
num_threads: std::thread::available_parallelism()
    .map(|n| n.get())
    .unwrap_or(4)
```

### 采样率转换

sherpa-onnx 通常要求 16kHz 音频：

```rust
// audio_toolkit/audio/recording.rs
let resampler = rubato::FftFixedIn::new(
    original_sample_rate,
    16000,
    chunk_size,
    2,
    num_channels
)?;
```

### 内存管理

- 及时释放未使用的识别器
- 避免长时间保持大量音频数据在内存

## 相关资源

- [sherpa-onnx 文档](https://k2-fsa.github.io/sherpa/onnx/)
- [sherpa-rs 仓库](https://github.com/thewh1teagle/sherpa-rs)
- [k2-fsa 社区](https://github.com/k2-fsa/)
- [ONNX Runtime](https://onnxruntime.ai/)

## 许可证

- sherpa-onnx: Apache 2.0
- sherpa-rs: MIT
- ONNX Runtime: MIT
