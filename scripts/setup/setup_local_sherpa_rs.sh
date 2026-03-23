#!/bin/bash
# 设置本地 sherpa-rs 依赖

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VENDOR_DIR="$PROJECT_ROOT/vendor"

echo "📦 设置本地 sherpa-rs 依赖"
echo "项目根目录: $PROJECT_ROOT"
echo "Vendor 目录: $VENDOR_DIR"

# 创建 vendor 目录
mkdir -p "$VENDOR_DIR"
cd "$VENDOR_DIR"

# 检查是否已经克隆
if [ -d "sherpa-rs" ]; then
    echo "⚠️  sherpa-rs 目录已存在，跳过克隆"
    echo "如果要重新克隆，请先删除: rm -rf $VENDOR_DIR/sherpa-rs"
else
    echo "📥 克隆 sherpa-rs..."
    git clone https://github.com/thewh1teagle/sherpa-rs.git
fi

cd sherpa-rs

# 检查最新版本
LATEST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.6.8")
echo "📌 当前版本: $LATEST_TAG"

# 切换到指定版本
echo "🔀 切换到版本: $LATEST_TAG"
git checkout "$LATEST_TAG" 2>/dev/null || git checkout -b "$LATEST_TAG" "$LATEST_TAG"

echo ""
echo "✅ sherpa-rs 已克隆到: $VENDOR_DIR/sherpa-rs"
echo ""
echo "下一步："
echo "1. 修改 vendor/sherpa-rs/sherpa-rs/src/paraformer.rs 添加 model_eb 字段"
echo "2. 更新 Cargo.toml 使用本地路径依赖"
echo "3. 运行: cargo build"
