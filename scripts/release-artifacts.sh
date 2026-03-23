#!/bin/bash
# 从构建输出收集发布物，并按规范命名：KeVoiceInput-<version>-<platform>-<arch>.<ext>
# 输出到 release-out/ 目录，便于上传到 GitHub Release。
# 用法: ./scripts/release-artifacts.sh [输出目录，默认 release-out]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUT_DIR="${1:-$PROJECT_ROOT/release-out}"

CONFIG="$PROJECT_ROOT/src-tauri/tauri.conf.json"
BUNDLE_MACOS="$PROJECT_ROOT/src-tauri/target/release/bundle/macos"
DMG_DIR="$PROJECT_ROOT/src-tauri/target/release/bundle/dmg"

# 从 tauri.conf.json 读取版本号
VERSION=$(grep -o '"version": *"[^"]*"' "$CONFIG" | head -1 | sed 's/"version": *"\(.*\)"/\1/')
if [ -z "$VERSION" ]; then
  echo "错误: 无法从 $CONFIG 读取 version"
  exit 1
fi

# 检测当前平台与架构（用于本机构建产物）
get_platform() {
  case "$(uname -s)" in
    Darwin) echo "macos" ;;
    Linux) echo "linux" ;;
    MINGW*|MSYS*|CYGWIN*) echo "windows" ;;
    *) echo "unknown" ;;
  esac
}
get_arch() {
  case "$(uname -m)" in
    arm64) echo "aarch64" ;;
    x86_64|AMD64) echo "x86_64" ;;
    armv7*) echo "armv7" ;;
    i686|i386) echo "i686" ;;
    *) echo "$(uname -m)" ;;
  esac
}

PLATFORM=$(get_platform)
ARCH=$(get_arch)
PREFIX="KeVoiceInput-${VERSION}-${PLATFORM}-${ARCH}"

mkdir -p "$OUT_DIR"
COPIED=0

# macOS: .app.tar.gz + .sig, .dmg
if [ "$PLATFORM" = "macos" ]; then
  if [ -f "$BUNDLE_MACOS/KeVoiceInput.app.tar.gz" ]; then
    cp "$BUNDLE_MACOS/KeVoiceInput.app.tar.gz" "$OUT_DIR/${PREFIX}.app.tar.gz"
    echo "  ${PREFIX}.app.tar.gz"
    COPIED=$((COPIED+1))
  fi
  if [ -f "$BUNDLE_MACOS/KeVoiceInput.app.tar.gz.sig" ]; then
    cp "$BUNDLE_MACOS/KeVoiceInput.app.tar.gz.sig" "$OUT_DIR/${PREFIX}.app.tar.gz.sig"
    echo "  ${PREFIX}.app.tar.gz.sig"
    COPIED=$((COPIED+1))
  fi
  # DMG 可能由 wrapper 生成为规范名或旧名
  for dmg in "$DMG_DIR/${PREFIX}.dmg" "$DMG_DIR/KeVoiceInput_${VERSION}_${ARCH}.dmg" "$DMG_DIR/KeVoiceInput_"*".dmg"; do
    if [ -f "$dmg" ]; then
      cp "$dmg" "$OUT_DIR/${PREFIX}.dmg"
      echo "  ${PREFIX}.dmg"
      COPIED=$((COPIED+1))
      break
    fi
  done
fi

# Windows: .msi, .exe 等（后续 Tauri 会生成在 bundle/msi 等）
if [ "$PLATFORM" = "windows" ]; then
  for f in "$PROJECT_ROOT/src-tauri/target/release/bundle/msi/"*.msi \
           "$PROJECT_ROOT/src-tauri/target/release/bundle/nsis/"*.exe; do
    if [ -f "$f" ]; then
      ext="${f##*.}"
      cp "$f" "$OUT_DIR/${PREFIX}.${ext}"
      echo "  ${PREFIX}.${ext}"
      COPIED=$((COPIED+1))
    fi
  done
fi

# Linux: .deb, .AppImage, .rpm 等
if [ "$PLATFORM" = "linux" ]; then
  for dir in "$PROJECT_ROOT/src-tauri/target/release/bundle/deb" \
             "$PROJECT_ROOT/src-tauri/target/release/bundle/appimage"; do
    for f in "$dir"/*.deb "$dir"/*.AppImage "$dir"/*.rpm; do
      [ -f "$f" ] || continue
      ext="${f##*.}"
      cp "$f" "$OUT_DIR/${PREFIX}.${ext}"
      echo "  ${PREFIX}.${ext}"
      COPIED=$((COPIED+1))
    done
  done
fi

if [ $COPIED -eq 0 ]; then
  echo "未找到任何构建产物。请先执行: bun run tauri:build"
  exit 1
fi

echo ""
echo "已复制 $COPIED 个文件到: $OUT_DIR"
echo "命名规范: KeVoiceInput-<version>-<platform>-<arch>.<ext>"
echo "上传示例: gh release create v$VERSION $OUT_DIR/* --repo Thinkre/KeVoiceInput --title \"v$VERSION\""
