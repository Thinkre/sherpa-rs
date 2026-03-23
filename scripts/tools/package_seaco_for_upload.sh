#!/bin/bash
# 将本机已导入的 KeSeaCoParaformer（SeACo Paraformer）模型打包为 .tar.bz2，
# 便于上传到自建服务器后通过「链接下载」导入。
# 使用：在项目根目录执行 scripts/tools/package_seaco_for_upload.sh
# 可选：MODELS_DIR=/path/to/app/models OUT=./KeSeaCoParaformer.tar.bz2 scripts/tools/package_seaco_for_upload.sh

set -e

# 应用 models 目录（可通过环境变量覆盖）
if [ -n "$MODELS_DIR" ]; then
    APP_MODELS_DIR="$MODELS_DIR"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    APP_MODELS_DIR="$HOME/Library/Application Support/com.kevoiceinput.app/models"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    APP_MODELS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/com.kevoiceinput.app/models"
else
    APP_MODELS_DIR="$APPDATA/com.kevoiceinput.app/models"
fi

# 输出压缩包路径（默认当前目录）
OUT="${OUT:-./KeSeaCoParaformer.tar.bz2}"

echo "KeSeaCoParaformer 打包脚本"
echo "=========================="
echo "应用 models 目录: $APP_MODELS_DIR"
echo ""

if [ ! -d "$APP_MODELS_DIR" ]; then
    echo "错误: 目录不存在: $APP_MODELS_DIR"
    echo "请先在应用内导入 SeACo Paraformer 模型后再运行本脚本。"
    exit 1
fi

# 查找包含 model_eb.onnx 的子目录（即 SeACo 模型）
SEACO_DIR=""
for dir in "$APP_MODELS_DIR"/*; do
    if [ -d "$dir" ] && [ -f "$dir/model_eb.onnx" ] && [ -f "$dir/model.onnx" ]; then
        SEACO_DIR="$dir"
        break
    fi
done

if [ -z "$SEACO_DIR" ]; then
    echo "错误: 未在 $APP_MODELS_DIR 下找到 SeACo 模型目录（需同时包含 model.onnx 和 model_eb.onnx）。"
    echo "请先在应用内通过「从文件夹导入」导入 KeSeaCoParaformer 模型。"
    exit 1
fi

DIR_NAME=$(basename "$SEACO_DIR")
echo "找到 SeACo 模型目录: $DIR_NAME"
echo "打包中 -> $OUT"
echo ""

# 打包：压缩包内为单层目录，解压后应用可直接使用
tar -cjf "$OUT" -C "$APP_MODELS_DIR" "$DIR_NAME"

echo "✓ 打包完成: $OUT"
echo ""
echo "下一步："
echo "  1. 将 $OUT 上传到你的文件服务器（例如 http://47.252.72.77:8888/ 的某个路径）。"
echo "  2. 在 models.toml 中添加以下条目（将 URL 改为你上传后的实际下载地址）："
echo ""
cat << 'TOML'
[[local_models]]
id = "ke-seaco-paraformer"
name = "KeSeaCoParaformer"
description = "SeACo Paraformer，支持热词。通过自建链接下载。"
engine_type = "Paraformer"
filename = "KeSeaCoParaformer"
url = "http://47.252.72.77:8888/你的路径/KeSeaCoParaformer.tar.bz2"
size_mb = 150
is_directory = true
accuracy_score = 0.90
speed_score = 0.70
TOML
echo ""
echo "  3. 在应用内点击「重新加载配置」，然后在该模型上点击「下载」即可通过链接导入。"
echo ""
