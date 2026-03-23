#!/bin/bash
# KeVoiceInput 自动安装脚本
# 双击此文件即可自动安装应用

APP_NAME="KeVoiceInput.app"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_PATH="$SCRIPT_DIR/$APP_NAME"

# 设置终端颜色
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo ""
echo "════════════════════════════════════════════"
echo "   KeVoiceInput 自动安装程序"
echo "════════════════════════════════════════════"
echo ""

# 检查应用是否存在
if [ ! -d "$APP_PATH" ]; then
    echo -e "${RED}错误: 找不到 $APP_NAME${NC}"
    echo "请确保此脚本与应用在同一个 DMG 中"
    echo ""
    read -p "按任意键关闭..." -n1 -s
    exit 1
fi

# 询问安装位置
echo "请选择安装位置："
echo "  1) /Applications (所有用户可用，推荐)"
echo "  2) ~/Applications (仅当前用户)"
echo ""
read -p "请输入选项 [1]: " choice
choice=${choice:-1}

if [ "$choice" = "2" ]; then
    DEST_DIR="$HOME/Applications"
    mkdir -p "$DEST_DIR"
else
    DEST_DIR="/Applications"
fi

DEST_PATH="$DEST_DIR/$APP_NAME"

echo ""
echo -e "${BLUE}正在安装到: $DEST_DIR${NC}"

# 如果已存在，询问是否覆盖
if [ -d "$DEST_PATH" ]; then
    echo ""
    echo -e "${RED}警告: 应用已存在${NC}"
    read -p "是否覆盖现有版本? (y/n) [y]: " overwrite
    overwrite=${overwrite:-y}
    
    if [ "$overwrite" != "y" ] && [ "$overwrite" != "Y" ]; then
        echo "安装已取消"
        echo ""
        read -p "按任意键关闭..." -n1 -s
        exit 0
    fi
    
    echo "正在删除旧版本..."
    rm -rf "$DEST_PATH"
fi

# 复制应用
echo "正在复制应用文件..."
if cp -R "$APP_PATH" "$DEST_PATH"; then
    echo -e "${GREEN}✓ 复制完成${NC}"
else
    echo -e "${RED}✗ 复制失败${NC}"
    echo "可能需要管理员权限"
    echo ""
    read -p "是否使用管理员权限重试? (y/n) [y]: " use_sudo
    use_sudo=${use_sudo:-y}
    
    if [ "$use_sudo" = "y" ] || [ "$use_sudo" = "Y" ]; then
        echo "请输入密码："
        sudo cp -R "$APP_PATH" "$DEST_PATH"
        sudo chown -R $(whoami):staff "$DEST_PATH"
    else
        echo "安装已取消"
        echo ""
        read -p "按任意键关闭..." -n1 -s
        exit 1
    fi
fi

# 移除隔离属性（放行）
echo "正在移除隔离属性..."
if xattr -rd com.apple.quarantine "$DEST_PATH" 2>/dev/null; then
    echo -e "${GREEN}✓ 已放行，应用可直接启动${NC}"
else
    echo -e "${BLUE}ⓘ 无隔离属性或已放行${NC}"
fi

# 移除扩展属性中的其他限制
xattr -cr "$DEST_PATH" 2>/dev/null || true

echo ""
echo "════════════════════════════════════════════"
echo -e "${GREEN}   ✓ 安装完成！${NC}"
echo "════════════════════════════════════════════"
echo ""
echo "应用已安装到: $DEST_PATH"
echo ""
echo "启动方式："
echo "  • 在 Finder 中打开 $DEST_DIR"
echo "  • 在 Launchpad 中搜索 'KeVoice'"
echo "  • 使用 Spotlight 搜索"
echo ""

# 询问是否立即启动
read -p "是否立即启动应用? (y/n) [y]: " launch
launch=${launch:-y}

if [ "$launch" = "y" ] || [ "$launch" = "Y" ]; then
    echo "正在启动应用..."
    open "$DEST_PATH"
    echo -e "${GREEN}✓ 已启动${NC}"
fi

echo ""
echo "感谢使用 KeVoiceInput！"
echo ""
read -p "按任意键关闭..." -n1 -s
