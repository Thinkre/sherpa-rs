#!/bin/bash
# 诊断 sherpa-onnx 问题

set -e

echo "=========================================="
echo "Sherpa-ONNX 问题诊断"
echo "=========================================="
echo ""

# 1. 检查环境变量
echo "1. 检查环境变量："
echo "   SHERPA_LIB_PATH=${SHERPA_LIB_PATH:-未设置}"
echo "   DYLD_LIBRARY_PATH=${DYLD_LIBRARY_PATH:-未设置}"
echo ""

# 2. 检查 Cargo.toml 配置
echo "2. 检查 Cargo.toml 配置："
if grep -q 'features = \["download-binaries"\]' src-tauri/Cargo.toml; then
    echo "   ✅ download-binaries feature 已启用"
else
    echo "   ❌ download-binaries feature 未启用"
fi
echo ""

# 3. 检查已下载的库文件
echo "3. 检查已下载的库文件："
LIB_DIR="$HOME/.cargo/registry/cache/*/sherpa-rs-sys-*/out"
if [ -d "$LIB_DIR" ]; then
    echo "   找到库目录: $LIB_DIR"
    find "$LIB_DIR" -name "*.dylib" -o -name "*.so" -o -name "*.dll" 2>/dev/null | head -5
else
    echo "   ⚠️  未找到已下载的库文件"
fi
echo ""

# 4. 检查构建输出
echo "4. 检查最近的构建日志："
if [ -f "src-tauri/target/debug/build/sherpa-rs-sys-*/output" ]; then
    echo "   最近的构建输出："
    cat src-tauri/target/debug/build/sherpa-rs-sys-*/output 2>/dev/null | tail -20 || echo "   无法读取构建输出"
else
    echo "   ⚠️  未找到构建输出文件"
fi
echo ""

# 5. 建议
echo "=========================================="
echo "建议："
echo "=========================================="
echo ""
echo "1. 确保使用 download-binaries（不使用本地库）："
echo "   unset SHERPA_LIB_PATH"
echo "   unset DYLD_LIBRARY_PATH"
echo ""
echo "2. 清理并重新构建："
echo "   cd src-tauri"
echo "   cargo clean -p sherpa-rs-sys"
echo "   cargo clean -p sherpa-rs"
echo "   cd .."
echo "   bun run tauri dev"
echo ""
echo "3. 如果问题仍然存在，请提供："
echo "   - 完整的错误日志"
echo "   - 使用的具体模型类型"
echo "   - 崩溃时的堆栈跟踪"
echo ""
