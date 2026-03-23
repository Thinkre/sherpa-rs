#!/bin/bash
# KeVoiceInput 崩溃诊断脚本

APP_PATH="${1:-/Applications/KeVoiceInput.app}"
BINARY="$APP_PATH/Contents/MacOS/kevoiceinput"
FRAMEWORKS="$APP_PATH/Contents/Frameworks"

echo "KeVoiceInput Crash Diagnostics"
echo "==============================="
echo ""

# 检查应用是否存在
if [ ! -d "$APP_PATH" ]; then
    echo "❌ App not found at: $APP_PATH"
    exit 1
fi

echo "✓ App found at: $APP_PATH"
echo ""

# 检查二进制文件
if [ ! -f "$BINARY" ]; then
    echo "❌ Binary not found: $BINARY"
    exit 1
fi

echo "✓ Binary found"
echo ""

# 检查 Frameworks 目录
if [ ! -d "$FRAMEWORKS" ]; then
    echo "❌ Frameworks directory not found!"
    echo "   This is the problem - dynamic libraries are missing"
    echo ""
    echo "Solution:"
    echo "  Reinstall the app using the DMG with Install.command"
    exit 1
fi

echo "✓ Frameworks directory exists"
echo ""

# 检查必需的动态库（onnxruntime 使用 1.23.2）
echo "Checking dynamic libraries..."
REQUIRED_LIBS=(
    "libcargs.dylib"
    "libonnxruntime.1.23.2.dylib"
    "libsherpa-onnx-c-api.dylib"
    "libsherpa-onnx-cxx-api.dylib"
)

MISSING_LIBS=()
for lib in "${REQUIRED_LIBS[@]}"; do
    if [ -f "$FRAMEWORKS/$lib" ]; then
        echo "  ✓ $lib"
    else
        echo "  ❌ $lib (MISSING)"
        MISSING_LIBS+=("$lib")
    fi
done

if [ ${#MISSING_LIBS[@]} -gt 0 ]; then
    echo ""
    echo "❌ Missing ${#MISSING_LIBS[@]} required libraries"
    echo ""
    echo "Solution:"
    echo "  Reinstall the app using the DMG with Install.command"
    exit 1
fi

# 检查 sherpa 是否错误引用 1.17.1（会导致意外退出）
echo "Checking sherpa-onnx → onnxruntime link..."
SHERPA_BAD_REF=0
for sherpa in libsherpa-onnx-c-api.dylib libsherpa-onnx-cxx-api.dylib; do
    if [ -f "$FRAMEWORKS/$sherpa" ]; then
        if otool -L "$FRAMEWORKS/$sherpa" | grep -q "libonnxruntime.1.17.1.dylib"; then
            echo "  ❌ $sherpa still links to libonnxruntime.1.17.1.dylib (should be 1.23.2) → 会导致意外退出"
            SHERPA_BAD_REF=1
        else
            echo "  ✓ $sherpa → libonnxruntime.1.23.2.dylib"
        fi
    fi
done
if [ $SHERPA_BAD_REF -eq 1 ]; then
    echo ""
    echo "Solution: 使用最新构建的 DMG 重新安装（copy-dylibs 已修复此引用）"
    exit 1
fi

echo ""
echo "✓ All required libraries present"
echo ""

# 检查二进制依赖
echo "Checking binary dependencies..."
DEP_ISSUES=0

# 检查主二进制
echo "  Binary: kevoiceinput"
otool -L "$BINARY" | grep -E "(@executable_path|@rpath)" | while read -r line; do
    lib_path=$(echo "$line" | awk '{print $1}')
    lib_name=$(basename "$lib_path")

    # 转换路径
    if [[ "$lib_path" == @executable_path/* ]]; then
        actual_path="${lib_path/@executable_path/$APP_PATH/Contents/MacOS}"
    elif [[ "$lib_path" == @rpath/* ]]; then
        actual_path="$FRAMEWORKS/${lib_name}"
    else
        actual_path="$lib_path"
    fi

    if [ -f "$actual_path" ]; then
        echo "    ✓ $lib_name"
    else
        echo "    ❌ $lib_name (path: $lib_path)"
        DEP_ISSUES=$((DEP_ISSUES + 1))
    fi
done

# 检查动态库之间的依赖
for lib in "${REQUIRED_LIBS[@]}"; do
    if [ -f "$FRAMEWORKS/$lib" ]; then
        echo "  Library: $lib"
        otool -L "$FRAMEWORKS/$lib" | grep -E "(@loader_path|@rpath)" | while read -r line; do
            lib_path=$(echo "$line" | awk '{print $1}')
            lib_name=$(basename "$lib_path")

            # 转换路径
            if [[ "$lib_path" == @loader_path/* ]]; then
                actual_path="${lib_path/@loader_path/$FRAMEWORKS}"
            elif [[ "$lib_path" == @rpath/* ]]; then
                actual_path="$FRAMEWORKS/${lib_name}"
            else
                actual_path="$lib_path"
            fi

            if [ -f "$actual_path" ]; then
                echo "    ✓ $lib_name"
            else
                echo "    ❌ $lib_name (path: $lib_path)"
                DEP_ISSUES=$((DEP_ISSUES + 1))
            fi
        done
    fi
done

echo ""

if [ $DEP_ISSUES -gt 0 ]; then
    echo "❌ Found $DEP_ISSUES dependency issues"
    echo ""
    echo "This explains the crash. Dynamic libraries have incorrect paths."
    echo ""
    echo "Solution:"
    echo "  Use the latest DMG which has this fixed"
    exit 1
fi

echo "✓ All dependencies are correctly linked"
echo ""

# 尝试启动应用
echo "Attempting to launch app..."
open "$APP_PATH" &
APP_PID=$!

sleep 3

# 检查是否还在运行
if ps -p $APP_PID > /dev/null 2>&1; then
    echo "✓ App launched successfully!"
    echo ""
    echo "Everything looks good. If the app still crashes:"
    echo "  1. Check Console.app for crash logs"
    echo "  2. Look for 'kevoiceinput' in crash reports"
    echo "  3. Report the issue with the crash log"
else
    echo "❌ App crashed immediately"
    echo ""
    echo "Please check Console.app for crash logs:"
    echo "  1. Open Console.app"
    echo "  2. Go to 'Crash Reports' in the sidebar"
    echo "  3. Look for 'kevoiceinput' crash logs"
    echo "  4. Share the crash log when reporting the issue"
fi

echo ""
echo "System Information:"
echo "  macOS: $(sw_vers -productVersion)"
echo "  Arch: $(uname -m)"
echo "  Shell: $SHELL"
