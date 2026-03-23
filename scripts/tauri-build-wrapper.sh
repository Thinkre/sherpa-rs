#!/bin/bash
# 包装 tauri build 命令，在构建后自动复制动态库

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# 若为 build 且存在本地更新密钥，则自动设置签名环境变量（避免更新包签名失败）
if [ "$1" = "build" ]; then
  KEY_FILE="$PROJECT_ROOT/.tauri-updater.key"
  if [ -f "$KEY_FILE" ]; then
    export TAURI_SIGNING_PRIVATE_KEY="$(cat "$KEY_FILE")"
    # 仅当存在密码文件或已设置环境变量时设置密码；否则使用空密码（与 tauri signer generate -p "" 一致）
    if [ -n "$TAURI_SIGNING_PRIVATE_KEY_PASSWORD" ]; then
      : # 已由用户设置，保持不变
    elif [ -f "$PROJECT_ROOT/.tauri-updater.key.password" ]; then
      export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="$(cat "$PROJECT_ROOT/.tauri-updater.key.password")"
    else
      export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
    fi
  fi
  # 避免 tauri CLI 的 --ci 报错（未传值时用 false）
  export CI="${CI:-false}"
fi

echo "Building Tauri application..."
cd "$PROJECT_ROOT" && bun run tauri "$@"

# 保存构建的退出码
BUILD_EXIT_CODE=$?

# 如果是 build 命令，无论构建是否成功都执行后处理
if [ "$1" = "build" ]; then
    echo ""
    echo "Running post-build processing..."

    APP_BUNDLE="$PROJECT_ROOT/src-tauri/target/release/bundle/macos/KeVoiceInput.app"

    if [ -d "$APP_BUNDLE" ]; then
        # 复制动态库并重新签名
        if "$SCRIPT_DIR/copy-dylibs.sh" "$APP_BUNDLE"; then
            echo ""
            echo "✅ Dynamic libraries copied and signed!"
            echo "📦 App: $APP_BUNDLE"

            # 重新创建 DMG（包含修复后的 app 和 Applications 链接）
            # 命名规范: KeVoiceInput-<version>-macos-<arch>.dmg（便于上传 GitHub Release）
            echo ""
            echo "Creating professional DMG installer..."
            DMG_DIR="$(dirname "$APP_BUNDLE")/../dmg"
            Tauri_VERSION=$(grep -o '"version": *"[^"]*"' "$PROJECT_ROOT/src-tauri/tauri.conf.json" | head -1 | sed 's/"version": *"\(.*\)"/\1/')
            ARCH=$(uname -m)
            case "$ARCH" in
                arm64) ARCH="aarch64" ;;
                x86_64) ;;
                *) ;;
            esac
            DMG_FILE="$DMG_DIR/KeVoiceInput-${Tauri_VERSION:-0.0.1}-macos-${ARCH}.dmg"

            if "$SCRIPT_DIR/create-dmg.sh" "$APP_BUNDLE" "$DMG_FILE"; then
                DMG_SIZE=$(du -h "$DMG_FILE" | cut -f1)
                echo "   📦 Size: $DMG_SIZE"
            else
                echo "⚠️  Warning: Failed to create DMG"
            fi

            echo ""
            echo "Testing app launch..."
            # 测试应用能否启动（使用 open 命令模拟真实启动）
            open "$APP_BUNDLE" > /dev/null 2>&1 &
            sleep 3
            if ps aux | grep -q "[k]evoiceinput"; then
                echo "✅ App can launch successfully!"
                pkill -9 kevoiceinput 2>/dev/null
            else
                echo "⚠️  Warning: App failed to launch"
            fi

            echo ""
            echo "════════════════════════════════════════════"
            echo "  ✅ Build Complete!"
            echo "════════════════════════════════════════════"
            echo "  App:  $APP_BUNDLE"
            echo "  DMG:  $DMG_FILE"
            echo "════════════════════════════════════════════"
            # 后处理全部成功则视为构建成功，避免 Tauri CLI 的次要错误导致整体 exit 1
            POST_BUILD_OK=1
        else
            echo "⚠️  Warning: Failed to copy dynamic libraries"
        fi
    else
        echo "⚠️  App bundle not found at $APP_BUNDLE"
        echo "Build might have failed. Check the output above."
    fi
fi

# 后处理成功则返回 0，否则沿用 Tauri 的退出码
if [ -n "${POST_BUILD_OK-}" ]; then
  exit 0
fi
exit $BUILD_EXIT_CODE
