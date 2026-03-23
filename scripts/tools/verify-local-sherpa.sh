#!/bin/bash
# 验证本地 sherpa-onnx 配置

set -e

SHERPA_BUILD_DIR="/Users/thinkre/Desktop/open/sherpa-onnx/build"
PROJECT_DIR="/Users/thinkre/Desktop/projects/KeVoiceInput"

echo "=========================================="
echo "验证本地 Sherpa-ONNX 配置"
echo "=========================================="
echo ""

# 检查源码目录
if [ ! -d "/Users/thinkre/Desktop/open/sherpa-onnx" ]; then
    echo "❌ 错误: Sherpa-ONNX 源码目录不存在"
    exit 1
fi
echo "✅ 源码目录存在: /Users/thinkre/Desktop/open/sherpa-onnx"

# 检查构建目录
if [ ! -d "$SHERPA_BUILD_DIR" ]; then
    echo "❌ 错误: 构建目录不存在: $SHERPA_BUILD_DIR"
    echo "   请先运行: ./scripts/build-with-local-sherpa.sh build"
    exit 1
fi
echo "✅ 构建目录存在: $SHERPA_BUILD_DIR"

# 检查库文件
STATIC_LIB="$SHERPA_BUILD_DIR/lib/libsherpa-onnx-c-api.a"
DYNAMIC_LIB="$SHERPA_BUILD_DIR/lib/libsherpa-onnx-c-api.dylib"

if [ -f "$STATIC_LIB" ]; then
    echo "✅ 找到静态库: $STATIC_LIB"
    echo "   库大小: $(ls -lh "$STATIC_LIB" | awk '{print $5}')"
    LIB_TYPE="static"
elif [ -f "$DYNAMIC_LIB" ]; then
    echo "✅ 找到动态库: $DYNAMIC_LIB"
    echo "   库大小: $(ls -lh "$DYNAMIC_LIB" | awk '{print $5}')"
    LIB_TYPE="dynamic"
else
    echo "❌ 错误: 找不到库文件"
    echo "   请先编译: cd $SHERPA_BUILD_DIR && make"
    exit 1
fi

# 检查 Cargo.toml
cd "$PROJECT_DIR"
# 检查是否在依赖声明中包含 download-binaries（排除注释）
if grep -E "^sherpa-rs.*download-binaries|features.*=.*\[.*download-binaries" src-tauri/Cargo.toml 2>/dev/null | grep -v "^#" > /dev/null; then
    echo "⚠️  警告: Cargo.toml 中仍包含 download-binaries feature"
    echo "   建议移除以使用本地源码"
else
    echo "✅ Cargo.toml 配置正确（已移除 download-binaries feature）"
fi

# 检查环境变量设置
echo ""
echo "环境变量配置："
echo "  SHERPA_LIB_PATH=${SHERPA_LIB_PATH:-未设置}"
if [ -z "$SHERPA_LIB_PATH" ]; then
    echo "  ⚠️  提示: 构建时需要设置 SHERPA_LIB_PATH=$SHERPA_BUILD_DIR"
fi

echo ""
echo "=========================================="
echo "配置验证完成"
echo "=========================================="
echo ""
echo "库类型: $LIB_TYPE"
if [ "$LIB_TYPE" == "static" ]; then
    echo "✅ 使用静态库，APP 将完全自包含"
    echo "✅ 可以直接拷贝到其他电脑运行"
else
    echo "⚠️  使用动态库，生产构建时需要打包库文件"
fi
echo ""
echo "下一步："
echo "  开发模式: ./scripts/build-with-local-sherpa.sh dev"
echo "  生产构建: ./scripts/build-with-local-sherpa.sh build"
