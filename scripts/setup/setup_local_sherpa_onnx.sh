#!/bin/bash
# 设置本地 sherpa-onnx 到 vendor 目录

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VENDOR_DIR="$PROJECT_ROOT/vendor"
SHERPA_ONNX_SRC="$HOME/Desktop/open/sherpa-onnx"
# build.rs 需要 sherpa-onnx 在 sherpa-rs-sys 目录下
SHERPA_ONNX_DST="$VENDOR_DIR/sherpa-rs/crates/sherpa-rs-sys/sherpa-onnx"

echo "📦 设置本地 sherpa-onnx 到 vendor 目录"
echo "源目录: $SHERPA_ONNX_SRC"
echo "目标目录: $SHERPA_ONNX_DST"

# 检查源目录是否存在
if [ ! -d "$SHERPA_ONNX_SRC" ]; then
    echo "❌ 错误: 找不到源目录 $SHERPA_ONNX_SRC"
    exit 1
fi

# 检查源目录是否包含必要的文件
if [ ! -f "$SHERPA_ONNX_SRC/sherpa-onnx/c-api/c-api.h" ]; then
    echo "❌ 错误: 源目录不包含 sherpa-onnx/c-api/c-api.h"
    exit 1
fi

# 创建 vendor 目录
mkdir -p "$VENDOR_DIR"

# 检查是否已经存在
if [ -d "$SHERPA_ONNX_DST" ]; then
    echo "⚠️  sherpa-onnx 目录已存在，跳过复制"
    echo "如果要重新复制，请先删除: rm -rf $SHERPA_ONNX_DST"
else
    echo "📥 复制 sherpa-onnx 到 vendor 目录..."
    # 使用 rsync 或 cp -R
    if command -v rsync &> /dev/null; then
        rsync -av --exclude='.git' --exclude='build' "$SHERPA_ONNX_SRC/" "$SHERPA_ONNX_DST/"
    else
        cp -R "$SHERPA_ONNX_SRC" "$SHERPA_ONNX_DST"
        # 删除 .git 目录以节省空间
        rm -rf "$SHERPA_ONNX_DST/.git"
    fi
    echo "✅ sherpa-onnx 已复制到: $SHERPA_ONNX_DST"
fi

# 检查 build 目录
if [ -d "$HOME/Desktop/open/sherpa-onnx/build" ]; then
    echo ""
    echo "ℹ️  检测到 build 目录，建议设置 SHERPA_LIB_PATH:"
    echo "   export SHERPA_LIB_PATH=$HOME/Desktop/open/sherpa-onnx/build"
    echo ""
    echo "或者在编译时设置:"
    echo "   SHERPA_LIB_PATH=$HOME/Desktop/open/sherpa-onnx/build cargo build"
fi

echo ""
echo "✅ 设置完成！"
echo ""
echo "下一步："
echo "1. 设置环境变量（如果使用已编译的库）:"
echo "   export SHERPA_LIB_PATH=$HOME/Desktop/open/sherpa-onnx/build"
echo ""
echo "2. 或者让 sherpa-rs-sys 从源码构建（需要 CMake）:"
echo "   确保 vendor/sherpa-rs/crates/sherpa-rs-sys/sherpa-onnx 存在"
echo ""
echo "3. 编译项目:"
echo "   cd src-tauri && cargo build"
