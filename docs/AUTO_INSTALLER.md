# 自动安装脚本说明

## 概述

DMG 中包含的"安装到应用程序.command"脚本提供了一键自动安装体验，解决了 macOS 应用分发的几个常见问题。

## 解决的问题

### 1. 隔离属性（Quarantine）

**问题：**
从互联网下载的应用会被 macOS 添加隔离属性（com.apple.quarantine），导致：
- 首次启动时显示安全警告
- 需要右键点击选择"打开"
- 用户体验不佳

**解决方案：**
```bash
xattr -rd com.apple.quarantine "$DEST_PATH"
xattr -cr "$DEST_PATH"
```

脚本自动移除所有限制性扩展属性，应用可以像系统应用一样直接启动。

### 2. 安装位置选择

**问题：**
- /Applications 需要管理员权限（某些系统配置）
- ~/Applications 只对当前用户可用

**解决方案：**
脚本提供交互式选择：
1. /Applications（所有用户，推荐）
2. ~/Applications（仅当前用户）

如果 /Applications 需要权限，脚本会自动提示使用 sudo。

### 3. 版本更新

**问题：**
手动安装时，如果应用已存在：
- 需要手动删除旧版本
- 可能导致配置丢失

**解决方案：**
脚本自动：
- 检测已存在的版本
- 询问是否覆盖
- 保持用户数据目录完整

### 4. 用户体验

**传统安装：**
1. 打开 DMG
2. 拖动 app 到 Applications
3. 等待复制（无进度提示）
4. 首次启动时遇到安全警告
5. 需要右键选择"打开"
6. 再次确认打开

**一键安装：**
1. 打开 DMG
2. 双击安装脚本
3. 选择位置（1 或 2）
4. 完成！可直接启动

## 脚本特性

### 安全性

- ✅ 不需要禁用 Gatekeeper
- ✅ 不修改系统安全设置
- ✅ 只移除应用自身的隔离属性
- ✅ 需要时才请求管理员权限
- ✅ 开源可审查

### 交互性

- 🎨 彩色终端输出
- 📊 清晰的进度提示
- ❓ 交互式选择
- ⚠️ 明确的错误提示
- ✅ 成功/失败反馈

### 智能处理

```bash
# 检查应用是否存在
if [ ! -d "$APP_PATH" ]; then
    echo "错误: 找不到应用"
    exit 1
fi

# 处理已存在的版本
if [ -d "$DEST_PATH" ]; then
    read -p "是否覆盖? (y/n)" overwrite
    # ...
fi

# 智能权限处理
if cp -R "$APP_PATH" "$DEST_PATH"; then
    echo "复制完成"
else
    # 尝试使用 sudo
    sudo cp -R "$APP_PATH" "$DEST_PATH"
fi
```

## 技术细节

### 文件类型

使用 `.command` 扩展名：
- macOS 会将其识别为可执行脚本
- 双击时在 Terminal.app 中打开
- 默认图标是终端窗口
- 无需手动 chmod +x

### 路径处理

```bash
# 获取脚本所在目录（DMG 挂载点）
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# 构建应用路径
APP_PATH="$SCRIPT_DIR/$APP_NAME"
```

无论 DMG 挂载到哪里，脚本都能正确找到应用。

### 扩展属性

```bash
# 移除隔离属性
xattr -rd com.apple.quarantine "$DEST_PATH"

# 清除所有扩展属性
xattr -cr "$DEST_PATH"
```

`-r` = 递归处理所有文件
`-d` = 删除指定属性
`-c` = 清除所有扩展属性

### 权限处理

```bash
# 复制后修正所有权
sudo chown -R $(whoami):staff "$DEST_PATH"
```

如果使用 sudo 复制，文件所有者会是 root，需要改回当前用户。

## 自定义

### 修改安装位置

编辑 `scripts/installer-script.sh`：

```bash
# 只安装到 /Applications
DEST_DIR="/Applications"

# 或只安装到用户目录
DEST_DIR="$HOME/Applications"
mkdir -p "$DEST_DIR"
```

### 修改 DMG 布局

编辑 `scripts/create-dmg.sh`：

```bash
# 调整窗口大小
set the bounds of container window to {400, 100, 920, 500}

# 调整图标位置
set position of item "KeVoiceInput.app" to {130, 120}
set position of item "Applications" to {390, 120}
set position of item "安装到应用程序.command" to {260, 280}

# 调整图标大小
set icon size of viewOptions to 100
```

### 添加背景图片

```bash
# 在 create-dmg.sh 中添加
cp background.png "$DMG_DIR/.background/background.png"

# 在 AppleScript 中设置
set background picture of viewOptions to file ".background:background.png"
```

## 最佳实践

### 对于开发者

1. **测试安装脚本**
   ```bash
   # 测试脚本本身
   ./scripts/installer-script.sh
   
   # 测试 DMG
   open *.dmg
   # 双击安装脚本
   ```

2. **验证扩展属性**
   ```bash
   xattr -l /Applications/KeVoiceInput.app
   # 应该是空的或只有无害属性
   ```

3. **测试不同场景**
   - 全新安装
   - 覆盖安装
   - /Applications 和 ~/Applications
   - 有/无管理员权限

### 对于用户

1. **推荐使用一键安装**
   - 最简单快捷
   - 自动处理权限
   - 无需右键"打开"

2. **如果遇到问题**
   - 查看终端输出的错误信息
   - 确认有足够磁盘空间
   - 确认没有其他应用占用该名称

3. **卸载**
   - 从 Finder 删除应用即可
   - 用户数据在 ~/Library/Application Support/

## 安全考虑

### 为什么可以信任？

1. **脚本是开源的**
   - 源代码在 scripts/installer-script.sh
   - 可以审查每一行代码
   - 没有网络请求或隐藏操作

2. **不修改系统**
   - 只操作应用文件
   - 不修改系统设置
   - 不禁用安全功能

3. **标准 macOS 命令**
   - cp：复制文件
   - xattr：管理扩展属性
   - chown：修改所有权
   - open：启动应用

### 与 Gatekeeper 的关系

移除隔离属性 ≠ 绕过 Gatekeeper

- Gatekeeper：检查应用签名
- 隔离属性：额外的安全提示

我们的应用：
- 有有效的代码签名（adhoc 或 Developer ID）
- Gatekeeper 仍然会验证签名
- 只是跳过了"从互联网下载"的额外警告

## 参考资料

- [Apple Extended Attributes](https://developer.apple.com/library/archive/documentation/FileManagement/Conceptual/FileSystemProgrammingGuide/ExtendedAttributes/ExtendedAttributes.html)
- [xattr man page](https://ss64.com/osx/xattr.html)
- [Gatekeeper documentation](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)

## 常见问题

### Q: 为什么需要密码？

A: 只有在安装到 /Applications 且当前用户没有写权限时才需要。
   选择 ~/Applications 不需要密码。

### Q: 脚本会修改我的系统吗？

A: 不会。脚本只复制应用文件和移除应用的隔离属性。

### Q: 可以手动移除隔离属性吗？

A: 可以，在终端运行：
   ```bash
   xattr -cr /Applications/KeVoiceInput.app
   ```

### Q: 为什么用 .command 而不是 .sh？

A: .command 文件在 macOS 上可以直接双击执行，
   而 .sh 文件需要先设置可执行权限。

### Q: 可以自定义脚本吗？

A: 可以！修改 scripts/installer-script.sh，
   然后重新运行 bun run tauri:build。
