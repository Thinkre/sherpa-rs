#!/bin/bash

# 使用 lldb 调试段错误并获取详细信息

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TAURI_DIR="$PROJECT_ROOT/src-tauri"

DEFAULT_MODEL_DIR="$HOME/Library/Application Support/com.kevoiceinput.app/models/conformer-zh-stateless2"
MODEL_DIR="${1:-$DEFAULT_MODEL_DIR}"

cd "$TAURI_DIR"

# 设置库搜索路径
DEPS_DIR="$TAURI_DIR/target/debug/deps"
if [ -d "$DEPS_DIR" ]; then
    export DYLD_LIBRARY_PATH="$DEPS_DIR:$DYLD_LIBRARY_PATH"
fi

BINARY_PATH="$TAURI_DIR/target/debug/test_sherpa_api"

if [ ! -f "$BINARY_PATH" ]; then
    echo "编译测试程序..."
    cargo build --bin test_sherpa_api
fi

echo "=========================================="
echo "使用 lldb 调试段错误"
echo "=========================================="
echo ""
echo "模型目录: $MODEL_DIR"
echo "二进制文件: $BINARY_PATH"
echo ""
echo "lldb 命令提示:"
echo "  1. 环境变量已自动设置"
echo "  2. 输入 'run' 运行程序（如果未自动运行）"
echo "  3. 崩溃后输入 'bt' 查看完整调用栈"
echo "  4. 输入 'frame select <number>' 选择特定帧"
echo "  5. 输入 'print <variable>' 查看变量值"
echo "  6. 输入 'quit' 退出"
echo ""

# 使用 env 命令设置环境变量并运行 lldb
# 注意：lldb 需要手动设置环境变量
echo "执行 lldb..."
echo ""
echo "⚠️  重要：在 lldb 中需要手动设置环境变量"
echo "   输入以下命令："
echo "   (lldb) settings set target.env-vars DYLD_LIBRARY_PATH=$DEPS_DIR"
echo "   (lldb) run \"$MODEL_DIR\""
echo ""

# 尝试使用 env 设置环境变量（lldb 可能不会继承，但试试看）
env DYLD_LIBRARY_PATH="$DEPS_DIR:$DYLD_LIBRARY_PATH" lldb "$BINARY_PATH" -- "$MODEL_DIR"
