#!/bin/bash

# 使用 lldb 调试 sherpa-onnx API 测试程序

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TAURI_DIR="$PROJECT_ROOT/src-tauri"

# 默认模型目录
DEFAULT_MODEL_DIR="$HOME/Library/Application Support/com.kevoiceinput.app/models/conformer-zh-stateless2"

# 如果提供了参数，使用它；否则使用默认值
MODEL_DIR="${1:-$DEFAULT_MODEL_DIR}"

echo "=========================================="
echo "使用 lldb 调试 sherpa-onnx API 测试"
echo "=========================================="
echo ""
echo "模型目录: $MODEL_DIR"
echo ""

cd "$TAURI_DIR"

# 先编译程序
echo "编译测试程序..."
cargo build --bin test_sherpa_api
echo ""

# 获取编译后的二进制文件路径
BINARY_PATH="$TAURI_DIR/target/debug/test_sherpa_api"

if [ ! -f "$BINARY_PATH" ]; then
    echo "错误: 找不到编译后的二进制文件: $BINARY_PATH"
    exit 1
fi

# 设置库搜索路径（macOS 需要）
DEPS_DIR="$TAURI_DIR/target/debug/deps"
if [ -d "$DEPS_DIR" ]; then
    export DYLD_LIBRARY_PATH="$DEPS_DIR:$DYLD_LIBRARY_PATH"
    echo "设置库搜索路径: $DEPS_DIR"
    echo ""
fi

echo "使用 lldb 运行: $BINARY_PATH"
echo ""
echo "提示:"
echo "  - 程序运行后如果崩溃，输入 'bt' 查看调用栈"
echo "  - 输入 'run' 开始运行程序"
echo "  - 输入 'quit' 退出 lldb"
echo ""
echo "注意: 库搜索路径已设置为: $DEPS_DIR"
echo ""

# 使用 env 命令设置环境变量并运行 lldb
echo "执行 lldb..."
echo ""
echo "⚠️  重要：在 lldb 中需要手动设置环境变量"
echo "   输入以下命令："
echo "   (lldb) settings set target.env-vars DYLD_LIBRARY_PATH=$DEPS_DIR"
echo "   (lldb) run \"$MODEL_DIR\""
echo ""

# 尝试使用 env 设置环境变量（lldb 可能不会继承，但试试看）
env DYLD_LIBRARY_PATH="$DEPS_DIR:$DYLD_LIBRARY_PATH" lldb "$BINARY_PATH" -- "$MODEL_DIR"
