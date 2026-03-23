#!/bin/bash

# KeVoiceInput Android Environment Setup Script
# This script automates the installation of Android development prerequisites

set -e  # Exit on error

echo "================================================"
echo "KeVoiceInput Android Environment Setup"
echo "================================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored messages
print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}➜ $1${NC}"
}

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    print_error "This script is designed for macOS. For other platforms, please refer to:"
    echo "https://tauri.app/start/prerequisites/#android"
    exit 1
fi

# Check if Homebrew is installed
print_info "Checking for Homebrew..."
if ! command -v brew &> /dev/null; then
    print_error "Homebrew not found. Please install it first:"
    echo "  /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
    exit 1
fi
print_success "Homebrew installed"

# Install Java JDK 17
print_info "Checking for Java JDK 17..."
if brew list openjdk@17 &> /dev/null; then
    print_success "OpenJDK 17 already installed"
else
    print_info "Installing OpenJDK 17..."
    brew install openjdk@17
    print_success "OpenJDK 17 installed"
fi

# Create symlink for Java
if [ -d "/Library/Java/JavaVirtualMachines/openjdk-17.jdk" ]; then
    print_success "Java symlink already exists"
else
    print_info "Creating Java symlink..."
    sudo ln -sfn /opt/homebrew/opt/openjdk@17/libexec/openjdk.jdk /Library/Java/JavaVirtualMachines/openjdk-17.jdk
    print_success "Java symlink created"
fi

# Install Android Studio
print_info "Checking for Android Studio..."
if [ -d "/Applications/Android Studio.app" ]; then
    print_success "Android Studio already installed"
else
    print_info "Installing Android Studio..."
    brew install --cask android-studio
    print_success "Android Studio installed"
fi

# Configure environment variables
print_info "Configuring environment variables..."

SHELL_RC=""
if [ -f "$HOME/.zshrc" ]; then
    SHELL_RC="$HOME/.zshrc"
elif [ -f "$HOME/.bashrc" ]; then
    SHELL_RC="$HOME/.bashrc"
else
    SHELL_RC="$HOME/.zshrc"
    touch "$SHELL_RC"
fi

# Check if environment variables are already configured
if grep -q "# KeVoiceInput Android Environment" "$SHELL_RC"; then
    print_success "Environment variables already configured in $SHELL_RC"
else
    print_info "Adding environment variables to $SHELL_RC..."
    cat >> "$SHELL_RC" << 'EOF'

# KeVoiceInput Android Environment
export JAVA_HOME="/opt/homebrew/opt/openjdk@17"
export PATH="$JAVA_HOME/bin:$PATH"

export ANDROID_HOME="$HOME/Library/Android/sdk"
export PATH="$PATH:$ANDROID_HOME/cmdline-tools/latest/bin"
export PATH="$PATH:$ANDROID_HOME/platform-tools"
export PATH="$PATH:$ANDROID_HOME/emulator"

if [ -d "$ANDROID_HOME/ndk" ]; then
    export NDK_HOME="$ANDROID_HOME/ndk/$(ls -1 $ANDROID_HOME/ndk | tail -1)"
fi
EOF
    print_success "Environment variables added to $SHELL_RC"
fi

# Source the shell rc file
source "$SHELL_RC"

# Install Rust Android targets
print_info "Installing Rust Android targets..."
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
print_success "Rust Android targets installed"

echo ""
echo "================================================"
echo "Initial Setup Complete!"
echo "================================================"
echo ""
print_warning "IMPORTANT: You still need to complete the following manual steps:"
echo ""
echo "1. Open Android Studio:"
echo "   open -a 'Android Studio'"
echo ""
echo "2. Complete the first-time setup wizard (if prompted)"
echo ""
echo "3. Install required SDK components:"
echo "   • Go to: Android Studio → Settings → SDK"
echo "   • SDK Platforms tab:"
echo "     - Install Android 13.0 (API 33) or Android 14.0 (API 34)"
echo "   • SDK Tools tab:"
echo "     - Install Android SDK Build-Tools"
echo "     - Install NDK (Side by side) - version 25.x+"
echo "     - Install CMake"
echo "     - Install Android SDK Command-line Tools"
echo ""
echo "4. After installing SDK components, restart your terminal:"
echo "   source $SHELL_RC"
echo ""
echo "5. Verify the setup:"
echo "   ./scripts/verify-android-env.sh"
echo ""
echo "6. Initialize the Android project:"
echo "   cd /Users/thinkre/Desktop/projects/KeVoiceInput"
echo "   bun run tauri android init"
echo ""
print_success "Setup script completed successfully!"
