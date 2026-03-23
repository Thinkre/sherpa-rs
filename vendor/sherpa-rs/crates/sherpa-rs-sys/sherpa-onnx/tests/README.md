# 测试脚本说明

本目录包含 SeACo-Paraformer 相关的测试脚本。

## 测试脚本列表

### 1. test_seaco_hotwords.sh
测试 SeACo-Paraformer 的热词功能。

**使用方法**:
```bash
cd tests
./test_seaco_hotwords.sh
```

**说明**:
- 从 `tests/` 目录运行
- 使用 `../build/bin/sherpa-onnx-offline` 命令行工具
- 创建临时热词文件进行测试

### 2. test_paraformer.sh
测试标准 Paraformer 和 SeACo-Paraformer 模型。

**使用方法**:
```bash
cd tests
./test_paraformer.sh
```

**说明**:
- 从 `tests/` 目录运行
- 测试多个模型配置

### 3. test_seaco_c_api.sh
测试 SeACo-Paraformer 的 C-API 调用。

**使用方法**:
```bash
cd tests
./test_seaco_c_api.sh <wav_file> [hotwords_file]
```

**示例**:
```bash
# 无热词测试
./test_seaco_c_api.sh ../models/sherpa-onnx-paraformer-zh-int8-2025-10-07/test_wavs/16.wav

# 有热词测试
./test_seaco_c_api.sh ../models/sherpa-onnx-paraformer-zh-int8-2025-10-07/test_wavs/16.wav hotwords.txt
```

**说明**:
- 从 `tests/` 目录运行
- 自动编译 C-API 测试程序（`../local/test_seaco_paraformer_c_api.c`）
- 使用 `../build/bin/test_seaco_paraformer_c_api` 执行测试

### 4. compare_onnx_metadata.sh
比较两个 ONNX 模型的元数据（metadata）。

**使用方法**:
```bash
cd tests
./compare_onnx_metadata.sh <model1.onnx> <model2.onnx> [tokens1.txt] [tokens2.txt]
```

**示例**:
```bash
./compare_onnx_metadata.sh \
    model1.onnx \
    model2.onnx \
    tokens1.txt \
    tokens2.txt
```

**说明**:
- 从 `tests/` 目录运行
- 使用 `../build/bin/sherpa-onnx-offline` 提取模型元数据
- 比较 vocab_size、LFR 参数、CMVN 参数等

### 5. compare_onnx_structures.sh
全面比较两个 ONNX 模型的结构。

**使用方法**:
```bash
cd tests
./compare_onnx_structures.sh <model1.onnx> <model2.onnx> [tokens1.txt] [tokens2.txt]
```

**示例**:
```bash
./compare_onnx_structures.sh \
    model1.onnx \
    model2.onnx \
    tokens1.txt \
    tokens2.txt
```

**说明**:
- 从 `tests/` 目录运行
- 比较模型的输入输出、元数据、图结构等
- 生成详细的比较报告

## 目录结构

```
sherpa-onnx/
├── tests/                    # 测试脚本目录（本目录）
│   ├── README.md
│   ├── test_seaco_hotwords.sh
│   ├── test_paraformer.sh
│   ├── test_seaco_c_api.sh
│   ├── compare_onnx_metadata.sh
│   └── compare_onnx_structures.sh
├── local/                    # 本地开发文件目录
│   ├── test_seaco_paraformer_c_api.c  # C-API 测试程序源码
│   ├── compare_onnx_models.py         # 工具脚本
│   └── ...                   # 其他本地文件
└── build/                    # 构建目录（不管理）
    └── bin/
        └── sherpa-onnx-offline
```

## 注意事项

- **所有脚本需要从 `tests/` 目录运行**
- 确保已经编译了 sherpa-onnx（`../build/bin/sherpa-onnx-offline` 存在）
- 模型文件路径在脚本中硬编码，可能需要根据实际情况修改
- C-API 测试程序会自动编译，编译输出在 `../build/bin/` 目录

## 路径说明

所有脚本中的路径都是相对于 `tests/` 目录的：
- `../build/` - 构建目录
- `../local/` - 本地文件目录
- `../sherpa-onnx/` - 源代码目录
