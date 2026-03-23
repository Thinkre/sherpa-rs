#!/bin/bash

# 使用 sherpa-onnx 命令行工具测试模型（如果可用）

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

MODEL_DIR="${1:-$HOME/Library/Application Support/com.kevoiceinput.app/models/conformer-zh-stateless2}"

echo "=========================================="
echo "使用命令行工具测试模型"
echo "=========================================="
echo ""
echo "模型目录: $MODEL_DIR"
echo ""

# 检查命令行工具是否存在
if ! command -v sherpa-onnx-offline &> /dev/null; then
    echo "⚠️  sherpa-onnx-offline 命令行工具未找到"
    echo ""
    echo "要安装命令行工具，请："
    echo "1. 从 https://github.com/k2-fsa/sherpa-onnx/releases 下载"
    echo "2. 或编译 sherpa-onnx 并安装"
    echo ""
    exit 1
fi

# 检查模型文件
ENCODER="$MODEL_DIR/encoder-epoch-99-avg-1.onnx"
DECODER="$MODEL_DIR/decoder-epoch-99-avg-1.onnx"
JOINER="$MODEL_DIR/joiner-epoch-99-avg-1.onnx"
TOKENS="$MODEL_DIR/tokens.txt"

echo "检查模型文件..."
for file in "$ENCODER" "$DECODER" "$JOINER" "$TOKENS"; do
    if [ -f "$file" ]; then
        echo "✅ $(basename "$file")"
    else
        echo "❌ $(basename "$file") 不存在"
        exit 1
    fi
done

echo ""
echo "测试模型配置..."
echo ""

# 尝试创建一个简单的测试（如果可能）
# 注意：这需要音频文件，所以我们只测试配置是否正确

echo "如果模型配置正确，命令行工具应该能够识别模型文件。"
echo "要完整测试，需要提供音频文件："
echo ""
echo "  sherpa-onnx-offline \\"
echo "    --tokens=\"$TOKENS\" \\"
echo "    --encoder=\"$ENCODER\" \\"
echo "    --decoder=\"$DECODER\" \\"
echo "    --joiner=\"$JOINER\" \\"
echo "    --num-threads=1 \\"
echo "    --decoding-method=modified_beam_search \\"
echo "    <audio_file.wav>"
echo ""
