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

# 需要打包的 sherpa 动态库
DYLIBS=(
    "libcargs.dylib"
    "libsherpa-onnx-c-api.dylib"
    "libsherpa-onnx-cxx-api.dylib"
)

# 项目根目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 优先使用 vendor/libs/ 中的预编译库（可移植，跨平台）
ARCH=$(uname -m)
VENDOR_LIBS="$PROJECT_ROOT/vendor/libs/macos-${ARCH}"

if [ -d "$VENDOR_LIBS" ] && ls "$VENDOR_LIBS"/*.dylib >/dev/null 2>&1; then
    echo "Copying dynamic libraries from vendor/libs/macos-${ARCH}..."
    for dylib in "$VENDOR_LIBS"/*.dylib; do
        NAME="$(basename "$dylib")"
        echo "  - Copying $NAME"
        cp "$dylib" "$FRAMEWORKS_DIR/"
    done
else
    echo "Warning: vendor/libs/macos-${ARCH} not found, falling back to build output..."

    # Fallback: 从 build output 搜索
    TARGET_RELEASE="$(cd "$(dirname "$APP_BUNDLE")/../.." && pwd)"
    RELEASE_DIR="$TARGET_RELEASE/release"
    SHERPA_BUILD_OUT="$(find "$TARGET_RELEASE/build" -maxdepth 2 -name "sherpa-rs-sys-*" -type d 2>/dev/null | head -1)/out"

    # onnxruntime
    ONNX_DYLIB=""
    for candidate_dir in "$RELEASE_DIR" "$TARGET_RELEASE" "$SHERPA_BUILD_OUT"; do
        found=$(find "$candidate_dir" -maxdepth 6 -name "libonnxruntime.*.dylib" ! -name "libonnxruntime.dylib" 2>/dev/null | head -1)
        if [ -n "$found" ]; then
            ONNX_DYLIB="$found"
            ONNX_NAME="$(basename "$found")"
            break
        fi
    done

    echo "Copying dynamic libraries to $FRAMEWORKS_DIR..."
    if [ -n "$ONNX_DYLIB" ]; then
        echo "  - Copying $ONNX_NAME"
        cp "$ONNX_DYLIB" "$FRAMEWORKS_DIR/"
    else
        echo "  - Warning: libonnxruntime.*.dylib not found"
    fi

    for dylib in "${DYLIBS[@]}"; do
        SRC=""
        for search_dir in "$RELEASE_DIR" "$TARGET_RELEASE" "$SHERPA_BUILD_OUT"; do
            candidate=$(find "$search_dir" -maxdepth 3 -name "$dylib" 2>/dev/null | head -1)
            if [ -n "$candidate" ]; then
                SRC="$candidate"
                break
            fi
        done
        if [ -n "$SRC" ]; then
            echo "  - Copying $dylib"
            cp "$SRC" "$FRAMEWORKS_DIR/"
        else
            echo "  - Warning: $dylib not found"
        fi
    done
fi

# Remove any extra onnxruntime versions
ONNX_IN_FRAMEWORKS=$(find "$FRAMEWORKS_DIR" -name "libonnxruntime.*.dylib" ! -name "libonnxruntime.dylib" 2>/dev/null | head -1)
ONNX_KEEP="$(basename "$ONNX_IN_FRAMEWORKS" 2>/dev/null)"
find "$FRAMEWORKS_DIR" -name "libonnxruntime.*.dylib" ! -name "libonnxruntime.dylib" ! -name "$ONNX_KEEP" -delete 2>/dev/null

# 至少需要 onnxruntime 和 sherpa 才能启动
if [ -z "$ONNX_IN_FRAMEWORKS" ] || [ ! -f "$FRAMEWORKS_DIR/libsherpa-onnx-c-api.dylib" ]; then
    echo ""
    echo "❌ 缺少关键动态库，应用将无法启动。"
    echo "   请将预编译 dylib 放到 vendor/libs/macos-$(uname -m)/ 目录"
    exit 1
fi

# 记录实际使用的 onnxruntime 版本名
ONNX_NAME="$(basename "$ONNX_IN_FRAMEWORKS")"

# 修改二进制文件的 rpath
BINARY="$APP_BUNDLE/Contents/MacOS/kevoiceinput"
echo "Updating rpath in $BINARY..."

# 主程序：将所有 onnxruntime 版本的 rpath 指向实际存在的版本
install_name_tool -change "@rpath/libonnxruntime.1.17.1.dylib" "@executable_path/../Frameworks/$ONNX_NAME" "$BINARY" 2>/dev/null || true
install_name_tool -change "@rpath/libonnxruntime.1.23.2.dylib" "@executable_path/../Frameworks/$ONNX_NAME" "$BINARY" 2>/dev/null || true
install_name_tool -change "@rpath/$ONNX_NAME" "@executable_path/../Frameworks/$ONNX_NAME" "$BINARY" 2>/dev/null || true
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
    install_name_tool -change "@rpath/libonnxruntime.1.17.1.dylib" "@loader_path/$ONNX_NAME" "$FRAMEWORKS_DIR/libsherpa-onnx-c-api.dylib" 2>/dev/null || true
    install_name_tool -change "@loader_path/libonnxruntime.1.17.1.dylib" "@loader_path/$ONNX_NAME" "$FRAMEWORKS_DIR/libsherpa-onnx-c-api.dylib" 2>/dev/null || true
    install_name_tool -change "@rpath/libonnxruntime.1.23.2.dylib" "@loader_path/$ONNX_NAME" "$FRAMEWORKS_DIR/libsherpa-onnx-c-api.dylib" 2>/dev/null || true
    install_name_tool -change "@loader_path/libonnxruntime.1.23.2.dylib" "@loader_path/$ONNX_NAME" "$FRAMEWORKS_DIR/libsherpa-onnx-c-api.dylib" 2>/dev/null || true
    install_name_tool -id "@loader_path/libsherpa-onnx-c-api.dylib" "$FRAMEWORKS_DIR/libsherpa-onnx-c-api.dylib" 2>/dev/null || true
fi

# libsherpa-onnx-cxx-api.dylib 依赖
if [ -f "$FRAMEWORKS_DIR/libsherpa-onnx-cxx-api.dylib" ]; then
    echo "  - Fixing libsherpa-onnx-cxx-api.dylib"
    install_name_tool -change "@rpath/libonnxruntime.1.17.1.dylib" "@loader_path/$ONNX_NAME" "$FRAMEWORKS_DIR/libsherpa-onnx-cxx-api.dylib" 2>/dev/null || true
    install_name_tool -change "@loader_path/libonnxruntime.1.17.1.dylib" "@loader_path/$ONNX_NAME" "$FRAMEWORKS_DIR/libsherpa-onnx-cxx-api.dylib" 2>/dev/null || true
    install_name_tool -change "@rpath/libonnxruntime.1.23.2.dylib" "@loader_path/$ONNX_NAME" "$FRAMEWORKS_DIR/libsherpa-onnx-cxx-api.dylib" 2>/dev/null || true
    install_name_tool -change "@loader_path/libonnxruntime.1.23.2.dylib" "@loader_path/$ONNX_NAME" "$FRAMEWORKS_DIR/libsherpa-onnx-cxx-api.dylib" 2>/dev/null || true
    install_name_tool -change "@rpath/libsherpa-onnx-c-api.dylib" "@loader_path/libsherpa-onnx-c-api.dylib" "$FRAMEWORKS_DIR/libsherpa-onnx-cxx-api.dylib" 2>/dev/null || true
    install_name_tool -id "@loader_path/libsherpa-onnx-cxx-api.dylib" "$FRAMEWORKS_DIR/libsherpa-onnx-cxx-api.dylib" 2>/dev/null || true
fi

# libonnxruntime 设置 ID
if [ -f "$FRAMEWORKS_DIR/$ONNX_NAME" ]; then
    echo "  - Fixing $ONNX_NAME"
    install_name_tool -id "@loader_path/$ONNX_NAME" "$FRAMEWORKS_DIR/$ONNX_NAME" 2>/dev/null || true
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
