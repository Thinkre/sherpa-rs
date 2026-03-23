# ONNX 模型结构比较报告

## 模型信息

- **模型1**: `sherpa-onnx-paraformer-zh-int8-2025-10-07/model.int8.onnx`
- **模型2**: `ke_paraformer_0902_onnx/model.onnx`

## 主要差异总结

### 1. 元数据（Metadata）差异

#### 模型1（sherpa-onnx-paraformer-zh-int8-2025-10-07）
- ✅ **包含完整的元数据**：
  - `vocab_size=8404`
  - `lfr_window_size=7`
  - `lfr_window_shift=6`
  - `neg_mean` (CMVN参数，560维)
  - `inv_stddev` (CMVN参数，560维)
  - `model_type`
  - `model_author`
  - `comment`

#### 模型2（ke_paraformer_0902_onnx）
- ❌ **完全没有元数据**
  - 所有元数据字段都缺失
  - CMVN参数存储在外部文件 `am.mvn` 中

### 2. CMVN参数存储位置

| 参数 | 模型1 | 模型2 |
|------|-------|-------|
| `neg_mean` | ONNX元数据中 | `am.mvn` 文件中 |
| `inv_stddev` | ONNX元数据中 | `am.mvn` 文件中 |

### 3. 其他参数

| 参数 | 模型1 | 模型2 |
|------|-------|-------|
| `vocab_size` | ONNX元数据中 (8404) | 从 tokens.txt 读取 (8404) |
| `lfr_window_size` | ONNX元数据中 (7) | 使用默认值 (7) |
| `lfr_window_shift` | ONNX元数据中 (6) | 使用默认值 (6) |

### 4. 文件结构

**模型1目录结构**:
```
sherpa-onnx-paraformer-zh-int8-2025-10-07/
├── model.int8.onnx  (包含完整元数据)
└── tokens.txt
```

**模型2目录结构**:
```
ke_paraformer_0902_onnx/
├── model.onnx  (无元数据)
├── am.mvn  (包含CMVN参数)
└── tokens.txt
```

## 兼容性处理

代码已添加以下兼容性处理：

1. **vocab_size**: 如果元数据中没有，从 tokens.txt 读取
2. **lfr_window_size**: 如果元数据中没有，使用默认值 7
3. **lfr_window_shift**: 如果元数据中没有，使用默认值 6
4. **neg_mean 和 inv_stddev**: 
   - 优先从 ONNX 元数据读取
   - 如果元数据中没有，从 `am.mvn` 文件读取
   - 如果都找不到，报错

## 结论

两个模型的主要差异在于：
- **模型1**: 所有参数都嵌入在 ONNX 文件的元数据中（自包含）
- **模型2**: 参数分散存储（ONNX文件 + am.mvn文件）

这种差异反映了不同的模型导出策略：
- 模型1采用了将参数嵌入元数据的方式，便于部署
- 模型2采用了传统的 FunASR 格式，参数存储在外部文件中

代码已兼容两种格式，可以自动处理。
