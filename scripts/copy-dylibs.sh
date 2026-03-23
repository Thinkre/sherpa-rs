#!/bin/bash

# 复制动态库到 macOS app bundle
APP_BUNDLE="$1"

if [ -z "$APP_BUNDLE" ]; then
    echo "Usage: $0 <path-to-app-bundle>"
    exit 1
fi

# 创建 Frameworks 目录
FRAMEWORKS_DIR="$APP_BUNDLE/Contents/Frameworks"
mkdir -p "$FRAMEWORKS_DIR"

# 复制所需的动态库
DYLIBS=(
    "libcargs.dylib"
    "libonnxruntime.1.23.2.dylib"
    "libsherpa-onnx-c-api.dylib"
    "libsherpa-onnx-cxx-api.dylib"
)

# 查找 dylib：先 target/release/release，再 target/release（与 upgrade-onnxruntime 等脚本一致）
RELEASE_DIR="$(cd "$(dirname "$APP_BUNDLE")/../.." && pwd)/release"
TARGET_RELEASE="$(cd "$(dirname "$APP_BUNDLE")/../.." && pwd)"

echo "Copying dynamic libraries to $FRAMEWORKS_DIR..."

for dylib in "${DYLIBS[@]}"; do
    SRC=""
    if [ -f "$RELEASE_DIR/$dylib" ]; then
        SRC="$RELEASE_DIR/$dylib"
    elif [ -f "$TARGET_RELEASE/$dylib" ]; then
        SRC="$TARGET_RELEASE/$dylib"
    fi
    if [ -n "$SRC" ]; then
        echo "  - Copying $dylib"
        cp "$SRC" "$FRAMEWORKS_DIR/"
    else
        echo "  - Warning: $dylib not found in $RELEASE_DIR or $TARGET_RELEASE"
    fi
done

# 至少需要 onnxruntime 和 sherpa 才能启动，否则直接报错
if [ ! -f "$FRAMEWORKS_DIR/libonnxruntime.1.23.2.dylib" ] || [ ! -f "$FRAMEWORKS_DIR/libsherpa-onnx-c-api.dylib" ]; then
    echo ""
    echo "❌ 缺少关键动态库，应用将无法启动。"
    echo "   请确保 dylib 在以下任一目录："
    echo "   - $RELEASE_DIR"
    echo "   - $TARGET_RELEASE"
    echo "   若使用 sherpa-onnx 构建，请设置 SHERPA_LIB_PATH 或把生成的 dylib 复制到 target/release/"
    exit 1
fi

# 修改二进制文件的 rpath
BINARY="$APP_BUNDLE/Contents/MacOS/kevoiceinput"
echo "Updating rpath in $BINARY..."

# 主程序可能链接的是 1.17.1（cargo 用 target/release 下的符号链接），需改为 1.23.2
install_name_tool -change "@rpath/libonnxruntime.1.17.1.dylib" "@executable_path/../Frameworks/libonnxruntime.1.23.2.dylib" "$BINARY" 2>/dev/null || true
# 为每个动态库更新安装名称
for dylib in "${DYLIBS[@]}"; do
    if [ -f "$FRAMEWORKS_DIR/$dylib" ]; then
        install_name_tool -change "@rpath/$dylib" "@executable_path/../Frameworks/$dylib" "$BINARY" 2>/dev/null || true
    fi
done

# 修复动态库之间的依赖关系
echo "Fixing library dependencies..."

# libsherpa-onnx-c-api.dylib 依赖
if [ -f "$FRAMEWORKS_DIR/libsherpa-onnx-c-api.dylib" ]; then
    echo "  - Fixing libsherpa-onnx-c-api.dylib"
    # 修复 onnxruntime：sherpa 可能链接的是 1.17.1 或 1.23.2，统一改为实际存在的 1.23.2
    install_name_tool -change "@rpath/libonnxruntime.1.17.1.dylib" "@loader_path/libonnxruntime.1.23.2.dylib" "$FRAMEWORKS_DIR/libsherpa-onnx-c-api.dylib" 2>/dev/null || true
    install_name_tool -change "@loader_path/libonnxruntime.1.17.1.dylib" "@loader_path/libonnxruntime.1.23.2.dylib" "$FRAMEWORKS_DIR/libsherpa-onnx-c-api.dylib" 2>/dev/null || true
    install_name_tool -change "@rpath/libonnxruntime.1.23.2.dylib" "@loader_path/libonnxruntime.1.23.2.dylib" "$FRAMEWORKS_DIR/libsherpa-onnx-c-api.dylib" 2>/dev/null || true
    install_name_tool -id "@loader_path/libsherpa-onnx-c-api.dylib" "$FRAMEWORKS_DIR/libsherpa-onnx-c-api.dylib" 2>/dev/null || true
fi

# libsherpa-onnx-cxx-api.dylib 依赖
if [ -f "$FRAMEWORKS_DIR/libsherpa-onnx-cxx-api.dylib" ]; then
    echo "  - Fixing libsherpa-onnx-cxx-api.dylib"
    # 修复 onnxruntime：同上，兼容 1.17.1 与 1.23.2 链接名
    install_name_tool -change "@rpath/libonnxruntime.1.17.1.dylib" "@loader_path/libonnxruntime.1.23.2.dylib" "$FRAMEWORKS_DIR/libsherpa-onnx-cxx-api.dylib" 2>/dev/null || true
    install_name_tool -change "@loader_path/libonnxruntime.1.17.1.dylib" "@loader_path/libonnxruntime.1.23.2.dylib" "$FRAMEWORKS_DIR/libsherpa-onnx-cxx-api.dylib" 2>/dev/null || true
    install_name_tool -change "@rpath/libonnxruntime.1.23.2.dylib" "@loader_path/libonnxruntime.1.23.2.dylib" "$FRAMEWORKS_DIR/libsherpa-onnx-cxx-api.dylib" 2>/dev/null || true
    install_name_tool -change "@rpath/libsherpa-onnx-c-api.dylib" "@loader_path/libsherpa-onnx-c-api.dylib" "$FRAMEWORKS_DIR/libsherpa-onnx-cxx-api.dylib" 2>/dev/null || true
    install_name_tool -id "@loader_path/libsherpa-onnx-cxx-api.dylib" "$FRAMEWORKS_DIR/libsherpa-onnx-cxx-api.dylib" 2>/dev/null || true
fi

# libonnxruntime 设置 ID
if [ -f "$FRAMEWORKS_DIR/libonnxruntime.1.23.2.dylib" ]; then
    echo "  - Fixing libonnxruntime.1.23.2.dylib"
    install_name_tool -id "@loader_path/libonnxruntime.1.23.2.dylib" "$FRAMEWORKS_DIR/libonnxruntime.1.23.2.dylib" 2>/dev/null || true
fi

# libcargs 设置 ID
if [ -f "$FRAMEWORKS_DIR/libcargs.dylib" ]; then
    echo "  - Fixing libcargs.dylib"
    install_name_tool -id "@loader_path/libcargs.dylib" "$FRAMEWORKS_DIR/libcargs.dylib" 2>/dev/null || true
fi

# 重新签名所有动态库和可执行文件
echo "Re-signing all binaries..."

# 签名所有动态库（不使用 runtime 选项）
for dylib_file in "$FRAMEWORKS_DIR"/*.dylib; do
    if [ -f "$dylib_file" ]; then
        echo "  - Signing $(basename "$dylib_file")"
        codesign --force --sign - "$dylib_file" 2>/dev/null || true
    fi
done

# 签名所有可执行文件（不使用 runtime 选项）
for binary in "$APP_BUNDLE/Contents/MacOS"/*; do
    if [ -f "$binary" ] && [ -x "$binary" ]; then
        echo "  - Signing $(basename "$binary")"
        codesign --force --sign - "$binary" 2>/dev/null || true
    fi
done

# 最后签名整个 app bundle（不使用 runtime 选项）
echo "  - Signing app bundle"
codesign --force --deep --sign - "$APP_BUNDLE" 2>/dev/null || true

echo "Done!"
