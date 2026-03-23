#!/bin/bash
# 修复旧版 DMG 的中文文件名问题
# 用法: ./fix-old-dmg.sh <old-dmg-path> <new-dmg-path>

set -e

OLD_DMG="$1"
NEW_DMG="$2"

if [ -z "$OLD_DMG" ] || [ -z "$NEW_DMG" ]; then
    echo "Usage: $0 <old-dmg-path> <new-dmg-path>"
    echo "Example: $0 KeVoiceInput_0.0.1_aarch64.dmg KeVoiceInput_0.0.1_aarch64_fixed.dmg"
    exit 1
fi

if [ ! -f "$OLD_DMG" ]; then
    echo "Error: Old DMG not found at $OLD_DMG"
    exit 1
fi

echo "Fixing DMG installer script..."
echo "  Old DMG: $OLD_DMG"
echo "  New DMG: $NEW_DMG"

# 创建临时目录
TMP_DIR=$(mktemp -d)
echo "  - Using temp directory: $TMP_DIR"

# 挂载旧 DMG
echo "  - Mounting old DMG..."
MOUNT_POINT=$(hdiutil attach "$OLD_DMG" -readonly | grep "/Volumes/" | awk '{print $3}')

if [ -z "$MOUNT_POINT" ]; then
    echo "Error: Failed to mount DMG"
    rm -rf "$TMP_DIR"
    exit 1
fi

echo "  - Mounted at: $MOUNT_POINT"

# 复制内容到临时目录
echo "  - Copying DMG contents..."
cp -R "$MOUNT_POINT"/* "$TMP_DIR/" 2>/dev/null || true

# 卸载旧 DMG
echo "  - Unmounting old DMG..."
hdiutil detach "$MOUNT_POINT" > /dev/null

# 检查是否有旧的中文文件名
OLD_SCRIPT="$TMP_DIR/安装到应用程序.command"
NEW_SCRIPT="$TMP_DIR/Install.command"

if [ -f "$OLD_SCRIPT" ]; then
    echo "  - Renaming installer script..."
    mv "$OLD_SCRIPT" "$NEW_SCRIPT"
    chmod +x "$NEW_SCRIPT"
    echo "  ✓ Renamed: 安装到应用程序.command → Install.command"
elif [ -f "$NEW_SCRIPT" ]; then
    echo "  ⓘ DMG already has Install.command, no changes needed"
else
    echo "  - No installer script found, adding new one..."
    SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
    cp "$SCRIPT_DIR/install-app.command" "$NEW_SCRIPT"
    chmod +x "$NEW_SCRIPT"
fi

# 创建新 DMG
echo "  - Creating new DMG..."
TMP_DMG="$TMP_DIR/tmp.dmg"
hdiutil create -volname "KeVoiceInput" -srcfolder "$TMP_DIR" -ov -format UDRW "$TMP_DMG" > /dev/null

# 挂载临时 DMG 以设置视图
echo "  - Configuring Finder view..."
NEW_MOUNT=$(hdiutil attach -readwrite -noverify -noautoopen "$TMP_DMG" | grep "/Volumes/" | awk '{print $3}')

# 设置 Finder 视图（如果挂载成功）
if [ -n "$NEW_MOUNT" ]; then
    echo '
       tell application "Finder"
         tell disk "KeVoiceInput"
               open
               set current view of container window to icon view
               set toolbar visible of container window to false
               set statusbar visible of container window to false
               set the bounds of container window to {400, 100, 920, 500}
               set viewOptions to the icon view options of container window
               set arrangement of viewOptions to not arranged
               set icon size of viewOptions to 100
               set position of item "KeVoiceInput.app" of container window to {130, 120}
               set position of item "Applications" of container window to {390, 120}
               set position of item "Install.command" of container window to {260, 280}
               close
               open
               update without registering applications
               delay 2
         end tell
       end tell
    ' | osascript > /dev/null 2>&1 || true

    sleep 2
    hdiutil detach "$NEW_MOUNT" > /dev/null 2>&1 || true
fi

# 转换为压缩的只读 DMG
echo "  - Converting to compressed DMG..."
rm -f "$NEW_DMG"
hdiutil convert "$TMP_DMG" -format UDZO -o "$NEW_DMG" > /dev/null

# 清理
rm -rf "$TMP_DIR"

DMG_SIZE=$(du -h "$NEW_DMG" | cut -f1)
echo ""
echo "✅ Fixed DMG created: $NEW_DMG ($DMG_SIZE)"
echo ""
echo "Changes:"
echo "  - Installer script renamed to English: Install.command"
echo "  - All other files remain unchanged"
echo ""
echo "Test the new DMG:"
echo "  hdiutil attach \"$NEW_DMG\""
echo "  ls -la /Volumes/KeVoiceInput/"
