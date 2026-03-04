# KeVoiceInput

KeVoiceInput 是一款功能强大的桌面语音输入应用，支持多种语音识别引擎，提供高质量的语音转文字功能。

## 项目简介

KeVoiceInput 是一个基于 Tauri 框架构建的跨平台桌面应用，集成了多种先进的语音识别引擎，包括 Whisper、Sherpa-ONNX、Paraformer 和 SeACo Paraformer。应用提供了直观的用户界面和丰富的配置选项，让语音输入变得简单高效。

## 主要特性

### 核心功能

- **多引擎语音识别**：支持 Whisper、Sherpa-ONNX、Paraformer、SeACo Paraformer 等多种识别引擎
- **实时语音转录**：支持实时语音输入和转录，提供流畅的使用体验
- **模型管理**：支持模型的下载、删除、切换和管理
- **历史记录**：自动保存转录历史，支持查看、搜索和管理
- **快捷键支持**：可自定义快捷键，快速启动语音输入
- **LLM 后处理**：支持使用 LLM 对转录结果进行后处理和优化
- **多语言支持**：支持 14 种语言的界面（中文、英文、日文、西班牙文、法文、德文、越南文、波兰文、意大利文、俄文、乌克兰文、葡萄牙文、捷克文、土耳其文）

### 高级功能

- **热词支持**：SeACo Paraformer 引擎支持热词功能，提高特定词汇识别准确率
- **标点符号自动添加**：使用 AI 模型自动为转录文本添加标点符号，提高文本可读性
- **音频设备管理**：支持选择和管理输入/输出音频设备
- **录音保留**：可配置录音文件的保留策略
- **自定义词典**：支持添加自定义词汇，使用模糊匹配和语音匹配提高识别准确率
- **ITN（逆文本规范化）**：支持中文数字、日期、时间、货币等文本的规范化处理
- **智能文本过滤**：自动移除填充词（如 "uh", "um"）和处理口吃重复
- **剪贴板集成**：支持自动将转录结果复制到剪贴板
- **自动启动**：支持系统启动时自动运行
- **托盘图标**：提供系统托盘图标，方便快速访问

## 技术栈

### 前端

- **React 18.3.1** - UI 框架
- **TypeScript 5.6.3** - 类型安全的 JavaScript
- **Tailwind CSS 4.1.16** - 实用优先的 CSS 框架
- **Vite 6.4.1** - 快速的前端构建工具
- **Zustand 5.0.8** - 轻量级状态管理
- **i18next** - 国际化支持
- **Lucide React** - 图标库

### 后端

- **Rust** - 系统编程语言
- **Tauri 2.9.1** - 轻量级桌面应用框架
- **sherpa-rs** - 基于 Sherpa-ONNX 的语音识别库
- **whisper-rs** - Whisper 模型的 Rust 绑定
- **transcribe-rs** - 转录引擎封装

### 语音识别引擎

- **sherpa-rs**: 基于 Sherpa-ONNX 的语音识别库，支持多种模型格式
- **whisper-rs**: Whisper 模型的 Rust 绑定
- **Paraformer**: 标准 Paraformer 模型（需要 model.onnx + tokens.txt）
- **SeACo Paraformer**: Paraformer 的增强版本，支持热词（hotword）功能（需要 model.onnx + tokens.txt + model_eb.onnx）
  - SeACo Paraformer 是贝壳（FunASR）开发的变种，通过 model_eb.onnx 提供热词支持
  - 虽然底层使用相同的 ParaformerRecognizer，但作为独立的引擎类型处理，因为功能特性不同

### 音频处理

- **cpal 0.16.0** - 跨平台音频库
- **rodio** - Rust 音频播放库
- **rubato 0.16.2** - 音频重采样库
- **vad-rs** - 语音活动检测

### 文本处理

- **natural** - 自然语言处理库（Soundex 语音匹配）
- **strsim** - 字符串相似度计算（Levenshtein 距离）
- **regex** - 正则表达式处理
- **ferrous-opencc** - 中文简繁转换

### 其他依赖

- **rusqlite** - SQLite 数据库绑定
- **reqwest** - HTTP 客户端
- **tokio** - 异步运行时
- **enigo** - 键盘和鼠标模拟

## 系统要求

### 最低要求

- **操作系统**：
  - macOS 10.13 或更高版本
  - Windows 10 或更高版本
  - Linux（支持主流发行版）
- **内存**：4GB RAM（推荐 8GB 或更多）
- **存储空间**：至少 2GB 可用空间（用于模型文件）
- **音频设备**：麦克风（内置或外置）

### 推荐配置

- **内存**：8GB 或更多
- **存储空间**：10GB 或更多（用于多个模型）
- **CPU**：多核处理器（用于更快的转录速度）

## 安装

### 从源码构建

#### 前置要求

1. **Node.js** 和 **npm** 或 **bun**（推荐使用 bun）
2. **Rust** 工具链（最新稳定版）
3. **系统依赖**：
   - macOS: Xcode Command Line Tools
   - Linux: 开发工具和库
   - Windows: Visual Studio Build Tools

#### 构建步骤

1. 克隆仓库：
```bash
git clone https://github.com/thinkre/KeVoiceInput.git
cd KeVoiceInput
```

2. 安装前端依赖：
```bash
bun install
# 或
npm install
```

3. 构建应用：
```bash
# 开发模式
bun run tauri:dev

# 生产构建
bun run tauri build
```

### 从预编译版本安装

请访问 [Releases](https://github.com/thinkre/KeVoiceInput/releases) 页面下载对应平台的安装包。

## 快速开始

### 首次使用

1. **启动应用**：运行 KeVoiceInput 应用

2. **授予权限**：
   - macOS: 授予辅助功能权限（用于键盘输入）
   - Windows/Linux: 确保应用有麦克风访问权限

3. **下载模型**：
   - 在"模型"页面选择一个推荐的模型（如 Whisper Small）
   - 点击"下载"按钮，等待下载完成

4. **开始使用**：
   - 使用默认快捷键（可在设置中修改）启动语音输入
   - 说话后，转录结果会自动输入到当前活动窗口

### 基本使用

1. **启动语音输入**：
   - 按下快捷键（默认：根据操作系统不同）
   - 或点击托盘图标中的"开始转录"

2. **说话**：
   - 对着麦克风说话
   - 应用会实时显示转录结果

3. **完成转录**：
   - 再次按下快捷键停止
   - 转录结果会自动输入到当前活动窗口

## 使用说明

### 模型管理

#### 下载模型

1. 打开"模型"页面
2. 浏览可用模型列表
3. 点击"下载"按钮开始下载
4. 等待下载完成（可在首页查看进度）

#### 切换模型

1. 在"模型"页面选择已下载的模型
2. 点击"使用"按钮激活模型
3. 模型会自动加载（首次加载可能需要一些时间）

#### 删除模型

1. 在"模型"页面找到要删除的模型
2. 点击"删除"按钮
3. 确认删除操作

### 设置配置

#### 音频设置

- **麦克风选择**：选择用于录音的麦克风设备
- **输出设备**：选择用于播放反馈声音的设备
- **始终开启麦克风**：启用后，麦克风会持续监听（消耗更多资源）

#### 转录设置

- **语言选择**：选择识别语言（支持自动检测）
- **ITN 启用**：启用逆文本规范化（中文数字、日期、时间、货币等）
- **标点符号**：启用自动标点符号添加功能
- **自定义词汇**：添加自定义词汇，使用模糊匹配和语音匹配提高识别准确率
- **热词**：为 SeACo Paraformer 模型添加热词，或使用热词规则进行文本替换
- **文本过滤**：自动移除填充词（如 "uh", "um"）和处理口吃重复

#### 快捷键设置

- **转录快捷键**：设置启动/停止转录的快捷键
- **其他快捷键**：可自定义其他功能的快捷键

#### LLM 后处理

- **启用后处理**：使用 LLM 对转录结果进行优化
- **选择提供商**：选择 LLM 服务提供商（OpenAI、Anthropic 等）
- **配置 API**：设置 API Key 和模型

### 历史记录

- **查看历史**：在"历史记录"页面查看所有转录记录
- **保存记录**：标记重要的转录记录
- **删除记录**：删除不需要的记录
- **播放录音**：播放原始录音文件

## 开发指南

### 项目结构

```
KeVoiceInput/
├── src/                    # 前端源代码
│   ├── components/         # React 组件
│   │   ├── pages/          # 页面组件
│   │   ├── settings/       # 设置组件
│   │   └── ui/             # UI 基础组件
│   ├── stores/            # Zustand 状态管理
│   ├── hooks/              # React Hooks
│   ├── i18n/               # 国际化资源（支持 13+ 种语言）
│   └── lib/                # 工具库
├── src-tauri/              # Rust 后端源代码
│   ├── src/
│   │   ├── commands/       # Tauri 命令接口
│   │   ├── managers/       # 业务逻辑管理器
│   │   │   ├── model.rs    # 模型管理器
│   │   │   ├── transcription.rs # 转录管理器
│   │   │   ├── audio.rs    # 音频录制管理器
│   │   │   ├── history.rs  # 历史记录管理器
│   │   │   └── punctuation.rs # 标点符号管理器
│   │   ├── audio_toolkit/ # 音频和文本处理工具
│   │   │   ├── audio/      # 音频录制和播放
│   │   │   ├── text/       # 文本处理（ITN、自定义词汇等）
│   │   │   └── vad/        # 语音活动检测
│   │   └── main.rs         # 应用入口
│   └── Cargo.toml          # Rust 依赖配置
├── scripts/                # 构建和工具脚本
├── vendor/                 # 第三方依赖（sherpa-rs 等）
└── docs/                   # 文档
```

### 开发环境设置

1. **安装 Rust**：
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. **安装 Node.js 工具**：
```bash
# 使用 bun（推荐）
curl -fsSL https://bun.sh/install | bash

# 或使用 npm
npm install -g npm
```

3. **安装系统依赖**：
   - macOS: `xcode-select --install`
   - Linux: 安装开发工具包
   - Windows: 安装 Visual Studio Build Tools

### 运行开发服务器

```bash
# 启动前端开发服务器
bun run dev

# 启动 Tauri 开发模式（包含前端）
bun run tauri:dev
```

### 代码格式化

```bash
# 格式化所有代码
bun run format

# 检查格式
bun run format:check

# 仅格式化前端
bun run format:frontend

# 仅格式化后端
bun run format:backend
```

### 代码检查

```bash
# 运行 ESLint
bun run lint

# 自动修复问题
bun run lint:fix
```

### 构建

```bash
# 构建前端
bun run build

# 构建 Tauri 应用
bun run tauri build
```

## 架构文档

详细的架构说明请参考 [ARCHITECTURE.md](docs/ARCHITECTURE.md)。

## API 文档

完整的 API 接口文档请参考 [API.md](docs/API.md)。

## 贡献指南

我们欢迎所有形式的贡献！请遵循以下步骤：

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

### 代码规范

- 遵循项目的代码格式化规则
- 添加必要的注释和文档
- 确保代码通过所有检查
- 编写或更新相关测试

## 许可证

本项目采用 MIT 许可证。详情请参阅 [LICENSE](LICENSE) 文件。

## 作者

**thinkre**

## 致谢

- [Tauri](https://tauri.app/) - 优秀的桌面应用框架
- [Sherpa-ONNX](https://github.com/k2-fsa/sherpa-onnx) - 强大的语音识别库
- [Whisper](https://github.com/openai/whisper) - OpenAI 的语音识别模型
- [FunASR](https://github.com/alibaba-damo-academy/FunASR) - 阿里达摩院的语音识别框架

## 相关链接

- [问题反馈](https://github.com/thinkre/KeVoiceInput/issues)
- [功能请求](https://github.com/thinkre/KeVoiceInput/issues)
- [讨论区](https://github.com/thinkre/KeVoiceInput/discussions)
