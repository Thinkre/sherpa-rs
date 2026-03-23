#!/bin/bash
# Compare ONNX model structures - comprehensive comparison

MODEL1="$1"
MODEL2="$2"
TOKENS1="$3"
TOKENS2="$4"

if [ -z "$MODEL1" ] || [ -z "$MODEL2" ]; then
    echo "Usage: $0 <model1.onnx> <model2.onnx> [tokens1.txt] [tokens2.txt]"
    exit 1
fi

echo "=================================================================================="
echo "  ONNX Model Structure Comparison"
echo "=================================================================================="
echo ""
echo "Model 1: $MODEL1"
echo "Model 2: $MODEL2"
echo ""

# Function to extract model info
extract_info() {
    local model="$1"
    local tokens="$2"
    
    if [ -n "$tokens" ]; then
        ../build/bin/sherpa-onnx-offline --tokens="$tokens" --paraformer="$model" --debug=1 /dev/null 2>&1
    else
        ../build/bin/sherpa-onnx-offline --paraformer="$model" --debug=1 /dev/null 2>&1
    fi
}

echo "=== Extracting Model 1 Information ==="
INFO1=$(extract_info "$MODEL1" "$TOKENS1" 2>&1)

echo ""
echo "=== Extracting Model 2 Information ==="
INFO2=$(extract_info "$MODEL2" "$TOKENS2" 2>&1)

echo ""
echo "=================================================================================="
echo "  METADATA COMPARISON"
echo "=================================================================================="
echo ""

# Compare metadata keys
echo "[Metadata Keys Present]"
echo "Model 1 metadata keys:"
echo "$INFO1" | grep -E "^[a-zA-Z_]+=" | cut -d= -f1 | sort | uniq
echo ""
echo "Model 2 metadata keys:"
echo "$INFO2" | grep -E "^[a-zA-Z_]+=" | cut -d= -f1 | sort | uniq

echo ""
echo "[Key Differences]"
KEYS1=$(echo "$INFO1" | grep -E "^[a-zA-Z_]+=" | cut -d= -f1 | sort | uniq)
KEYS2=$(echo "$INFO2" | grep -E "^[a-zA-Z_]+=" | cut -d= -f1 | sort | uniq)

echo "Keys only in Model 1:"
comm -23 <(echo "$KEYS1") <(echo "$KEYS2") | sed 's/^/  - /'

echo ""
echo "Keys only in Model 2:"
comm -13 <(echo "$KEYS1") <(echo "$KEYS2") | sed 's/^/  - /'

echo ""
echo "Common keys:"
comm -12 <(echo "$KEYS1") <(echo "$KEYS2") | sed 's/^/  - /'

echo ""
echo "=================================================================================="
echo "  DETAILED METADATA VALUES"
echo "=================================================================================="
echo ""

# Compare specific important keys
for key in vocab_size lfr_window_size lfr_window_shift; do
    val1=$(echo "$INFO1" | grep "^${key}=" | cut -d= -f2)
    val2=$(echo "$INFO2" | grep "^${key}=" | cut -d= -f2)
    
    if [ -z "$val1" ]; then
        val1="<not in metadata>"
    fi
    if [ -z "$val2" ]; then
        val2="<not in metadata>"
    fi
    
    marker="⚠️"
    if [ "$val1" = "$val2" ]; then
        marker="✓"
    fi
    
    printf "%s %-25s Model1: %-40s Model2: %-40s\n" "$marker" "$key" "$val1" "$val2"
done

echo ""
echo "[CMVN Parameters]"
echo "Model 1:"
echo "$INFO1" | grep -E "(neg_mean|inv_stddev|Loaded CMVN|am\.mvn)" | head -3
echo ""
echo "Model 2:"
echo "$INFO2" | grep -E "(neg_mean|inv_stddev|Loaded CMVN|am\.mvn)" | head -3
