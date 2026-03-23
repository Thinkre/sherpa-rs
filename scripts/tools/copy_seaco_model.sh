#!/bin/bash
# 快速复制 SeACo Paraformer 模型到应用目录

SOURCE_DIR="$HOME/Desktop/models/beike/seaco_paraformer.20250904.for_general.onnx"
TARGET_DIR="$HOME/Library/Application Support/com.kevoiceinput.app/models/seaco-paraformer-beike"

echo "复制 SeACo Paraformer 模型..."
echo "源目录: $SOURCE_DIR"
echo "目标目录: $TARGET_DIR"
echo ""

# 检查源目录
if [ ! -d "$SOURCE_DIR" ]; then
    echo "错误: 源目录不存在: $SOURCE_DIR"
    exit 1
fi

# 创建目标目录
mkdir -p "$TARGET_DIR"

# 检查并创建 tokens.txt
if [ ! -f "$SOURCE_DIR/tokens.txt" ] && [ -f "$SOURCE_DIR/tokens.json" ]; then
    echo "创建 tokens.txt..."
    python3 << 'PYEOF'
import json
import sys

try:
    with open('$SOURCE_DIR/tokens.json', 'r', encoding='utf-8') as f:
        data = json.load(f)
    
    if isinstance(data, list):
        tokens = data
    elif isinstance(data, dict):
        tokens = data.get('token_list', data.get('tokens', []))
    else:
        tokens = []
    
    with open('$SOURCE_DIR/tokens.txt', 'w', encoding='utf-8') as f:
        for token in tokens:
            f.write(f"{token}\n")
    
    print(f"✓ 已创建 tokens.txt，包含 {len(tokens)} 个 tokens")
except Exception as e:
    print(f"错误: {e}")
    sys.exit(1)
PYEOF
fi

# 复制文件
echo "复制模型文件..."
cp "$SOURCE_DIR/model.onnx" "$TARGET_DIR/" && echo "  ✓ model.onnx"
[ -f "$SOURCE_DIR/tokens.txt" ] && cp "$SOURCE_DIR/tokens.txt" "$TARGET_DIR/" && echo "  ✓ tokens.txt"
[ -f "$SOURCE_DIR/am.mvn" ] && cp "$SOURCE_DIR/am.mvn" "$TARGET_DIR/" && echo "  ✓ am.mvn"

echo ""
echo "✓ 模型文件已复制完成！"
echo ""
echo "文件列表:"
ls -lh "$TARGET_DIR" | grep -E "model.onnx|tokens.txt|am.mvn" || true
