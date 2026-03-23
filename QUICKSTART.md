# KeVoiceInput 快速开始指南

本文档帮助新贡献者快速上手 KeVoiceInput 项目。

## 🚀 5 分钟快速开始

### 1. 克隆仓库

```bash
git clone https://github.com/yourusername/KeVoiceInput.git
cd KeVoiceInput
```

### 2. 安装依赖

```bash
# 安装前端依赖（推荐使用 bun）
bun install
# 或使用 npm
npm install

# Rust 依赖会在构建时自动安装
```

### 3. 运行开发服务器

```bash
# 启动 Tauri 开发模式（包含热重载）
bun run tauri:dev
```

应用会自动启动并监听文件变化。

### 4. 开始开发

编辑前端代码：
- `src/` - React 组件和页面
- `src/stores/` - Zustand 状态管理

编辑后端代码：
- `src-tauri/src/` - Rust 代码
- `src-tauri/src/commands/` - Tauri 命令（IPC）
- `src-tauri/src/managers/` - 业务逻辑

保存后会自动重新编译和刷新。

## 📋 项目结构速览

```
KeVoiceInput/
├── src/                    # 前端 React/TypeScript 代码
│   ├── components/         # UI 组件
│   ├── stores/            # 状态管理
│   └── i18n/              # 国际化
├── src-tauri/             # 后端 Rust 代码
│   ├── src/
│   │   ├── commands/      # Tauri 命令
│   │   └── managers/      # 业务逻辑管理器
│   └── Cargo.toml
├── scripts/               # 构建和工具脚本
├── docs/                  # 文档
└── tests/                 # 测试
```

## 🔧 常用命令

### 开发命令

```bash
# 前端开发服务器（仅前端）
bun run dev

# Tauri 开发模式（前端 + 后端）
bun run tauri:dev

# 代码格式化
bun run format              # 格式化所有代码
bun run format:frontend     # 仅前端
bun run format:backend      # 仅后端

# 代码检查
bun run lint                # ESLint
bun run lint:fix            # 自动修复问题
```

### 构建命令

```bash
# 构建前端
bun run build

# 构建完整应用
bun run tauri:build

# 清理构建产物
cd src-tauri && cargo clean
```

### 版本管理

```bash
# 更新所有配置文件的版本号
./scripts/sync-version.sh 0.0.2
```

## 📚 关键文档

### 开始之前先读

1. **[README.md](README.md)** - 项目概述和功能介绍
2. **[CLAUDE.md](CLAUDE.md)** - 代码库架构和开发指南
3. **[CONTRIBUTING.md](CONTRIBUTING.md)** - 贡献规范

### 技术文档

- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)** - 系统架构详解
- **[API.md](docs/API.md)** - Tauri 命令 API 文档
- **[BUILD_GUIDE.md](docs/BUILD_GUIDE.md)** - 详细构建说明
- **[SHERPA_ONNX.md](docs/SHERPA_ONNX.md)** - sherpa-onnx 集成文档
- **[DEBUGGING.md](docs/DEBUGGING.md)** - 调试和故障排查

### 发布流程

- **[RELEASE_GUIDE.md](docs/RELEASE_GUIDE.md)** - 发布流程说明
- **[.github/workflows/README.md](.github/workflows/README.md)** - CI/CD 工作流

## 🐛 常见问题

### 构建失败

**问题**: 编译错误

**解决**:
```bash
# 清理并重新安装
rm -rf node_modules dist
bun install
cd src-tauri && cargo clean && cd ..
bun run tauri:build
```

### 端口被占用

**问题**: `Port 1420 is already in use`

**解决**:
```bash
# 使用清理脚本
bun run dev:clean

# 或手动清理端口
lsof -ti:1420 | xargs kill -9
```

### Tauri 命令未定义

**问题**: TypeScript 报错找不到 Tauri 命令

**解决**:
```bash
# 重新生成 bindings
cd src-tauri
cargo build
cd ..
# 检查 src/bindings.ts 是否更新
```

### macOS 权限问题

**问题**: 应用无法输入文本

**解决**:
- 系统偏好设置 → 隐私与安全性 → 辅助功能
- 添加 KeVoiceInput.app 到允许列表

## 🎯 第一个贡献

### 简单的起点

1. **修复文档错误**: 查找并修复文档中的错别字或过时信息
2. **添加翻译**: 为 `src/i18n/locales/` 添加新语言翻译
3. **改进 UI**: 优化现有组件的样式或交互
4. **修复小 bug**: 查看 [Issues](https://github.com/yourusername/KeVoiceInput/issues) 中标记为 `good first issue` 的问题

### 提交流程

```bash
# 1. 创建功能分支
git checkout -b feature/your-feature-name

# 2. 进行更改并测试
# ... 编辑代码 ...
bun run lint
bun run build

# 3. 提交更改
git add -A
git commit -m "feat: add your feature description"

# 4. 推送到你的 fork
git push origin feature/your-feature-name

# 5. 创建 Pull Request
# 在 GitHub 上打开 PR
```

### Commit 规范

遵循 [Conventional Commits](https://www.conventionalcommits.org/)：

- `feat:` - 新功能
- `fix:` - Bug 修复
- `docs:` - 文档更新
- `style:` - 代码格式（不影响逻辑）
- `refactor:` - 重构
- `test:` - 测试
- `chore:` - 维护任务

## 💡 开发技巧

### 热重载

- **前端**: 保存文件后自动刷新
- **后端**: Rust 代码更改需要重新编译（约 10-30 秒）

### 调试

**前端调试**:
- 开发模式下打开浏览器开发者工具（F12 或右键 → 检查）
- 查看 Console 和 Network 标签

**后端调试**:
```rust
// 添加日志
log::info!("Debug message: {:?}", variable);
log::error!("Error: {:?}", error);
```

查看日志输出在终端。

### 测试新功能

```bash
# 在开发模式测试
bun run tauri:dev

# 测试生产构建
bun run tauri:build
open src-tauri/target/release/bundle/macos/KeVoiceInput.app
```

## 🤝 获取帮助

- **文档**: 优先查阅 `docs/` 目录
- **Issues**: [GitHub Issues](https://github.com/yourusername/KeVoiceInput/issues)
- **讨论**: [GitHub Discussions](https://github.com/yourusername/KeVoiceInput/discussions)
- **代码指南**: 查看 [CLAUDE.md](CLAUDE.md) 了解代码库结构

## 📖 深入学习

准备好深入了解？查看这些资源：

1. **[Tauri 文档](https://tauri.app/)** - 了解 Tauri 框架
2. **[React 文档](https://react.dev/)** - 学习 React 基础
3. **[Rust 书籍](https://doc.rust-lang.org/book/)** - Rust 编程语言
4. **[sherpa-onnx](https://k2-fsa.github.io/sherpa/onnx/)** - 语音识别库

## ✅ 检查清单

开始开发前确保：

- [ ] 已安装 Node.js 18+ 和 Rust
- [ ] 已安装 Bun（推荐）或 npm
- [ ] 系统依赖已安装（macOS: Xcode CLI Tools）
- [ ] 成功运行 `bun run tauri:dev`
- [ ] 阅读了 README.md 和 CONTRIBUTING.md
- [ ] 了解了项目的 commit 规范

准备好了？开始贡献吧！🎉
