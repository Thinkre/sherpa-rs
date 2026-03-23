#!/bin/bash
set -e

# 设置代理
export https_proxy=http://127.0.0.1:7897
export http_proxy=http://127.0.0.1:7897
export all_proxy=socks5://127.0.0.1:7897

echo "================================"
echo "升级 onnxruntime 到 1.23.2（使用代理）"
echo "================================"
echo "代理: $https_proxy"
echo ""

PROJECT_DIR="/Users/thinkre/Desktop/projects/KeVoiceInput"
DOWNLOAD_URL="https://github.com/microsoft/onnxruntime/releases/download/v1.23.2/onnxruntime-osx-universal2-1.23.2.tgz"
DOWNLOAD_FILE="$HOME/Downloads/onnxruntime-osx-universal2-1.23.2.tgz"
EXTRACT_DIR="$HOME/Downloads/onnxruntime-osx-universal2-1.23.2"
TARGET_DIR="$PROJECT_DIR/src-tauri/target/release"

# 步骤 1: 下载
if [ ! -f "$DOWNLOAD_FILE" ]; then
    echo "步骤 1/6: 下载 onnxruntime 1.23.2..."
    echo "URL: $DOWNLOAD_URL"
    curl -L "$DOWNLOAD_URL" -o "$DOWNLOAD_FILE" --progress-bar
    echo "✓ 下载完成 ($(ls -lh $DOWNLOAD_FILE | awk '{print $5}'))"
else
    echo "步骤 1/6: 文件已存在，跳过下载"
    ls -lh "$DOWNLOAD_FILE"
fi
echo ""

# 步骤 2: 解压
if [ ! -d "$EXTRACT_DIR" ]; then
    echo "步骤 2/6: 解压文件..."
    cd ~/Downloads
    tar -xzf "$(basename $DOWNLOAD_FILE)"
    echo "✓ 解压完成"
else
    echo "步骤 2/6: 已解压，跳过"
fi
echo ""

# 步骤 3: 验证库文件
echo "步骤 3/6: 验证下载的库..."
NEW_LIB="$EXTRACT_DIR/lib/libonnxruntime.1.23.2.dylib"
if [ ! -f "$NEW_LIB" ]; then
    echo "❌ 错误: 找不到 $NEW_LIB"
    echo "解压目录内容:"
    ls -la "$EXTRACT_DIR"
    exit 1
fi

echo "文件类型:"
file "$NEW_LIB"
echo ""
echo "库依赖:"
otool -L "$NEW_LIB" | head -5
echo "✓ 库文件验证成功"
echo ""

# 步骤 4: 备份旧库
echo "步骤 4/6: 备份旧版本..."
cd "$TARGET_DIR"
if [ -f "libonnxruntime.1.17.1.dylib" ] && [ ! -L "libonnxruntime.1.17.1.dylib" ]; then
    mv libonnxruntime.1.17.1.dylib libonnxruntime.1.17.1.dylib.bak
    echo "✓ 旧库已备份为 libonnxruntime.1.17.1.dylib.bak"
else
    if [ -L "libonnxruntime.1.17.1.dylib" ]; then
        rm -f libonnxruntime.1.17.1.dylib
        echo "✓ 删除旧符号链接"
    else
        echo "✓ 无需备份"
    fi
fi
echo ""

# 步骤 5: 复制新库
echo "步骤 5/6: 安装新库..."
cp "$NEW_LIB" "$TARGET_DIR/"
echo "✓ 新库已复制"

# 创建符号链接（保持兼容性）
rm -f libonnxruntime.1.17.1.dylib libonnxruntime.dylib
ln -sf libonnxruntime.1.23.2.dylib libonnxruntime.1.17.1.dylib
ln -sf libonnxruntime.1.23.2.dylib libonnxruntime.dylib
echo "✓ 符号链接已创建"
echo ""

# 步骤 6: 验证安装
echo "步骤 6/6: 验证安装..."
echo "已安装的库:"
ls -lh libonnxruntime* | grep -v ".bak"
echo ""
echo "新库 ID:"
otool -L libonnxruntime.1.23.2.dylib | grep onnxruntime
echo ""

echo "================================"
echo "  ✅ onnxruntime 升级完成！"
echo "================================"
echo ""
echo "版本对比:"
echo "  旧版本: 1.17.1 (2025-08-16)"
echo "  新版本: 1.23.2 (2025-06-xx)"
echo ""
echo "接下来的步骤："
echo "  1. 更新 copy-dylibs.sh 脚本以识别新版本"
echo "  2. 重新构建应用："
echo "     ./scripts/tauri-build-wrapper.sh build"
echo ""
echo "  3. 重新安装："
echo "     rm -rf /Applications/KeVoiceInput.app"
echo "     hdiutil attach src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg"
echo "     /Volumes/KeVoiceInput/Install.command"
echo ""
