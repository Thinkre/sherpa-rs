#!/bin/bash
# Generate latest.json for Tauri auto-updater
# Supports multiple platforms (macOS, Windows, Linux)
# Automatically extracts release notes from CHANGELOG.md
#
# Usage:
#   ./scripts/generate-latest-json.sh [version]
# Examples:
#   ./scripts/generate-latest-json.sh 0.0.2
#   ./scripts/generate-latest-json.sh  # Uses version from tauri.conf.json

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONFIG="$PROJECT_ROOT/src-tauri/tauri.conf.json"
OUT_DIR="${RELEASE_OUT_DIR:-$PROJECT_ROOT/release-out}"
CHANGELOG="$PROJECT_ROOT/CHANGELOG.md"
REPO="${GITHUB_REPO:-yourusername/KeVoiceInput}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get version
VERSION="$1"
if [ -z "$VERSION" ]; then
  VERSION=$(grep -o '"version": *"[^"]*"' "$CONFIG" | head -1 | sed 's/"version": *"\(.*\)"/\1/')
fi
if [ -z "$VERSION" ]; then
  echo -e "${RED}Error: Please specify version or ensure version exists in tauri.conf.json${NC}"
  exit 1
fi

echo -e "${GREEN}Generating latest.json for version $VERSION${NC}"
echo ""

# GitHub Release base URL
BASE_URL="https://github.com/${REPO}/releases/download/v${VERSION}"

# Extract release notes from CHANGELOG.md
extract_release_notes() {
  if [ ! -f "$CHANGELOG" ]; then
    echo "Release $VERSION"
    return
  fi

  # Extract content between ## [$VERSION] and next ## [
  awk -v version="$VERSION" '
    BEGIN { found = 0; collecting = 0 }
    /^## \[/ {
      if (collecting) { exit }
      if (index($0, "[" version "]") > 0) {
        collecting = 1
        next
      }
    }
    collecting && /^### / { print $0; next }
    collecting && /^- / { print $0; next }
    collecting && /^$/ { next }
    collecting && /./ { print $0 }
  ' "$CHANGELOG" | sed 's/^/  /'  # Indent for JSON

  # If nothing found, provide default
  if [ "${PIPESTATUS[1]}" -ne 0 ] || [ -z "$(awk -v version="$VERSION" 'BEGIN{found=0}/^## \[/{if(index($0,"["version"]")>0){found=1}}END{print found}' "$CHANGELOG")" ]; then
    echo "  Release $VERSION"
  fi
}

RELEASE_NOTES=$(extract_release_notes)

# Validate and generate platform configurations
declare -A PLATFORMS
PLATFORM_JSON=""

# macOS platforms
for arch in aarch64 x86_64; do
  platform_key="darwin-${arch}"
  tar_gz="${PREFIX:-KeVoiceInput-${VERSION}}-macos-${arch}.app.tar.gz"
  sig_file="$OUT_DIR/$tar_gz.sig"

  if [ -f "$sig_file" ]; then
    signature=$(cat "$sig_file" | sed 's/\\/\\\\/g' | sed 's/"/\\"/g' | tr -d '\n')
    tar_gz_url="${BASE_URL}/$tar_gz"

    if [ -n "$PLATFORM_JSON" ]; then
      PLATFORM_JSON="${PLATFORM_JSON},"
    fi
    PLATFORM_JSON="${PLATFORM_JSON}
    \"${platform_key}\": {
      \"url\": \"${tar_gz_url}\",
      \"signature\": \"${signature}\"
    }"
    echo -e "  ${GREEN}✓${NC} Found signature for $platform_key"
  else
    echo -e "  ${YELLOW}⚠${NC}  No signature found for $platform_key (skipping)"
  fi
done

# Windows platforms
for arch in x86_64; do
  platform_key="windows-${arch}"
  msi="${PREFIX:-KeVoiceInput-${VERSION}}-windows-${arch}.msi"
  sig_file="$OUT_DIR/${msi}.zip.sig"

  if [ -f "$sig_file" ]; then
    signature=$(cat "$sig_file" | sed 's/\\/\\\\/g' | sed 's/"/\\"/g' | tr -d '\n')
    msi_url="${BASE_URL}/${msi}.zip"

    if [ -n "$PLATFORM_JSON" ]; then
      PLATFORM_JSON="${PLATFORM_JSON},"
    fi
    PLATFORM_JSON="${PLATFORM_JSON}
    \"${platform_key}\": {
      \"url\": \"${msi_url}\",
      \"signature\": \"${signature}\"
    }"
    echo -e "  ${GREEN}✓${NC} Found signature for $platform_key"
  else
    echo -e "  ${YELLOW}⚠${NC}  No signature found for $platform_key (skipping)"
  fi
done

# Linux platforms
for arch in x86_64; do
  platform_key="linux-${arch}"
  appimage="${PREFIX:-KeVoiceInput-${VERSION}}-linux-${arch}.AppImage"
  sig_file="$OUT_DIR/${appimage}.tar.gz.sig"

  if [ -f "$sig_file" ]; then
    signature=$(cat "$sig_file" | sed 's/\\/\\\\/g' | sed 's/"/\\"/g' | tr -d '\n')
    appimage_url="${BASE_URL}/${appimage}.tar.gz"

    if [ -n "$PLATFORM_JSON" ]; then
      PLATFORM_JSON="${PLATFORM_JSON},"
    fi
    PLATFORM_JSON="${PLATFORM_JSON}
    \"${platform_key}\": {
      \"url\": \"${appimage_url}\",
      \"signature\": \"${signature}\"
    }"
    echo -e "  ${GREEN}✓${NC} Found signature for $platform_key"
  else
    echo -e "  ${YELLOW}⚠${NC}  No signature found for $platform_key (skipping)"
  fi
done

# Check if at least one platform was found
if [ -z "$PLATFORM_JSON" ]; then
  echo -e "${RED}Error: No signature files found in $OUT_DIR${NC}"
  echo "Please run: ./scripts/release-artifacts.sh"
  exit 1
fi

# Generate latest.json
cat > "$PROJECT_ROOT/latest.json" << EOF
{
  "version": "$VERSION",
  "notes": "$(echo "$RELEASE_NOTES" | sed 's/"/\\"/g' | awk '{printf "%s\\n", $0}' | sed 's/\\n$//')",
  "pub_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "platforms": {${PLATFORM_JSON}
  }
}
EOF

echo ""
echo -e "${GREEN}✅ Successfully generated latest.json${NC}"
echo ""
echo "Version: $VERSION"
echo "Output: $PROJECT_ROOT/latest.json"
echo "Repository: $REPO"
echo ""
echo "Next steps:"
echo "  1. Verify latest.json content:"
echo "     cat latest.json"
echo "  2. Commit to repository:"
echo "     git add latest.json"
echo "     git commit -m 'chore: update latest.json for v$VERSION'"
echo "     git push origin main"
echo "  3. Or upload to update server if using external hosting"
