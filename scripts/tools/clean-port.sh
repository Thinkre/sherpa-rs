#!/bin/bash
# 清理端口 1420 的脚本
# 注意：此脚本只清理占用端口的进程，不会影响正在启动的进程

PORT=1420

echo "正在检查端口 $PORT..."

# 查找并终止占用端口的进程（只清理旧的进程）
PIDS=$(lsof -ti:$PORT 2>/dev/null)
if [ ! -z "$PIDS" ]; then
    echo "找到占用端口的进程: $PIDS"
    # 检查这些进程是否是 vite（通过命令行参数判断）
    for PID in $PIDS; do
        if ps -p $PID -o command= 2>/dev/null | grep -q "vite"; then
            echo "检测到 Vite 进程 $PID，跳过清理（可能是正在启动的进程）"
            continue
        fi
        # 只清理非 vite 进程
        kill -TERM $PID 2>/dev/null
        sleep 0.2
        if kill -0 $PID 2>/dev/null; then
            kill -9 $PID 2>/dev/null
        fi
    done
    echo "已清理非 Vite 进程"
else
    echo "端口 $PORT 未被占用"
fi

exit 0
