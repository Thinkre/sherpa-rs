#!/bin/bash
# 启动 Vite 并等待它完全准备好，然后在前台运行

# 输出到 stderr 以确保 Tauri 能看到
exec 2>&1

PORT=1420
MAX_WAIT=30
WAITED=0

echo "[DEBUG] Script started from: $(pwd)" >&2
echo "[DEBUG] Script path: $0" >&2
echo "[DEBUG] Starting Vite server..." >&2

# 启动 Vite（后台运行）
bun run dev &
VITE_PID=$!

echo "[DEBUG] Vite PID: $VITE_PID" >&2

# 等待 Vite 服务器完全启动
echo "[DEBUG] Waiting for Vite server to be ready..." >&2
while [ $WAITED -lt $MAX_WAIT ]; do
  if curl -s http://127.0.0.1:$PORT > /dev/null 2>&1; then
    echo "[DEBUG] Vite server is ready!" >&2
    # 额外等待一秒确保完全准备好
    sleep 1
    break
  fi
  sleep 0.5
  WAITED=$((WAITED + 1))
  if [ $((WAITED % 4)) -eq 0 ]; then
    echo "[DEBUG] Still waiting... ($WAITED/$MAX_WAIT)" >&2
  fi
done

if [ $WAITED -eq $MAX_WAIT ]; then
  echo "[DEBUG] ERROR: Vite server did not start within $MAX_WAIT seconds" >&2
  kill $VITE_PID 2>/dev/null || true
  exit 1
fi

echo "[DEBUG] Vite server confirmed ready, keeping process alive..." >&2

# 在前台保持进程运行（这样 Tauri 知道进程还在运行）
# 使用 trap 确保在脚本退出时清理进程
trap "echo '[DEBUG] Script received signal, cleaning up...' >&2; kill $VITE_PID 2>/dev/null || true; exit" EXIT INT TERM

# 等待 Vite 进程（在前台）
# 如果进程退出，脚本也会退出
wait $VITE_PID
EXIT_CODE=$?
echo "[DEBUG] Vite process exited with code: $EXIT_CODE" >&2
exit $EXIT_CODE
