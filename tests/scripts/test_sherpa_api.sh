#!/bin/bash

# 测试 sherpa-onnx API 的便捷脚本

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TAURI_DIR="$PROJECT_ROOT/src-tauri"

# 默认模型目录（如果存在）
DEFAULT_MODEL_DIR="$HOME/Library/Application Support/com.kevoiceinput.app/models/conformer-zh-stateless2"

# 如果提供了参数，使用它；否则使用默认值
MODEL_DIR="${1:-$DEFAULT_MODEL_DIR}"

echo "=========================================="
echo "测试 sherpa-onnx API"
echo "=========================================="
echo ""
echo "模型目录: $MODEL_DIR"
echo ""

cd "$TAURI_DIR"

# 检查模型目录是否存在
if [ ! -d "$MODEL_DIR" ]; then
    echo "⚠️  警告: 模型目录不存在: $MODEL_DIR"
    echo "将只运行基本测试（不加载模型）"
    echo ""
    MODEL_DIR="/tmp"
fi

# 设置库搜索路径（macOS 需要）
# sherpa-rs-sys 下载的库在 target/debug/deps 目录
DEPS_DIR="$TAURI_DIR/target/debug/deps"
if [ -d "$DEPS_DIR" ]; then
    export DYLD_LIBRARY_PATH="$DEPS_DIR:$DYLD_LIBRARY_PATH"
    echo "✅ 设置库搜索路径: $DEPS_DIR"
    echo ""
else
    echo "⚠️  警告: 库目录不存在: $DEPS_DIR"
    echo "   请先运行: cd src-tauri && cargo build --bin test_sherpa_api"
    echo ""
fi

# 运行测试
echo "运行测试程序..."
echo ""

cargo run --bin test_sherpa_api -- "$MODEL_DIR"

echo ""
echo "=========================================="
echo "测试完成"
echo "=========================================="
