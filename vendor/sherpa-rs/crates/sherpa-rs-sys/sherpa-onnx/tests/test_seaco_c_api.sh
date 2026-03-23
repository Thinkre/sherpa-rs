#!/bin/bash

# Test script for SeACo-Paraformer C-API
# Usage: ./test_seaco_c_api.sh [wav_file] [hotwords_file]

set -e

# Model directory
MODEL_DIR="/Users/thinkre/Desktop/models/beike/seaco_paraformer.20250904.for_general.sherpa_onnx"

# Check if model files exist
if [ ! -f "${MODEL_DIR}/model.onnx" ]; then
    echo "Error: ${MODEL_DIR}/model.onnx not found!"
    exit 1
fi

if [ ! -f "${MODEL_DIR}/model_eb.onnx" ]; then
    echo "Error: ${MODEL_DIR}/model_eb.onnx not found!"
    exit 1
fi

if [ ! -f "${MODEL_DIR}/tokens.txt" ]; then
    echo "Error: ${MODEL_DIR}/tokens.txt not found!"
    exit 1
fi

echo "Model files check passed!"
echo "  Model: ${MODEL_DIR}/model.onnx"
echo "  Model EB: ${MODEL_DIR}/model_eb.onnx"
echo "  Tokens: ${MODEL_DIR}/tokens.txt"
echo ""

# Build directory
BUILD_DIR="../build"
if [ ! -d "${BUILD_DIR}" ]; then
    echo "Error: Build directory ${BUILD_DIR} not found!"
    echo "Please build sherpa-onnx first:"
    echo "  mkdir build && cd build"
    echo "  cmake .."
    echo "  make -j4"
    exit 1
fi

# Check if library exists
LIB_DIR="${BUILD_DIR}/lib"
HAS_SO=$(ls "${LIB_DIR}"/libsherpa-onnx-c-api.so 2>/dev/null | wc -l)
HAS_DYLIB=$(ls "${LIB_DIR}"/libsherpa-onnx-c-api.dylib 2>/dev/null | wc -l)
HAS_A=$(ls "${LIB_DIR}"/libsherpa-onnx-c-api.a 2>/dev/null | wc -l)

if [ "$HAS_SO" -eq 0 ] && [ "$HAS_DYLIB" -eq 0 ] && [ "$HAS_A" -eq 0 ]; then
    echo "Error: sherpa-onnx-c-api library not found in ${LIB_DIR}!"
    exit 1
fi

# Compile test program
echo "Compiling test program..."
SOURCE_FILE="../local/test_seaco_paraformer_c_api.c"
OUTPUT_FILE="${BUILD_DIR}/test_seaco_paraformer_c_api"

# Detect OS and library type
if [ "$HAS_DYLIB" -gt 0 ] || [ "$HAS_A" -gt 0 ]; then
    # macOS or static library
    if [ "$HAS_DYLIB" -gt 0 ]; then
        # Dynamic library on macOS
        RPATH_FLAG="-Wl,-rpath,${LIB_DIR}"
        LIB_FLAGS="-L${LIB_DIR} -lsherpa-onnx-c-api"
    else
        # Static library - need to link all dependencies
        RPATH_FLAG=""
        # Try to find required libraries
        ONNXRUNTIME_LIB=$(find "${BUILD_DIR}" -name "libonnxruntime*.a" -o -name "libonnxruntime*.dylib" | head -1)
        if [ -z "$ONNXRUNTIME_LIB" ]; then
            echo "Warning: onnxruntime library not found, trying system paths..."
            LIB_FLAGS="-L${LIB_DIR} -lsherpa-onnx-c-api -lonnxruntime"
        else
            LIB_DIR_ONNX=$(dirname "$ONNXRUNTIME_LIB")
            LIB_FLAGS="-L${LIB_DIR} -lsherpa-onnx-c-api -L${LIB_DIR_ONNX} -lonnxruntime"
        fi
    fi
else
    # Linux dynamic library
    RPATH_FLAG="-Wl,-rpath,${LIB_DIR}"
    LIB_FLAGS="-L${LIB_DIR} -lsherpa-onnx-c-api"
fi

echo "Using library flags: ${LIB_FLAGS}"

gcc -o "${OUTPUT_FILE}" "${SOURCE_FILE}" \
    -I../sherpa-onnx/c-api \
    ${LIB_FLAGS} \
    ${RPATH_FLAG} \
    -std=c99 -Wall -Wextra

if [ $? -ne 0 ]; then
    echo "Compilation failed!"
    exit 1
fi

echo "Compilation successful!"
echo ""

# Run test
WAV_FILE="${1:-}"
HOTWORDS_FILE="${2:-}"

if [ -z "${WAV_FILE}" ]; then
    echo "Usage: $0 <wav_file> [hotwords_file]"
    echo ""
    echo "Example:"
    echo "  $0 test.wav"
    echo "  $0 test.wav hotwords.txt"
    exit 1
fi

if [ ! -f "${WAV_FILE}" ]; then
    echo "Error: WAV file ${WAV_FILE} not found!"
    exit 1
fi

echo "Running test..."
echo "  WAV file: ${WAV_FILE}"
if [ -n "${HOTWORDS_FILE}" ]; then
    echo "  Hotwords file: ${HOTWORDS_FILE}"
    if [ ! -f "${HOTWORDS_FILE}" ]; then
        echo "Warning: Hotwords file ${HOTWORDS_FILE} not found, continuing without hotwords..."
        HOTWORDS_FILE=""
    fi
fi
echo ""

if [ -n "${HOTWORDS_FILE}" ]; then
    "${OUTPUT_FILE}" "${WAV_FILE}" "${HOTWORDS_FILE}"
else
    "${OUTPUT_FILE}" "${WAV_FILE}"
fi
