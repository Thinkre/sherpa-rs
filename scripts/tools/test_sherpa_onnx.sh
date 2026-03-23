#!/bin/bash

# 测试sherpa-onnx是否能正确识别SeACo-Paraformer模型
# 这将验证特征提取是否是问题所在

set -e

# 配置（请根据实际路径修改）
SHERPA_BIN="/Users/thinkre/Desktop/open/sherpa-onnx/build/bin/sherpa-onnx"
MODEL_DIR="<MODEL_PATH>"  # 替换为你的SeACo-Paraformer模型路径
TEST_AUDIO="<AUDIO_PATH>" # 替换为测试音频路径

# 检查文件是否存在
if [ ! -f "$SHERPA_BIN" ]; then
    echo "错误: sherpa-onnx binary not found at $SHERPA_BIN"
    exit 1
fi

if [ ! -d "$MODEL_DIR" ]; then
    echo "错误: Model directory not found at $MODEL_DIR"
    echo "请修改脚本中的MODEL_DIR变量"
    exit 1
fi

if [ ! -f "$TEST_AUDIO" ]; then
    echo "错误: Test audio not found at $TEST_AUDIO"
    echo "请修改脚本中的TEST_AUDIO变量"
    exit 1
fi

echo "================================"
echo "Testing SeACo-Paraformer with sherpa-onnx"
echo "================================"
echo "Model dir: $MODEL_DIR"
echo "Audio: $TEST_AUDIO"
echo ""

# 运行sherpa-onnx
echo "Running sherpa-onnx..."
"$SHERPA_BIN" \
  --tokens="$MODEL_DIR/tokens.txt" \
  --paraformer-encoder="$MODEL_DIR/model.onnx" \
  --num-threads=2 \
  --decoding-method=greedy_search \
  --debug=1 \
  "$TEST_AUDIO"

echo ""
echo "================================"
echo "如果上面能正确识别，说明："
echo "1. 模型文件没问题"
echo "2. sherpa-onnx的特征提取方式是正确的"
echo "3. Rust实现的特征提取需要改为sherpa-onnx的方式"
echo "================================"
