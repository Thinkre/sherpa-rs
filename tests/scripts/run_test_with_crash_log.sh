#!/bin/bash

# 运行测试并捕获崩溃信息

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TAURI_DIR="$PROJECT_ROOT/src-tauri"

# 默认模型目录
DEFAULT_MODEL_DIR="$HOME/Library/Application Support/com.kevoiceinput.app/models/conformer-zh-stateless2"

# 如果提供了参数，使用它；否则使用默认值
MODEL_DIR="${1:-$DEFAULT_MODEL_DIR}"

cd "$TAURI_DIR"

echo "=========================================="
echo "运行测试并捕获崩溃信息"
echo "=========================================="
echo ""
echo "模型目录: $MODEL_DIR"
echo ""

# 设置库搜索路径（macOS 需要）
DEPS_DIR="$TAURI_DIR/target/debug/deps"
if [ -d "$DEPS_DIR" ]; then
    export DYLD_LIBRARY_PATH="$DEPS_DIR:$DYLD_LIBRARY_PATH"
    echo "设置库搜索路径: $DEPS_DIR"
    echo ""
fi

# 设置环境变量以捕获更多信息
export RUST_BACKTRACE=full
export RUST_LIB_BACKTRACE=1

# 运行测试
echo "运行测试程序..."
echo ""

# 使用 ulimit 来捕获核心转储（如果系统支持）
ulimit -c unlimited 2>/dev/null || true

# 运行程序，如果崩溃会显示 backtrace
cargo run --bin test_sherpa_api -- "$MODEL_DIR" 2>&1 | tee /tmp/sherpa_test_output.log

EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
    echo ""
    echo "=========================================="
    echo "程序退出，退出码: $EXIT_CODE"
    echo "=========================================="
    echo ""
    echo "如果程序崩溃，可以使用以下命令查看详细信息:"
    echo "  lldb target/debug/test_sherpa_api"
    echo "  然后在 lldb 中输入: run \"$MODEL_DIR\""
    echo "  崩溃后输入: bt"
    echo ""
    echo "或者查看日志:"
    echo "  cat /tmp/sherpa_test_output.log"
fi

exit $EXIT_CODE
