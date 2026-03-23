#!/bin/bash
# Tauri post-bundle hook to copy dynamic libraries

echo "Running post-bundle hook..."

# 查找所有的 .app bundle
find "$TAURI_BUNDLE_TARGET_DIR" -name "*.app" -type d | while read -r app_bundle; do
    if [ -f "$app_bundle/Contents/MacOS/kevoiceinput" ]; then
        echo "Processing $app_bundle"
        /Users/thinkre/Desktop/projects/KeVoiceInput/scripts/copy-dylibs.sh "$app_bundle"
    fi
done

echo "Post-bundle hook completed"
