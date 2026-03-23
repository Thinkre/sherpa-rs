#!/bin/bash
# KeVoiceInput Auto Installer
# Double-click this file to install the app

APP_NAME="KeVoiceInput.app"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_PATH="$SCRIPT_DIR/$APP_NAME"

# Terminal colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo ""
echo "════════════════════════════════════════════"
echo "   KeVoiceInput Auto Installer"
echo "════════════════════════════════════════════"
echo ""

# Check if app exists
if [ ! -d "$APP_PATH" ]; then
    echo -e "${RED}Error: Cannot find $APP_NAME${NC}"
    echo "Please make sure this script is in the same DMG as the app"
    echo ""
    read -p "Press any key to close..." -n1 -s
    exit 1
fi

# Ask for installation location
echo "Please select installation location:"
echo "  1) /Applications (All users, recommended)"
echo "  2) ~/Applications (Current user only)"
echo ""
read -p "Enter option [1]: " choice
choice=${choice:-1}

if [ "$choice" = "2" ]; then
    DEST_DIR="$HOME/Applications"
    mkdir -p "$DEST_DIR"
else
    DEST_DIR="/Applications"
fi

DEST_PATH="$DEST_DIR/$APP_NAME"

echo ""
echo -e "${BLUE}Installing to: $DEST_DIR${NC}"

# If already exists, ask to overwrite
if [ -d "$DEST_PATH" ]; then
    echo ""
    echo -e "${RED}Warning: App already exists${NC}"
    read -p "Overwrite existing version? (y/n) [y]: " overwrite
    overwrite=${overwrite:-y}

    if [ "$overwrite" != "y" ] && [ "$overwrite" != "Y" ]; then
        echo "Installation cancelled"
        echo ""
        read -p "Press any key to close..." -n1 -s
        exit 0
    fi

    echo "Removing old version..."
    rm -rf "$DEST_PATH"
fi

# Copy app
echo "Copying app files..."
if cp -R "$APP_PATH" "$DEST_PATH"; then
    echo -e "${GREEN}✓ Copy completed${NC}"
else
    echo -e "${RED}✗ Copy failed${NC}"
    echo "Administrator permission may be required"
    echo ""
    read -p "Retry with sudo? (y/n) [y]: " use_sudo
    use_sudo=${use_sudo:-y}

    if [ "$use_sudo" = "y" ] || [ "$use_sudo" = "Y" ]; then
        echo "Please enter your password:"
        sudo cp -R "$APP_PATH" "$DEST_PATH"
        sudo chown -R $(whoami):staff "$DEST_PATH"
    else
        echo "Installation cancelled"
        echo ""
        read -p "Press any key to close..." -n1 -s
        exit 1
    fi
fi

# Remove quarantine attribute (bypass Gatekeeper)
echo "Removing quarantine attribute..."
if xattr -rd com.apple.quarantine "$DEST_PATH" 2>/dev/null; then
    echo -e "${GREEN}✓ App is cleared, can launch directly${NC}"
else
    echo -e "${BLUE}ⓘ No quarantine attribute or already cleared${NC}"
fi

# Remove other extended attributes
xattr -cr "$DEST_PATH" 2>/dev/null || true

echo ""
echo "════════════════════════════════════════════"
echo -e "${GREEN}   ✓ Installation Complete!${NC}"
echo "════════════════════════════════════════════"
echo ""
echo "App installed to: $DEST_PATH"
echo ""
echo "How to launch:"
echo "  • Open $DEST_DIR in Finder"
echo "  • Search 'KeVoice' in Launchpad"
echo "  • Use Spotlight search"
echo ""

# Ask to launch now
read -p "Launch app now? (y/n) [y]: " launch
launch=${launch:-y}

if [ "$launch" = "y" ] || [ "$launch" = "Y" ]; then
    echo "Launching app..."
    open "$DEST_PATH"
    echo -e "${GREEN}✓ Launched${NC}"
fi

echo ""
echo "Thank you for using KeVoiceInput!"
echo ""
read -p "Press any key to close..." -n1 -s
