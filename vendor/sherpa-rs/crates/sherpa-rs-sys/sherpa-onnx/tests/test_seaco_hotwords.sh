#!/bin/bash

# Test script for SeACo-Paraformer with hotwords

model_dir=/Users/thinkre/Desktop/models/beike/seaco_paraformer.20250904.for_general.onnx

# Create hotwords file
cat > /tmp/hotwords.txt <<EOF
停滞
交易
情况
EOF

echo "Testing SeACo-Paraformer with hotwords..."
../build/bin/sherpa-onnx-offline \
    --tokens=${model_dir}/tokens.txt \
    --paraformer=${model_dir}/model.onnx \
    --paraformer-eb=${model_dir}/model_eb.onnx \
    --hotwords-file=/tmp/hotwords.txt \
    --debug=1 \
    /Users/thinkre/Desktop/open/FunASR_2601/audio/BAC009S0764W0121.wav

echo ""
echo "Hotwords file content:"
cat /tmp/hotwords.txt
