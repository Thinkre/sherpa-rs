#!/bin/bash
# SeACo Paraformer 模型设置脚本
# 将导出的 ONNX 模型复制到应用的 models 目录

set -e

MODEL_SOURCE_DIR="${1:-}"
APP_MODELS_DIR=""

# 检测操作系统并设置应用 models 目录
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    APP_MODELS_DIR="$HOME/Library/Application Support/com.kevoiceinput.app/models"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux
    APP_MODELS_DIR="$HOME/.local/share/com.kevoiceinput.app/models"
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    # Windows
    APP_MODELS_DIR="$APPDATA/com.kevoiceinput.app/models"
else
    echo "错误: 不支持的操作系统: $OSTYPE"
    exit 1
fi

# 如果没有提供源目录，使用默认路径
if [ -z "$MODEL_SOURCE_DIR" ]; then
    MODEL_SOURCE_DIR="$HOME/Desktop/models/beike/seaco_paraformer.20250904.for_general/onnx_export"
fi

MODEL_SOURCE_DIR=$(realpath "$MODEL_SOURCE_DIR")
TARGET_DIR="$APP_MODELS_DIR/seaco-paraformer-beike"

echo "SeACo Paraformer 模型设置脚本"
echo "================================"
echo "源目录: $MODEL_SOURCE_DIR"
echo "目标目录: $TARGET_DIR"
echo ""

# 检查源目录是否存在
if [ ! -d "$MODEL_SOURCE_DIR" ]; then
    echo "错误: 源目录不存在: $MODEL_SOURCE_DIR"
    echo ""
    echo "请先导出模型:"
    echo "  1. 安装依赖: pip install -U modelscope funasr"
    echo "  2. 导出模型: python -m funasr.export.export_model \\"
    echo "     --model-name $HOME/Desktop/models/beike/seaco_paraformer.20250904.for_general \\"
    echo "     --export-dir $MODEL_SOURCE_DIR \\"
    echo "     --type onnx"
    exit 1
fi

# 检查必需文件
REQUIRED_FILES=("model.onnx" "tokens.txt")
MISSING_FILES=()

for file in "${REQUIRED_FILES[@]}"; do
    if [ ! -f "$MODEL_SOURCE_DIR/$file" ]; then
        MISSING_FILES+=("$file")
    fi
done

if [ ${#MISSING_FILES[@]} -ne 0 ]; then
    echo "错误: 缺少必需文件:"
    for file in "${MISSING_FILES[@]}"; do
        echo "  - $file"
    done
    echo ""
    echo "请确保导出目录包含以下文件:"
    echo "  - model.onnx"
    echo "  - tokens.txt"
    exit 1
fi

# 创建目标目录
echo "创建目标目录..."
mkdir -p "$TARGET_DIR"

# 复制文件
echo "复制模型文件..."
cp "$MODEL_SOURCE_DIR/model.onnx" "$TARGET_DIR/"
cp "$MODEL_SOURCE_DIR/tokens.txt" "$TARGET_DIR/"

# 复制可选文件
if [ -f "$MODEL_SOURCE_DIR/am.mvn" ]; then
    cp "$MODEL_SOURCE_DIR/am.mvn" "$TARGET_DIR/"
    echo "  ✓ 复制 am.mvn"
fi

if [ -f "$MODEL_SOURCE_DIR/config.yaml" ]; then
    cp "$MODEL_SOURCE_DIR/config.yaml" "$TARGET_DIR/"
    echo "  ✓ 复制 config.yaml"
fi

echo ""
echo "✓ 模型文件已复制到: $TARGET_DIR"
echo ""
echo "文件列表:"
ls -lh "$TARGET_DIR" | grep -E "model.onnx|tokens.txt|am.mvn|config.yaml" || true
echo ""
echo "下一步:"
echo "  1. 启动 KeVoiceInput 应用"
echo "  2. 在模型选择页面选择 'SeACo Paraformer 贝壳'"
echo "  3. 开始使用语音识别功能"
echo ""
echo "注意: SeACo Paraformer 支持热词功能，可以在设置中添加热词以提高识别准确率。"
