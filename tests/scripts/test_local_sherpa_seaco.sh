#!/bin/bash

# 测试本地编译的 sherpa-onnx API，使用 SeACo Paraformer 模型
# 用法: ./scripts/test_local_sherpa_seaco.sh

set -e

TAURI_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$TAURI_DIR/src-tauri"

# 本地 sherpa-onnx 构建目录
SHERPA_BUILD_DIR="/Users/thinkre/Desktop/open/sherpa-onnx/build"

# 模型目录
MODEL_DIR="$HOME/Desktop/models/beike/seaco_paraformer.20250904.for_general.sherpa_onnx"

echo "=========================================="
echo "测试本地 sherpa-onnx API"
echo "=========================================="
echo ""
echo "本地构建目录: $SHERPA_BUILD_DIR"
echo "模型目录: $MODEL_DIR"
echo ""

# 检查构建目录是否存在
if [ ! -d "$SHERPA_BUILD_DIR" ]; then
    echo "❌ 错误: 本地构建目录不存在: $SHERPA_BUILD_DIR"
    exit 1
fi

# 检查模型目录是否存在
if [ ! -d "$MODEL_DIR" ]; then
    echo "❌ 错误: 模型目录不存在: $MODEL_DIR"
    exit 1
fi

# 检查必要的文件
echo "检查模型文件..."
if [ ! -f "$MODEL_DIR/model.onnx" ]; then
    echo "❌ 错误: 缺少 model.onnx"
    exit 1
fi
if [ ! -f "$MODEL_DIR/tokens.txt" ]; then
    echo "❌ 错误: 缺少 tokens.txt"
    exit 1
fi
if [ ! -f "$MODEL_DIR/model_eb.onnx" ]; then
    echo "⚠️  警告: 缺少 model_eb.onnx（SeACo Paraformer 需要此文件）"
fi

echo "✅ 模型文件检查通过"
echo ""

# 设置环境变量
export SHERPA_LIB_PATH="$SHERPA_BUILD_DIR"
echo "设置 SHERPA_LIB_PATH=$SHERPA_LIB_PATH"

# 查找 ONNX Runtime 库
# 首先尝试在本地构建目录中查找
ONNX_RUNTIME_LIB=$(find "$SHERPA_BUILD_DIR" -name "libonnxruntime*.dylib" 2>/dev/null | head -1)
if [ -n "$ONNX_RUNTIME_LIB" ]; then
    ONNX_RUNTIME_DIR=$(dirname "$ONNX_RUNTIME_LIB")
    export DYLD_LIBRARY_PATH="$ONNX_RUNTIME_DIR:$DYLD_LIBRARY_PATH"
    echo "设置 DYLD_LIBRARY_PATH=$ONNX_RUNTIME_DIR (从本地构建目录)"
else
    # 尝试从 sherpa-rs 缓存中查找
    CACHE_ONNX_LIB=$(find ~/Library/Caches/sherpa-rs -name "libonnxruntime*.dylib" 2>/dev/null | head -1)
    if [ -n "$CACHE_ONNX_LIB" ]; then
        CACHE_ONNX_DIR=$(dirname "$CACHE_ONNX_LIB")
        export DYLD_LIBRARY_PATH="$CACHE_ONNX_DIR:$DYLD_LIBRARY_PATH"
        echo "设置 DYLD_LIBRARY_PATH=$CACHE_ONNX_DIR (从 sherpa-rs 缓存)"
    else
        echo "⚠️  警告: 未找到 ONNX Runtime 库，可能需要手动设置 DYLD_LIBRARY_PATH"
    fi
fi
echo ""

# 清理并重新构建（使用本地库）
echo "清理之前的构建..."
cargo clean -p sherpa-rs-sys 2>/dev/null || true

echo ""
echo "开始构建测试程序（使用本地 sherpa-onnx）..."
cargo build --bin test_sherpa_api 2>&1 | tail -20

echo ""
echo "=========================================="
echo "运行测试..."
echo "=========================================="
echo ""

# 运行测试
./target/debug/test_sherpa_api "$MODEL_DIR"

echo ""
echo "=========================================="
echo "测试完成"
echo "=========================================="
