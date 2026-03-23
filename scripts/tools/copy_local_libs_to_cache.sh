#!/bin/bash

# 将本地编译的 sherpa-onnx 库复制到 download-binaries 使用的缓存目录
# 这样就不需要设置 SHERPA_LIB_PATH，库会自动被找到

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 本地构建目录
LOCAL_BUILD_DIR="/Users/thinkre/Desktop/open/sherpa-onnx/build"

# 缓存目录（download-binaries 使用的）
CACHE_BASE="$HOME/Library/Caches/sherpa-rs"

# 自动检测缓存目录
EXISTING_CACHE=$(find "$CACHE_BASE" -type d -name "sherpa-onnx-v*" 2>/dev/null | head -1)
if [ -n "$EXISTING_CACHE" ]; then
    CACHE_DIR="$EXISTING_CACHE"
    echo "✅ 找到现有缓存目录: $CACHE_DIR"
else
    # 如果不存在，使用默认路径
    TARGET="aarch64-apple-darwin"
    HASH="e3f3596b84c65eef8939a8a45ced3d7c4ed857455ba5ae1148ce7a1c93ff9a3c"
    VERSION_DIR="sherpa-onnx-v1.12.9-osx-universal2-shared"
    CACHE_DIR="$CACHE_BASE/$TARGET/$HASH/$VERSION_DIR"
    echo "⚠️  使用默认缓存路径: $CACHE_DIR"
fi

echo "=========================================="
echo "复制本地库到 download-binaries 缓存目录"
echo "=========================================="
echo ""
echo "本地构建目录: $LOCAL_BUILD_DIR"
echo "缓存目录: $CACHE_DIR"
echo ""

# 检查本地构建目录
if [ ! -d "$LOCAL_BUILD_DIR" ]; then
    echo "❌ 错误: 本地构建目录不存在: $LOCAL_BUILD_DIR"
    exit 1
fi

# 检查缓存目录是否存在
if [ ! -d "$CACHE_DIR" ]; then
    echo "⚠️  警告: 缓存目录不存在，正在创建..."
    mkdir -p "$CACHE_DIR/lib"
    mkdir -p "$CACHE_DIR/include"
    echo "✅ 缓存目录已创建"
fi

# 检查本地是否有动态库
LOCAL_LIB_DIR="$LOCAL_BUILD_DIR/lib"
HAS_DYNAMIC_LIBS=false

if [ -d "$LOCAL_LIB_DIR" ]; then
    DYNAMIC_LIBS=$(find "$LOCAL_LIB_DIR" -name "*.dylib" -o -name "*.so" 2>/dev/null | head -5)
    if [ -n "$DYNAMIC_LIBS" ]; then
        HAS_DYNAMIC_LIBS=true
        echo "✅ 找到动态库文件"
    fi
fi

if [ "$HAS_DYNAMIC_LIBS" = false ]; then
    echo "❌ 错误: 本地构建目录中没有找到动态库文件（.dylib 或 .so）"
    echo ""
    echo "本地构建只有静态库（.a），但需要动态库才能工作。"
    echo ""
    echo "解决方案："
    echo ""
    echo "方案 1: 重新编译本地 sherpa-onnx 为动态库（推荐）"
    echo "   cd /Users/thinkre/Desktop/open/sherpa-onnx"
    echo "   rm -rf build && mkdir build && cd build"
    echo "   cmake -DCMAKE_BUILD_TYPE=Release -DBUILD_SHARED_LIBS=ON \\"
    echo "         -DSHERPA_ONNX_ENABLE_C_API=ON \\"
    echo "         -DSHERPA_ONNX_ENABLE_BINARY=OFF .."
    echo "   make -j8"
    echo ""
    echo "   然后重新运行此脚本："
    echo "   ./scripts/copy_local_libs_to_cache.sh"
    echo ""
    echo "方案 2: 使用官方预编译库（取消设置 SHERPA_LIB_PATH）"
    echo "   unset SHERPA_LIB_PATH"
    echo "   bun run tauri dev"
    echo ""
    exit 1
fi

echo ""
echo "开始复制库文件..."
echo ""

# 复制动态库
echo "复制动态库文件..."
CACHE_LIB_DIR="$CACHE_DIR/lib"
mkdir -p "$CACHE_LIB_DIR"

# 复制所有 .dylib 文件
DYNAMIC_COUNT=0
for lib in "$LOCAL_LIB_DIR"/*.dylib; do
    if [ -f "$lib" ]; then
        lib_name=$(basename "$lib")
        cp -v "$lib" "$CACHE_LIB_DIR/$lib_name"
        DYNAMIC_COUNT=$((DYNAMIC_COUNT + 1))
    fi
done

if [ $DYNAMIC_COUNT -eq 0 ]; then
    echo "❌ 错误: 没有找到 .dylib 文件"
    exit 1
fi

echo "✅ 已复制 $DYNAMIC_COUNT 个动态库文件"
echo ""

# 复制头文件（如果需要）
if [ -d "$LOCAL_BUILD_DIR/include" ]; then
    echo "复制头文件..."
    CACHE_INCLUDE_DIR="$CACHE_DIR/include"
    mkdir -p "$CACHE_INCLUDE_DIR"
    cp -r "$LOCAL_BUILD_DIR/include/"* "$CACHE_INCLUDE_DIR/" 2>/dev/null || true
    echo "✅ 头文件已复制"
    echo ""
fi

# 检查 ONNX Runtime 库
echo "检查 ONNX Runtime 库..."
ONNX_LIB=$(find "$LOCAL_LIB_DIR" -name "libonnxruntime*.dylib" 2>/dev/null | head -1)
if [ -n "$ONNX_LIB" ]; then
    onnx_name=$(basename "$ONNX_LIB")
    cp -v "$ONNX_LIB" "$CACHE_LIB_DIR/$onnx_name"
    echo "✅ ONNX Runtime 库已复制: $onnx_name"
else
    echo "⚠️  警告: 未找到 ONNX Runtime 库，可能需要从其他地方复制"
    echo "   检查现有的缓存目录..."
    EXISTING_ONNX=$(find "$CACHE_LIB_DIR" -name "libonnxruntime*.dylib" 2>/dev/null | head -1)
    if [ -n "$EXISTING_ONNX" ]; then
        echo "✅ 缓存目录中已有 ONNX Runtime 库"
    else
        echo "⚠️  警告: 缓存目录中也没有 ONNX Runtime 库"
    fi
fi

echo ""
echo "=========================================="
echo "复制完成"
echo "=========================================="
echo ""
echo "已复制的库文件："
ls -lh "$CACHE_LIB_DIR"/*.dylib 2>/dev/null | awk '{print "  " $9 " (" $5 ")"}'
echo ""
echo "现在可以："
echo "1. 取消设置 SHERPA_LIB_PATH（如果已设置）"
echo "2. 重新构建项目：cd src-tauri && cargo clean -p sherpa-rs-sys && cargo build"
echo "3. 运行应用：bun run tauri dev"
echo ""
echo "库文件会自动从缓存目录加载，就像使用 download-binaries 一样。"
