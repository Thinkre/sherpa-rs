#!/bin/bash
# Script to synchronize version across all configuration files
# Usage: ./scripts/sync-version.sh <version>
# Example: ./scripts/sync-version.sh 0.0.2

set -e

VERSION="$1"

# Validate arguments
if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version>"
  echo "Example: $0 0.0.2"
  exit 1
fi

# Validate version format (semantic versioning)
if ! echo "$VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-[a-z0-9.-]+)?(\+[a-z0-9.-]+)?$'; then
  echo "Error: Invalid version format. Use semantic versioning (e.g., 0.0.2 or 1.0.0-beta.1)"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Updating version to $VERSION..."
echo ""

# Update tauri.conf.json
if [ -f "$PROJECT_ROOT/src-tauri/tauri.conf.json" ]; then
  # macOS sed requires -i with extension, Linux doesn't
  if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" "$PROJECT_ROOT/src-tauri/tauri.conf.json"
  else
    sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" "$PROJECT_ROOT/src-tauri/tauri.conf.json"
  fi
  echo "✅ Updated src-tauri/tauri.conf.json"
else
  echo "⚠️  Warning: src-tauri/tauri.conf.json not found"
fi

# Update package.json
if [ -f "$PROJECT_ROOT/package.json" ]; then
  if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" "$PROJECT_ROOT/package.json"
  else
    sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" "$PROJECT_ROOT/package.json"
  fi
  echo "✅ Updated package.json"
else
  echo "⚠️  Warning: package.json not found"
fi

# Update Cargo.toml (only update the first version field in [package] section)
if [ -f "$PROJECT_ROOT/src-tauri/Cargo.toml" ]; then
  if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "0,/^version = \".*\"/{s/^version = \".*\"/version = \"$VERSION\"/}" "$PROJECT_ROOT/src-tauri/Cargo.toml"
  else
    sed -i "0,/^version = \".*\"/{s/^version = \".*\"/version = \"$VERSION\"/}" "$PROJECT_ROOT/src-tauri/Cargo.toml"
  fi
  echo "✅ Updated src-tauri/Cargo.toml"
else
  echo "⚠️  Warning: src-tauri/Cargo.toml not found"
fi

echo ""
echo "✅ Version updated to $VERSION"
echo ""
echo "Next steps:"
echo "  1. Update CHANGELOG.md with release notes"
echo "  2. Commit changes:"
echo "     git add -A"
echo "     git commit -m 'chore: bump version to $VERSION'"
echo "  3. Create and push tag:"
echo "     git tag -a v$VERSION -m 'Release $VERSION'"
echo "     git push origin main --tags"
echo "  4. Build release artifacts:"
echo "     bun run tauri:build"
