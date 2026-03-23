#!/bin/bash
# 创建专业的 DMG 安装包，带有 Applications 文件夹链接

set -e

APP_NAME="KeVoiceInput"
APP_BUNDLE="$1"
DMG_OUTPUT="$2"
VOLUME_NAME="$APP_NAME"

if [ -z "$APP_BUNDLE" ] || [ -z "$DMG_OUTPUT" ]; then
    echo "Usage: $0 <path-to-app-bundle> <output-dmg-path>"
    exit 1
fi

if [ ! -d "$APP_BUNDLE" ]; then
    echo "Error: App bundle not found at $APP_BUNDLE"
    exit 1
fi

echo "Creating DMG installer for $APP_NAME..."

# 创建临时目录
TMP_DIR=$(mktemp -d)
DMG_DIR="$TMP_DIR/dmg"
mkdir -p "$DMG_DIR"

# 复制 app 到临时目录
echo "  - Copying app bundle..."
cp -R "$APP_BUNDLE" "$DMG_DIR/"

# 创建 Applications 文件夹的符号链接
echo "  - Creating Applications symlink..."
ln -s /Applications "$DMG_DIR/Applications"

# 复制安装脚本（使用英文文件名以避免终端路径问题）
echo "  - Adding installer script..."
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cp "$SCRIPT_DIR/install-app.command" "$DMG_DIR/Install.command"
chmod +x "$DMG_DIR/Install.command"
# 移除隔离属性，避免 Gatekeeper 阻止（构建机上的属性；其他 Mac 下载 DMG 后仍可能被标记）
xattr -cr "$DMG_DIR/Install.command" 2>/dev/null || true
# 若提供 Developer ID，对安装脚本签名，便于其他 Mac 上通过 Gatekeeper（需配合 DMG 公证）
if [ -n "$MACOS_INSTALLER_SIGNING_IDENTITY" ] && [ "$MACOS_INSTALLER_SIGNING_IDENTITY" != "-" ]; then
    echo "  - Signing Install.command with Developer ID..."
    if codesign -s "$MACOS_INSTALLER_SIGNING_IDENTITY" --force --timestamp "$DMG_DIR/Install.command" 2>/dev/null; then
        echo "    ✓ Install.command signed"
    else
        echo "    ⚠ Signing Install.command failed, DMG may show security warning on other Macs"
    fi
fi

# 添加安装说明和备用安装脚本
echo "  - Adding installation guide..."
cp "$SCRIPT_DIR/dmg-readme.txt" "$DMG_DIR/README.txt"
cp "$SCRIPT_DIR/manual-install.sh" "$DMG_DIR/manual-install.sh"
chmod +x "$DMG_DIR/manual-install.sh"
xattr -cr "$DMG_DIR/manual-install.sh" 2>/dev/null || true
if [ -n "$MACOS_INSTALLER_SIGNING_IDENTITY" ] && [ "$MACOS_INSTALLER_SIGNING_IDENTITY" != "-" ]; then
    codesign -s "$MACOS_INSTALLER_SIGNING_IDENTITY" --force --timestamp "$DMG_DIR/manual-install.sh" 2>/dev/null || true
fi

# 创建临时 DMG
echo "  - Creating temporary DMG..."
TMP_DMG="$TMP_DIR/tmp.dmg"
hdiutil create -volname "$VOLUME_NAME" -srcfolder "$DMG_DIR" -ov -format UDRW "$TMP_DMG" > /dev/null

# 挂载临时 DMG 以设置视图选项
echo "  - Mounting temporary DMG..."
MOUNT_DIR=$(hdiutil attach -readwrite -noverify -noautoopen "$TMP_DMG" | grep -E "/Volumes/" | sed 's/.*\/Volumes/\/Volumes/')

# 设置 Finder 视图选项
echo "  - Configuring Finder view..."
echo '
   tell application "Finder"
     tell disk "'$VOLUME_NAME'"
           open
           set current view of container window to icon view
           set toolbar visible of container window to false
           set statusbar visible of container window to false
           set the bounds of container window to {400, 100, 920, 500}
           set viewOptions to the icon view options of container window
           set arrangement of viewOptions to not arranged
           set icon size of viewOptions to 100
           set position of item "'$APP_NAME'.app" of container window to {100, 120}
           set position of item "Applications" of container window to {400, 120}
           set position of item "Install.command" of container window to {100, 280}
           set position of item "README.txt" of container window to {400, 280}
           close
           open
           update without registering applications
           delay 2
     end tell
   end tell
' | osascript > /dev/null 2>&1 || true

# 等待 Finder 完成
sleep 2

# 卸载临时 DMG
echo "  - Unmounting temporary DMG..."
hdiutil detach "$MOUNT_DIR" > /dev/null 2>&1 || true
sleep 1

# 转换为压缩的只读 DMG
echo "  - Converting to compressed DMG..."
rm -f "$DMG_OUTPUT"
hdiutil convert "$TMP_DMG" -format UDZO -o "$DMG_OUTPUT" > /dev/null

# 清理临时文件
rm -rf "$TMP_DIR"

DMG_SIZE=$(du -h "$DMG_OUTPUT" | cut -f1)
echo "✅ DMG created: $DMG_OUTPUT ($DMG_SIZE)"
