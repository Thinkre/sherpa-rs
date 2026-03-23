#!/bin/bash
# Compare ONNX model metadata using sherpa-onnx-offline tool

MODEL1="$1"
MODEL2="$2"
TOKENS1="$3"
TOKENS2="$4"

if [ -z "$MODEL1" ] || [ -z "$MODEL2" ]; then
    echo "Usage: $0 <model1.onnx> <model2.onnx> [tokens1.txt] [tokens2.txt]"
    exit 1
fi

echo "=================================================================================="
echo "  Comparing ONNX Model Structures"
echo "=================================================================================="
echo ""
echo "Model 1: $MODEL1"
echo "Model 2: $MODEL2"
echo ""

# Extract metadata from model 1
echo "=== Model 1 Metadata ==="
if [ -n "$TOKENS1" ]; then
    ../build/bin/sherpa-onnx-offline --tokens="$TOKENS1" --paraformer="$MODEL1" --debug=1 /dev/null 2>&1 | \
        grep -E "(vocab_size|lfr_window_size|lfr_window_shift|neg_mean|inv_stddev)" | head -5
else
    ../build/bin/sherpa-onnx-offline --paraformer="$MODEL1" --debug=1 /dev/null 2>&1 | \
        grep -E "(vocab_size|lfr_window_size|lfr_window_shift|neg_mean|inv_stddev)" | head -5
fi

echo ""
echo "=== Model 2 Metadata ==="
if [ -n "$TOKENS2" ]; then
    ../build/bin/sherpa-onnx-offline --tokens="$TOKENS2" --paraformer="$MODEL2" --debug=1 /dev/null 2>&1 | \
        grep -E "(vocab_size|lfr_window_size|lfr_window_shift|neg_mean|inv_stddev)" | head -5
else
    ../build/bin/sherpa-onnx-offline --paraformer="$MODEL2" --debug=1 /dev/null 2>&1 | \
        grep -E "(vocab_size|lfr_window_size|lfr_window_shift|neg_mean|inv_stddev)" | head -5
fi
