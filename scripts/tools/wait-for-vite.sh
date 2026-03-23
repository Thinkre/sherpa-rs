#!/bin/bash
# 等待 Vite 服务器启动的脚本

PORT=1420
MAX_ATTEMPTS=30
ATTEMPT=0

echo "等待 Vite 服务器在端口 $PORT 上启动..."

while [ $ATTEMPT -lt $MAX_ATTEMPTS ]; do
  if curl -s http://localhost:$PORT > /dev/null 2>&1; then
    echo "Vite 服务器已启动！"
    exit 0
  fi
  ATTEMPT=$((ATTEMPT + 1))
  sleep 0.5
done

echo "警告: Vite 服务器在 $MAX_ATTEMPTS 次尝试后仍未响应"
exit 1
