#!/bin/bash
# 手动启动开发环境：先启动 Vite，然后启动 Tauri

set -e

echo "=== 手动启动开发环境 ==="
echo "步骤 1: 启动 Vite 服务器..."

# 清理端口
./scripts/clean-port.sh || true

# 启动 Vite（后台运行）
bun run dev &
VITE_PID=$!
echo "Vite PID: $VITE_PID"

# 等待 Vite 完全启动
echo "步骤 2: 等待 Vite 服务器启动..."
MAX_WAIT=30
WAITED=0
while [ $WAITED -lt $MAX_WAIT ]; do
  if curl -s http://127.0.0.1:1420 > /dev/null 2>&1; then
    echo "✓ Vite 服务器已启动并响应！"
    sleep 2  # 额外等待确保完全准备好
    break
  fi
  sleep 0.5
  WAITED=$((WAITED + 1))
  if [ $((WAITED % 4)) -eq 0 ]; then
    echo "等待中... ($WAITED/$MAX_WAIT)"
  fi
done

if [ $WAITED -eq $MAX_WAIT ]; then
  echo "✗ 错误: Vite 服务器在 $MAX_WAIT 秒后仍未启动"
  kill $VITE_PID 2>/dev/null || true
  exit 1
fi

# 验证 Vite 可以响应
echo "步骤 3: 验证 Vite 服务器..."
if curl -s http://127.0.0.1:1420 | grep -q "root"; then
  echo "✓ Vite 服务器响应正常"
else
  echo "✗ 警告: Vite 服务器响应异常"
fi

# 启动 Tauri（不使用 tauri dev，直接运行 cargo）
echo "步骤 4: 启动 Tauri 应用..."
cd src-tauri
cargo run --no-default-features --color always -- || {
  EXIT_CODE=$?
  echo "✗ Tauri 启动失败，退出码: $EXIT_CODE"
  kill $VITE_PID 2>/dev/null || true
  exit $EXIT_CODE
}

# 清理
echo "步骤 5: 清理..."
kill $VITE_PID 2>/dev/null || true
