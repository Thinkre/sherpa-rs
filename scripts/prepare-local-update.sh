#!/bin/bash
# 构建后运行此脚本：将 macos 更新包复制到项目根，并生成带正确签名的 latest.json，便于本地 http 更新测试。
# 生产环境请改用 GitHub：运行 scripts/generate-latest-json.sh 并提交 latest.json，tauri.conf 中 endpoint 指向 raw.githubusercontent.com。
#
# 本地测试时需在 tauri.conf.json 中临时改回：
#   "endpoints": ["http://localhost:8765/latest.json"],
#   "dangerousInsecureTransportProtocol": true
#
# 完整流程（首次需生成密钥）：
#   1. 生成密钥: bun run tauri signer generate -w .tauri-updater.key
#   2. 将 .tauri-updater.key 中的 *公钥* 内容复制到 src-tauri/tauri.conf.json -> plugins.updater.pubkey（替换现有 pubkey）
#   3. 构建时传入私钥: export TAURI_SIGNING_PRIVATE_KEY="$(cat .tauri-updater.key)" && bun run tauri:build
#   4. 执行本脚本: ./scripts/prepare-local-update.sh
#   5. 启动本地服务: python3 -m http.server 8765
#   6. 打开应用，点击「有可用更新」->「更新」
#
# 若已有所需私钥（与 tauri.conf 中 pubkey 对应），只需执行 3、4、5、6。

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUNDLE_MACOS="$PROJECT_ROOT/src-tauri/target/release/bundle/macos"
TAR_GZ="KeVoiceInput.app.tar.gz"
SIG_FILE="${TAR_GZ}.sig"

if [ ! -f "$BUNDLE_MACOS/$TAR_GZ" ] || [ ! -f "$BUNDLE_MACOS/$SIG_FILE" ]; then
  echo "错误: 未找到 $BUNDLE_MACOS/$TAR_GZ 或 $BUNDLE_MACOS/$SIG_FILE"
  echo "请先设置签名密钥并构建："
  echo "  export TAURI_SIGNING_PRIVATE_KEY=\"\$(cat /path/to/your/private-key)\""
  echo "  bun run tauri:build"
  exit 1
fi

cp "$BUNDLE_MACOS/$TAR_GZ" "$PROJECT_ROOT/"
# 签名内容放入 JSON 时转义反斜杠和双引号
SIGNATURE=$(cat "$BUNDLE_MACOS/$SIG_FILE" | sed 's/\\/\\\\/g' | sed 's/"/\\"/g' | tr -d '\n')

# 从 tauri.conf.json 读版本，并生成比当前版本高的版本号（例如 0.0.1 -> 0.0.2）
CURRENT_VERSION=$(grep -o '"version": *"[^"]*"' "$PROJECT_ROOT/src-tauri/tauri.conf.json" | head -1 | sed 's/"version": *"\(.*\)"/\1/')
# 简单小版本 +1：0.0.1 -> 0.0.2
NEW_VERSION="0.0.2"
if [ -n "$CURRENT_VERSION" ]; then
  LAST_NUM=$(echo "$CURRENT_VERSION" | sed -n 's/.*\.\([0-9]*\)$/\1/p')
  if [ -n "$LAST_NUM" ]; then
    NEXT=$((LAST_NUM + 1))
    NEW_VERSION=$(echo "$CURRENT_VERSION" | sed "s/\.[0-9]*$/.$NEXT/")
  fi
fi

cat > "$PROJECT_ROOT/latest.json" << EOF
{
  "version": "$NEW_VERSION",
  "notes": "本地测试更新。",
  "pub_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "platforms": {
    "darwin-aarch64": {
      "url": "http://localhost:8765/$TAR_GZ",
      "signature": "$SIGNATURE"
    },
    "darwin-x86_64": {
      "url": "http://localhost:8765/$TAR_GZ",
      "signature": "$SIGNATURE"
    }
  }
}
EOF

echo "已复制 $TAR_GZ 到项目根，并生成 latest.json (version=$NEW_VERSION)。"
echo "请在本目录执行: python3 -m http.server 8765"
echo "然后打开应用，点击「有可用更新」->「更新」。"
