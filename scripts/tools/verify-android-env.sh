#!/bin/bash

# KeVoiceInput Android Environment Verification Script

set +e  # Don't exit on error, we want to check everything

echo "================================================"
echo "KeVoiceInput Android Environment Verification"
echo "================================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counters
PASS=0
FAIL=0

check_command() {
    local cmd=$1
    local name=$2
    local required=${3:-true}

    if command -v $cmd &> /dev/null; then
        echo -e "${GREEN}✓${NC} $name: Found"
        ((PASS++))
        return 0
    else
        if [ "$required" = true ]; then
            echo -e "${RED}✗${NC} $name: Not found (REQUIRED)"
            ((FAIL++))
        else
            echo -e "${YELLOW}⚠${NC} $name: Not found (optional)"
        fi
        return 1
    fi
}

check_env_var() {
    local var=$1
    local name=$2

    if [ -n "${!var}" ]; then
        echo -e "${GREEN}✓${NC} $name: ${!var}"
        ((PASS++))
        return 0
    else
        echo -e "${RED}✗${NC} $name: Not set"
        ((FAIL++))
        return 1
    fi
}

check_directory() {
    local dir=$1
    local name=$2

    if [ -d "$dir" ]; then
        echo -e "${GREEN}✓${NC} $name: Found at $dir"
        ((PASS++))
        return 0
    else
        echo -e "${RED}✗${NC} $name: Not found at $dir"
        ((FAIL++))
        return 1
    fi
}

# Check Java
echo "Checking Java..."
if check_command java "Java Runtime"; then
    JAVA_VERSION=$(java -version 2>&1 | head -n 1)
    echo "  Version: $JAVA_VERSION"
    if check_env_var JAVA_HOME "JAVA_HOME"; then
        :
    fi
fi
echo ""

# Check Rust
echo "Checking Rust..."
if check_command rustc "Rust Compiler"; then
    RUST_VERSION=$(rustc --version)
    echo "  Version: $RUST_VERSION"
fi
echo ""

# Check Rust Android targets
echo "Checking Rust Android targets..."
for target in aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android; do
    if rustup target list --installed | grep -q $target; then
        echo -e "${GREEN}✓${NC} $target: Installed"
        ((PASS++))
    else
        echo -e "${RED}✗${NC} $target: Not installed"
        echo "  Install with: rustup target add $target"
        ((FAIL++))
    fi
done
echo ""

# Check Android environment
echo "Checking Android environment..."
check_env_var ANDROID_HOME "ANDROID_HOME"
check_env_var NDK_HOME "NDK_HOME"
echo ""

# Check Android SDK directories
if [ -n "$ANDROID_HOME" ]; then
    echo "Checking Android SDK components..."
    check_directory "$ANDROID_HOME" "Android SDK"
    check_directory "$ANDROID_HOME/platform-tools" "Platform Tools"
    check_directory "$ANDROID_HOME/cmdline-tools" "Command-line Tools"
    check_directory "$ANDROID_HOME/emulator" "Emulator" false

    # Check for at least one platform
    if [ -d "$ANDROID_HOME/platforms" ]; then
        PLATFORMS=$(ls -1 $ANDROID_HOME/platforms 2>/dev/null | wc -l | xargs)
        if [ "$PLATFORMS" -gt 0 ]; then
            echo -e "${GREEN}✓${NC} Android Platforms: $PLATFORMS installed"
            ls -1 $ANDROID_HOME/platforms | sed 's/^/  - /'
            ((PASS++))
        else
            echo -e "${RED}✗${NC} Android Platforms: None installed"
            ((FAIL++))
        fi
    else
        echo -e "${RED}✗${NC} Android Platforms: Directory not found"
        ((FAIL++))
    fi

    # Check for NDK
    if [ -d "$ANDROID_HOME/ndk" ]; then
        NDK_VERSIONS=$(ls -1 $ANDROID_HOME/ndk 2>/dev/null | wc -l | xargs)
        if [ "$NDK_VERSIONS" -gt 0 ]; then
            echo -e "${GREEN}✓${NC} Android NDK: $NDK_VERSIONS version(s) installed"
            ls -1 $ANDROID_HOME/ndk | sed 's/^/  - /'
            ((PASS++))
        else
            echo -e "${RED}✗${NC} Android NDK: None installed"
            ((FAIL++))
        fi
    else
        echo -e "${RED}✗${NC} Android NDK: Directory not found"
        ((FAIL++))
    fi
    echo ""
fi

# Check Android tools
echo "Checking Android tools..."
check_command adb "Android Debug Bridge (adb)"
check_command emulator "Android Emulator" false
echo ""

# Check build tools
if [ -d "$ANDROID_HOME/build-tools" ]; then
    BUILD_TOOLS=$(ls -1 $ANDROID_HOME/build-tools 2>/dev/null | wc -l | xargs)
    if [ "$BUILD_TOOLS" -gt 0 ]; then
        echo -e "${GREEN}✓${NC} Build Tools: $BUILD_TOOLS version(s) installed"
        ls -1 $ANDROID_HOME/build-tools | sed 's/^/  - /'
        ((PASS++))
    else
        echo -e "${RED}✗${NC} Build Tools: None installed"
        ((FAIL++))
    fi
else
    echo -e "${RED}✗${NC} Build Tools: Directory not found"
    ((FAIL++))
fi
echo ""

# Summary
echo "================================================"
echo "Verification Summary"
echo "================================================"
echo -e "${GREEN}Passed:${NC} $PASS"
echo -e "${RED}Failed:${NC} $FAIL"
echo ""

if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}✓ All checks passed! Your Android environment is ready.${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Initialize Android project:"
    echo "     cd /Users/thinkre/Desktop/projects/KeVoiceInput"
    echo "     bun run tauri android init"
    echo ""
    exit 0
else
    echo -e "${RED}✗ Some checks failed. Please install missing components.${NC}"
    echo ""
    echo "Common fixes:"
    echo "  1. If ANDROID_HOME is not set, restart your terminal:"
    echo "     source ~/.zshrc"
    echo ""
    echo "  2. If Android SDK components are missing:"
    echo "     open -a 'Android Studio'"
    echo "     Settings → SDK → Install required components"
    echo ""
    echo "  3. If Rust Android targets are missing:"
    echo "     rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android"
    echo ""
    exit 1
fi
