#!/bin/bash
# 将 release-out/ 中的规范化发布物上传到 GitHub Release。
# 需安装 gh 并已登录: https://cli.github.com/
# 若报 Invalid target_commitish：请先在仓库中至少有一次提交（如 git push -u origin main），再创建 Release。
#
# 用法:
#   ./scripts/upload-release-to-github.sh                    # 上传当前版本到 v$VERSION
#   ./scripts/upload-release-to-github.sh v0.0.2              # 指定 tag
#   ./scripts/upload-release-to-github.sh v0.0.2 release-out # 指定 tag 与目录
#   ./scripts/upload-release-to-github.sh --models-only       # 上传 release-out/models/ 内所有文件
#   ./scripts/upload-release-to-github.sh --models-only KeSeaCoParaformer.tar.bz2   # 上传指定文件
#   ./scripts/upload-release-to-github.sh --models-only dir/ file.tar.bz2           # 上传目录内容+文件

set -e

REPO="${GITHUB_REPO:-Thinkre/KeVoiceInput}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONFIG="$PROJECT_ROOT/src-tauri/tauri.conf.json"

# 解析参数
TAG=""
UPLOAD_DIR=""
MODELS_ONLY=false
MODEL_PATHS=()
for arg in "$@"; do
  case "$arg" in
    --models-only) MODELS_ONLY=true ;;
    v*.*.*) TAG="$arg" ;;
    *)
      if [ -d "$arg" ]; then
        [ "$MODELS_ONLY" = true ] && MODEL_PATHS+=("$arg") || UPLOAD_DIR="$arg"
      elif [ -f "$arg" ]; then
        [ "$MODELS_ONLY" = true ] && MODEL_PATHS+=("$arg")
      fi
      ;;
  esac
done

if [ -z "$UPLOAD_DIR" ]; then
  UPLOAD_DIR="$PROJECT_ROOT/release-out"
fi

if [ "$MODELS_ONLY" = true ]; then
  # 收集要上传的模型文件：显式传入的 或 默认 release-out/models/
  MODEL_FILES=()
  if [ ${#MODEL_PATHS[@]} -gt 0 ]; then
    for p in "${MODEL_PATHS[@]}"; do
      if [ -f "$p" ]; then
        MODEL_FILES+=("$p")
      elif [ -d "$p" ]; then
        for f in "$p"/*; do
          [ -f "$f" ] && MODEL_FILES+=("$f")
        done
      fi
    done
  else
    MODELS_DIR="$PROJECT_ROOT/release-out/models"
    if [ -d "$MODELS_DIR" ]; then
      for f in "$MODELS_DIR"/*; do
        [ -f "$f" ] && MODEL_FILES+=("$f")
      done
    fi
  fi
  if [ ${#MODEL_FILES[@]} -eq 0 ]; then
    echo "错误: 未找到要上传的模型文件。"
    echo "请指定文件或目录: ./scripts/upload-release-to-github.sh --models-only <文件或目录> ..."
    echo "或将模型放入: $PROJECT_ROOT/release-out/models/"
    exit 1
  fi
  echo "上传模型到 Release 'models'... (共 ${#MODEL_FILES[@]} 个文件)"
  TARGET_BRANCH="${GITHUB_RELEASE_TARGET:-main}"
  if gh release view "models" --repo "$REPO" &>/dev/null; then
    gh release upload "models" "${MODEL_FILES[@]}" --repo "$REPO" --clobber
    echo "已追加/覆盖到 release: models"
  else
    gh release create "models" "${MODEL_FILES[@]}" --repo "$REPO" --target "$TARGET_BRANCH" --title "Models" --notes "语音识别模型文件，供 KeVoiceInput 下载。"
    echo "已创建 release: models"
  fi
  exit 0
fi

# 应用发布：需要 tag
if [ -z "$TAG" ]; then
  VERSION=$(grep -o '"version": *"[^"]*"' "$CONFIG" | head -1 | sed 's/"version": *"\(.*\)"/\1/')
  TAG="v${VERSION}"
fi

if [ ! -d "$UPLOAD_DIR" ] || [ -z "$(ls -A "$UPLOAD_DIR" 2>/dev/null)" ]; then
  echo "目录为空或不存在: $UPLOAD_DIR"
  echo "请先执行: ./scripts/release-artifacts.sh [$UPLOAD_DIR]"
  exit 1
fi

echo "Repository: $REPO"
echo "Tag:        $TAG"
echo "Directory:  $UPLOAD_DIR"
echo ""

# 指定 target_commitish，避免空仓库或 API 使用默认分支时报 Invalid target_commitish
TARGET_BRANCH="${GITHUB_RELEASE_TARGET:-main}"

if gh release view "$TAG" --repo "$REPO" &>/dev/null; then
  echo "Release $TAG 已存在，上传附件..."
  gh release upload "$TAG" "$UPLOAD_DIR"/* --repo "$REPO" --clobber
  echo "已上传到 $TAG"
else
  echo "创建 Release $TAG 并上传 (target=$TARGET_BRANCH)..."
  gh release create "$TAG" "$UPLOAD_DIR"/* --repo "$REPO" --target "$TARGET_BRANCH" --title "$TAG" --notes "Release $TAG. 下载对应平台与架构的安装包。"
  echo "已创建并上传: $TAG"
fi

echo ""
echo "发布页: https://github.com/$REPO/releases/tag/$TAG"
