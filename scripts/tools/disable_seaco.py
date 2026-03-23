#!/usr/bin/env python3
"""禁用 SeacoParaformer - 注释掉所有相关代码"""

import re

file_path = "src-tauri/src/managers/transcription.rs"

with open(file_path, 'r') as f:
    lines = f.readlines()

# 需要注释的代码块范围
blocks_to_comment = [
    (761, 764),   # LoadedEngine::SeacoParaformer in Drop
    (1723, 1831), # EngineType::SeacoParaformer
    (2295, 2313), # LoadedEngine::SeacoParaformer in transcribe
]

# 注释掉指定范围的代码块
for start, end in blocks_to_comment:
    for i in range(start - 1, min(end, len(lines))):
        if not lines[i].strip().startswith('//'):
            lines[i] = '// DISABLED_SEACO: ' + lines[i]

# 写回文件
with open(file_path, 'w') as f:
    f.writelines(lines)

print("✅ transcription.rs 已更新 - SeacoParaformer 代码已注释")
