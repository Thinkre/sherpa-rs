#!/bin/bash
# KeVoiceInput Manual Installation (No GUI)
# This script can be run directly from Terminal without double-clicking

APP_NAME="KeVoiceInput.app"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_PATH="$SCRIPT_DIR/$APP_NAME"
DEST_DIR="/Applications"
DEST_PATH="$DEST_DIR/$APP_NAME"

echo ""
echo "KeVoiceInput Manual Installation"
echo "================================"
echo ""

# Check if app exists
if [ ! -d "$APP_PATH" ]; then
    echo "Error: Cannot find $APP_NAME"
    exit 1
fi

# Remove old version if exists
if [ -d "$DEST_PATH" ]; then
    echo "Removing old version..."
    rm -rf "$DEST_PATH" || sudo rm -rf "$DEST_PATH"
fi

# Copy app
echo "Installing to $DEST_DIR..."
if cp -R "$APP_PATH" "$DEST_PATH" 2>/dev/null; then
    echo "✓ Copy completed"
elif sudo cp -R "$APP_PATH" "$DEST_PATH"; then
    sudo chown -R $(whoami):staff "$DEST_PATH"
    echo "✓ Copy completed (with sudo)"
else
    echo "✗ Installation failed"
    exit 1
fi

# Remove quarantine
echo "Removing quarantine attribute..."
xattr -cr "$DEST_PATH" 2>/dev/null || sudo xattr -cr "$DEST_PATH" 2>/dev/null || true
echo "✓ Quarantine removed"

echo ""
echo "✅ Installation Complete!"
echo ""
echo "App installed to: $DEST_PATH"
echo ""
echo "To launch:"
echo "  open /Applications/KeVoiceInput.app"
echo ""
