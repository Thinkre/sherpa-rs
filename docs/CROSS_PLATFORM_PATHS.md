# 跨平台路径处理验证

## 概述

KeVoiceInput 使用 Tauri 和 Rust 标准库提供的跨平台 API 进行路径处理，无需平台特定代码。

## 路径 API 使用情况

### 应用数据目录

**API**: `app.path().app_data_dir()`

**平台映射**:
- **macOS**: `~/Library/Application Support/com.kevoiceinput.app/`
- **Windows**: `%APPDATA%\com.kevoiceinput.app\`
- **Linux**: `~/.config/com.kevoiceinput.app/`

**使用位置**:
1. `src-tauri/src/commands/mod.rs:32-38` - `get_app_dir_path()`
2. `src-tauri/src/managers/model.rs:122-124` - `load_models_config()`
3. `src-tauri/src/managers/model.rs:230-234` - 模型目录
4. `src-tauri/src/managers/history.rs:69-85` - 历史记录管理器初始化
   - 录音文件目录: `app_data_dir/recordings/`
   - 数据库路径: `app_data_dir/history.db`

### 日志目录

**API**: `app.path().app_log_dir()`

**平台映射**:
- **macOS**: `~/Library/Logs/com.kevoiceinput.app/`
- **Windows**: `%APPDATA%\com.kevoiceinput.app\logs\`
- **Linux**: `~/.config/com.kevoiceinput.app/logs/`

**使用位置**:
1. `src-tauri/src/commands/mod.rs:55-62` - `get_log_dir_path()`

### 临时目录

**API**: `std::env::temp_dir()`

**平台映射**:
- **macOS**: `/tmp/` 或 `$TMPDIR`
- **Windows**: `%TEMP%\`
- **Linux**: `/tmp/` 或 `$TMPDIR`

**使用位置**:
1. `src-tauri/src/managers/transcription.rs:92` - 清理临时热词文件
2. `src-tauri/src/managers/transcription.rs:323` - 导出 BPE 词汇表脚本
3. `src-tauri/src/managers/transcription.rs:379` - 热词文件
4. `src-tauri/src/managers/transcription.rs:485-486` - 热词标记化脚本
5. `src-tauri/src/managers/transcription.rs:614` - SeaCo 推理脚本
6. `src-tauri/src/managers/transcription.rs:919` - Paraformer 热词文件
7. `src-tauri/src/managers/model.rs:889-966` - 模型解压临时目录

## 路径构建最佳实践

### ✅ 正确做法

```rust
// 使用 Tauri PathResolver
let app_data_dir = app.path().app_data_dir()?;
let models_dir = app_data_dir.join("models");

// 使用标准库
let temp_dir = std::env::temp_dir();
let temp_file = temp_dir.join("hotwords.txt");

// 使用 PathBuf 进行路径操作
use std::path::PathBuf;
let file_path = PathBuf::from(base_dir).join("subdir").join("file.txt");
```

### ❌ 错误做法

```rust
// 不要硬编码路径分隔符
let path = format!("{}\\models\\{}", base_dir, model_name); // Windows only!
let path = format!("{}/models/{}", base_dir, model_name); // Unix only!

// 不要硬编码平台特定路径
let app_data = "~/Library/Application Support/com.kevoiceinput.app"; // macOS only!
let app_data = "%APPDATA%\\com.kevoiceinput.app"; // Windows only!
```

## 验证方法

### 开发环境测试

**macOS**:
```bash
# 检查应用数据目录
ls -la ~/Library/Application\ Support/com.kevoiceinput.app/

# 检查日志目录
ls -la ~/Library/Logs/com.kevoiceinput.app/
```

**Windows**:
```powershell
# 检查应用数据目录
dir $env:APPDATA\com.kevoiceinput.app\

# 检查日志目录
dir $env:APPDATA\com.kevoiceinput.app\logs\
```

**Linux**:
```bash
# 检查应用数据目录
ls -la ~/.config/com.kevoiceinput.app/

# 检查日志目录
ls -la ~/.config/com.kevoiceinput.app/logs/
```

### 运行时验证

启动应用后，检查以下内容是否正确创建：

1. **应用数据目录结构**:
   ```
   app_data_dir/
   ├── models/           # 模型文件
   ├── recordings/       # 录音文件
   ├── history.db        # 历史记录数据库
   ├── models.toml       # 模型配置
   └── settings_store.json # 设置存储
   ```

2. **日志文件**:
   - 日志目录下应有应用日志文件

3. **临时文件**:
   - 临时文件应在系统临时目录中创建
   - 使用后应正确清理

## 跨平台兼容性状态

| 功能 | 状态 | 说明 |
|------|------|------|
| 应用数据目录 | ✅ 完全兼容 | 使用 Tauri PathResolver |
| 日志目录 | ✅ 完全兼容 | 使用 Tauri PathResolver |
| 模型存储 | ✅ 完全兼容 | 使用 app_data_dir + PathBuf |
| 历史记录存储 | ✅ 完全兼容 | 使用 app_data_dir + PathBuf |
| 录音文件存储 | ✅ 完全兼容 | 使用 app_data_dir + PathBuf |
| 临时文件 | ✅ 完全兼容 | 使用 std::env::temp_dir() |
| 路径分隔符 | ✅ 完全兼容 | 使用 PathBuf::join() |

## 结论

✅ **KeVoiceInput 的路径处理已经完全跨平台兼容**

- 所有路径都使用 Tauri 和 Rust 标准库的跨平台 API
- 没有硬编码的平台特定路径
- 使用 `PathBuf::join()` 自动处理路径分隔符
- 无需额外的平台特定代码或条件编译

**Windows 适配状态**: ✅ 不需要额外工作，已完全支持

## 相关文档

- [Tauri Path API 文档](https://docs.rs/tauri/latest/tauri/path/)
- [Rust std::path 文档](https://doc.rust-lang.org/std/path/)
- [Windows 路径约定](https://docs.microsoft.com/en-us/windows/win32/fileio/naming-a-file)
