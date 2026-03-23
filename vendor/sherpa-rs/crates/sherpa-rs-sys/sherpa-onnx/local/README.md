# Local 文件说明

本目录包含本地开发和测试使用的文件。

## 文件列表

### C-API 测试程序

#### test_seaco_c_api.c
SeACo-Paraformer C-API 测试程序的源代码（推荐使用）。

**编译方法**:
使用 CMake（推荐）:
```bash
cd build
cmake --build . --target test-seaco-c-api
```

编译后的可执行文件位于 `build/bin/test-seaco-c-api`。

**使用方法**:
```bash
./build/bin/test-seaco-c-api <wav_file> [hotwords_file]
```

或者手动编译：
```bash
gcc -o ../build/test_seaco_paraformer_c_api ../local/test_seaco_paraformer_c_api.c \
    -I../sherpa-onnx/c-api \
    -L../build/lib \
    -lsherpa-onnx-c-api \
    -Wl,-rpath,../build/lib \
    -std=c99 -Wall -Wextra
```

#### test_seaco_paraformer_c_api.c
SeACo-Paraformer C-API 测试程序的另一个版本（已废弃，使用 test_seaco_c_api.c）。

### 工具脚本

#### compare_onnx_models.py
比较两个 ONNX 模型结构的 Python 工具。

**使用方法**:
```bash
python3 local/compare_onnx_models.py \
    --model1 model1.onnx \
    --model2 model2.onnx
```

#### compare_onnx_simple.py
使用 onnxruntime 简单比较 ONNX 模型的工具。

**使用方法**:
```bash
python3 local/compare_onnx_simple.py model1.onnx model2.onnx
```

#### convert_tokens_json_to_txt.py
将 FunASR 格式的 tokens.json 转换为 sherpa-onnx 格式的 tokens.txt。

**使用方法**:
```bash
python3 local/convert_tokens_json_to_txt.py \
    --input tokens.json
```

#### inspect_onnx_model.py
检查 ONNX 模型结构的工具。

**使用方法**:
```bash
python3 local/inspect_onnx_model.py model.onnx
```

#### inspect_onnx.cc
检查 ONNX 模型的 C++ 程序源码。

### 构建脚本

#### build.sh
快速构建脚本。

**使用方法**:
```bash
./local/build.sh
```

### 文档文件

#### seaco_paraformer_comparison.md
SeACo-Paraformer 实现对比文档（sherpa-onnx vs FunASR）。

#### seaco_paraformer_implementation_analysis.md
SeACo-Paraformer 实现流程详细分析文档。

#### seaco_paraformer_api_status.md
SeACo-Paraformer API 支持状态文档。

#### seaco_paraformer_code_locations.md
SeACo-Paraformer 实现代码位置文档。

#### model_comparison_summary.md
ONNX 模型结构比较报告。

## 注意事项

- 这些文件是本地开发使用的，不会被提交到版本控制系统
- 如果需要添加到 CMake 构建系统，请修改 `c-api-examples/CMakeLists.txt`
- 文档文件仅供参考，实际实现请参考 `scripts/` 目录下的官方文档
