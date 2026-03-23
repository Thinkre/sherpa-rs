#!/bin/bash
# 带日志的开发启动脚本

set -e

echo "=== 开始开发服务器启动 ==="
echo "时间: $(date)"
echo "工作目录: $(pwd)"

# 清理端口
echo "=== 清理端口 1420 ==="
./scripts/clean-port.sh || true

# 启动 Vite（后台运行）
echo "=== 启动 Vite 服务器 ==="
bun run dev &
VITE_PID=$!
echo "Vite PID: $VITE_PID"

# 等待 Vite 启动
echo "=== 等待 Vite 服务器启动 ==="
MAX_WAIT=30
WAITED=0
while [ $WAITED -lt $MAX_WAIT ]; do
  if curl -s http://127.0.0.1:1420 > /dev/null 2>&1; then
    echo "Vite 服务器已启动！"
    break
  fi
  sleep 1
  WAITED=$((WAITED + 1))
  echo "等待中... ($WAITED/$MAX_WAIT)"
done

if [ $WAITED -eq $MAX_WAIT ]; then
  echo "错误: Vite 服务器在 $MAX_WAIT 秒后仍未启动"
  kill $VITE_PID 2>/dev/null || true
  exit 1
fi

# 启动 Tauri
echo "=== 启动 Tauri ==="
cd src-tauri
cargo run --no-default-features --color always -- || {
  echo "Tauri 启动失败，退出码: $?"
  kill $VITE_PID 2>/dev/null || true
  exit 1
}

# 清理
kill $VITE_PID 2>/dev/null || true
