#!/bin/bash

# 修复编译错误 - 禁用 SeacoParaformer

echo "🔧 修复编译错误 - 禁用 SeacoParaformer"
echo ""

cd /Users/thinkre/Desktop/projects/KeVoiceInput/src-tauri/src/managers

# 1. 修复 mod.rs - 完全禁用 seaco_paraformer
echo "1. 修复 mod.rs..."
cat > mod.rs << 'EOF'
pub mod audio;
pub mod history;
pub mod model;
// 临时禁用 seaco_paraformer (崩溃问题)
// pub mod seaco_paraformer;
pub mod sherpa_debug;
pub mod transcription;
EOF
echo "✅ mod.rs 已更新"

# 2. 修复 transcription.rs 中的 import
echo "2. 修复 transcription.rs import..."
sed -i.fix1 's/^use crate::managers::seaco_paraformer.*/\/\/ DISABLED: use crate::managers::seaco_paraformer.../' transcription.rs
echo "✅ 已注释 seaco_paraformer import"

# 3. 编译测试
echo ""
echo "3. 编译测试..."
cd /Users/thinkre/Desktop/projects/KeVoiceInput/src-tauri
cargo build 2>&1 | tail -30

echo ""
echo "====================================="
echo "如果编译成功，你现在只能使用:"
echo "  ✅ Whisper"
echo "  ✅ Paraformer"
echo ""
echo "暂时不可用的模型:"
echo "  ❌ Transducer (会崩溃)"
echo "  ❌ FireRedAsr (会崩溃)"
echo "  ❌ SeacoParaformer (已禁用)"
echo "====================================="
