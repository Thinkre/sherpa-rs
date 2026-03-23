#!/bin/bash
# 测试使用 download-binaries feature（不使用本地库）

set -e

echo "=========================================="
echo "测试使用 download-binaries feature"
echo "=========================================="
echo ""

# 确保没有设置 SHERPA_LIB_PATH（这会覆盖 download-binaries）
if [ -n "$SHERPA_LIB_PATH" ]; then
    echo "⚠️  警告: SHERPA_LIB_PATH 已设置，会优先使用本地库"
    echo "   当前值: $SHERPA_LIB_PATH"
    echo "   正在取消设置..."
    unset SHERPA_LIB_PATH
fi

# 确保没有设置 DYLD_LIBRARY_PATH（macOS 动态库路径）
if [ -n "$DYLD_LIBRARY_PATH" ]; then
    echo "⚠️  警告: DYLD_LIBRARY_PATH 已设置"
    echo "   当前值: $DYLD_LIBRARY_PATH"
    echo "   正在取消设置..."
    unset DYLD_LIBRARY_PATH
fi

echo "✅ 环境变量已清理"
echo ""

# 检查 Cargo.toml 配置
echo "检查 Cargo.toml 配置..."
if grep -q 'features = \["download-binaries"\]' src-tauri/Cargo.toml; then
    echo "✅ Cargo.toml 已启用 download-binaries feature"
else
    echo "❌ 错误: Cargo.toml 未启用 download-binaries feature"
    exit 1
fi

echo ""
echo "=========================================="
echo "清理构建缓存（可选）"
echo "=========================================="
echo ""
read -p "是否清理 sherpa-rs-sys 的构建缓存？(y/N) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "清理构建缓存..."
    cd src-tauri
    cargo clean -p sherpa-rs-sys
    cargo clean -p sherpa-rs
    cd ..
    echo "✅ 构建缓存已清理"
else
    echo "跳过清理构建缓存"
fi

echo ""
echo "=========================================="
echo "开始构建（使用 download-binaries）"
echo "=========================================="
echo ""
echo "环境变量状态："
echo "  SHERPA_LIB_PATH=${SHERPA_LIB_PATH:-未设置}"
echo "  DYLD_LIBRARY_PATH=${DYLD_LIBRARY_PATH:-未设置}"
echo ""

# 检查参数
if [ "$1" == "build" ]; then
    echo "开始生产构建..."
    bun run tauri build
elif [ "$1" == "dev" ] || [ -z "$1" ]; then
    echo "开始开发模式..."
    bun run tauri dev
else
    echo "用法: $0 [dev|build]"
    echo "  dev  - 开发模式（默认）"
    echo "  build - 生产构建"
    exit 1
fi
