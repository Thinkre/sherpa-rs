#!/bin/bash
# 应用所有修复并重新构建应用

set -e

echo "================================"
echo "KeVoiceInput 修复应用脚本"
echo "================================"
echo ""

# 检查是否在项目根目录
if [ ! -f "package.json" ] || [ ! -d "src-tauri" ]; then
    echo "❌ 错误：请在项目根目录运行此脚本"
    exit 1
fi

echo "✓ 项目目录确认"
echo ""

# 步骤 1：重新构建前端
echo "步骤 1/4: 构建前端..."
bun run build
echo "✓ 前端构建完成"
echo ""

# 步骤 2：构建后端
echo "步骤 2/4: 构建后端..."
cd src-tauri
cargo build --release
cd ..
echo "✓ 后端构建完成"
echo ""

# 步骤 3：打包应用
echo "步骤 3/4: 打包应用..."
bun run tauri build
echo "✓ 应用打包完成"
echo ""

# 步骤 4：验证动态库
echo "步骤 4/4: 验证动态库..."
APP_BUNDLE="src-tauri/target/release/bundle/macos/KeVoiceInput.app"
FRAMEWORKS_DIR="$APP_BUNDLE/Contents/Frameworks"

if [ ! -d "$FRAMEWORKS_DIR" ]; then
    echo "❌ Frameworks 目录不存在！"
    exit 1
fi

DYLIB_COUNT=$(ls -1 "$FRAMEWORKS_DIR"/*.dylib 2>/dev/null | wc -l)
if [ "$DYLIB_COUNT" -lt 4 ]; then
    echo "⚠️  警告：只找到 $DYLIB_COUNT 个动态库（应该有 4 个）"
    echo "正在重新复制动态库..."
    ./scripts/copy-dylibs.sh "$APP_BUNDLE"
else
    echo "✓ 动态库完整（$DYLIB_COUNT 个文件）"
fi

echo ""
echo "验证动态库依赖..."
otool -L "$FRAMEWORKS_DIR/libsherpa-onnx-cxx-api.dylib" | grep -E "onnxruntime|sherpa"

echo ""
echo "================================"
echo "  ✅ 修复应用完成！"
echo "================================"
echo ""
echo "构建产物："
echo "  App: $APP_BUNDLE"
echo "  DMG: src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg"
echo ""
echo "下一步："
echo "  1. 测试应用：open $APP_BUNDLE"
echo "  2. 或安装 DMG："
echo "     rm -rf /Applications/KeVoiceInput.app"
echo "     hdiutil attach src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg"
echo "     /Volumes/KeVoiceInput/Install.command"
echo ""
