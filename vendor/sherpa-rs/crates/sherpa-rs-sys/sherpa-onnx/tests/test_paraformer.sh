#!/bin/bash


# model_dir=models/sherpa-onnx-paraformer-zh-int8-2025-10-07

# ../build/bin/sherpa-onnx-offline \
#     --tokens=${model_dir}/tokens.txt \
#     --paraformer=${model_dir}/model.int8.onnx \
#     /Users/thinkre/Desktop/open/FunASR_2601/audio/BAC009S0764W0121.wav


model_dir=~/Desktop/models/beike/ke_paraformer_0902_onnx
../build/bin/sherpa-onnx-offline \
    --tokens=${model_dir}/tokens.txt \
    --paraformer=${model_dir}/model.onnx \
    /Users/thinkre/Desktop/open/FunASR_2601/audio/BAC009S0764W0121.wav


model_dir=/Users/thinkre/Desktop/models/beike/seaco_paraformer.20250904.for_general.onnx
../build/bin/sherpa-onnx-offline \
    --tokens=${model_dir}/tokens.txt \
    --paraformer=${model_dir}/model.onnx \
    /Users/thinkre/Desktop/open/FunASR_2601/audio/BAC009S0764W0121.wav