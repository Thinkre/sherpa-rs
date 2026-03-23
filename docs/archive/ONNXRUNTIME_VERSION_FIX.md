# onnxruntime 版本号修复记录

## 修复日期
2026-02-27

## 问题描述

虽然我们已经升级到 onnxruntime 1.23.2，但是库文件名和依赖路径还在使用 `libonnxruntime.1.17.1.dylib`：

**之前的状态**：
```bash
# 应用包中的文件
/Applications/KeVoiceInput.app/Contents/Frameworks/
└── libonnxruntime.1.17.1.dylib  # 文件名是 1.17.1，但内容是 1.23.2

# 依赖路径
kevoiceinput -> @executable_path/../Frameworks/libonnxruntime.1.17.1.dylib
libsherpa-onnx-c-api.dylib -> @loader_path/libonnxruntime.1.17.1.dylib
libsherpa-onnx-cxx-api.dylib -> @loader_path/libonnxruntime.1.17.1.dylib
```

**问题**：
- 文件名和版本号不一致，造成混淆
- 不符合语义化版本规范
- 难以追踪实际使用的库版本

## 修复方案

### 1. 修改已安装应用的库路径

使用 `install_name_tool` 修改所有依赖路径：

```bash
# 1. 创建新版本库文件
cd /Applications/KeVoiceInput.app/Contents/Frameworks
cp libonnxruntime.1.17.1.dylib libonnxruntime.1.23.2.dylib

# 2. 修改主程序依赖
cd /Applications/KeVoiceInput.app/Contents/MacOS
install_name_tool -change \
  "@executable_path/../Frameworks/libonnxruntime.1.17.1.dylib" \
  "@executable_path/../Frameworks/libonnxruntime.1.23.2.dylib" \
  kevoiceinput

# 3. 修改 sherpa-onnx C API 依赖
cd /Applications/KeVoiceInput.app/Contents/Frameworks
install_name_tool -change \
  "@loader_path/libonnxruntime.1.17.1.dylib" \
  "@loader_path/libonnxruntime.1.23.2.dylib" \
  libsherpa-onnx-c-api.dylib

# 4. 修改 sherpa-onnx C++ API 依赖
install_name_tool -change \
  "@loader_path/libonnxruntime.1.17.1.dylib" \
  "@loader_path/libonnxruntime.1.23.2.dylib" \
  libsherpa-onnx-cxx-api.dylib

# 5. 修改 onnxruntime 库自身 ID
install_name_tool -id \
  "@loader_path/libonnxruntime.1.23.2.dylib" \
  libonnxruntime.1.23.2.dylib

# 6. 重新签名
codesign --force --sign - libonnxruntime.1.23.2.dylib
codesign --force --sign - libsherpa-onnx-c-api.dylib
codesign --force --sign - libsherpa-onnx-cxx-api.dylib
codesign --force --sign - ../MacOS/kevoiceinput
codesign --force --deep --sign - /Applications/KeVoiceInput.app

# 7. 删除旧文件
rm libonnxruntime.1.17.1.dylib
```

### 2. 修改构建脚本

修改 `scripts/copy-dylibs.sh` 确保以后构建时使用正确的版本号：

**修改内容**：

1. **第 18 行** - 复制库列表：
```bash
# 修改前
"libonnxruntime.1.17.1.dylib"

# 修改后
"libonnxruntime.1.23.2.dylib"
```

2. **第 54 行** - sherpa-onnx C API 依赖：
```bash
# 修改前
install_name_tool -change "@rpath/libonnxruntime.1.23.2.dylib" "@loader_path/libonnxruntime.1.17.1.dylib" ...

# 修改后
install_name_tool -change "@rpath/libonnxruntime.1.23.2.dylib" "@loader_path/libonnxruntime.1.23.2.dylib" ...
```

3. **第 63 行** - sherpa-onnx C++ API 依赖：
```bash
# 修改前
install_name_tool -change "@rpath/libonnxruntime.1.23.2.dylib" "@loader_path/libonnxruntime.1.17.1.dylib" ...

# 修改后
install_name_tool -change "@rpath/libonnxruntime.1.23.2.dylib" "@loader_path/libonnxruntime.1.23.2.dylib" ...
```

4. **第 71-74 行** - onnxruntime 库 ID：
```bash
# 修改前
if [ -f "$FRAMEWORKS_DIR/libonnxruntime.1.17.1.dylib" ]; then
    echo "  - Fixing libonnxruntime.1.17.1.dylib"
    install_name_tool -id "@loader_path/libonnxruntime.1.17.1.dylib" "$FRAMEWORKS_DIR/libonnxruntime.1.17.1.dylib" 2>/dev/null || true
fi

# 修改后
if [ -f "$FRAMEWORKS_DIR/libonnxruntime.1.23.2.dylib" ]; then
    echo "  - Fixing libonnxruntime.1.23.2.dylib"
    install_name_tool -id "@loader_path/libonnxruntime.1.23.2.dylib" "$FRAMEWORKS_DIR/libonnxruntime.1.23.2.dylib" 2>/dev/null || true
fi
```

## 验证结果

### 修复后的状态

```bash
# 应用包中的文件
/Applications/KeVoiceInput.app/Contents/Frameworks/
└── libonnxruntime.1.23.2.dylib  # ✅ 文件名和版本号一致

# 依赖路径
kevoiceinput:
  @executable_path/../Frameworks/libonnxruntime.1.23.2.dylib ✅

libsherpa-onnx-c-api.dylib:
  @loader_path/libonnxruntime.1.23.2.dylib ✅

libsherpa-onnx-cxx-api.dylib:
  @loader_path/libonnxruntime.1.23.2.dylib ✅

libonnxruntime.1.23.2.dylib:
  @loader_path/libonnxruntime.1.23.2.dylib ✅
```

### 验证命令

```bash
# 查看库文件
ls -lh /Applications/KeVoiceInput.app/Contents/Frameworks/libonnxruntime*

# 查看主程序依赖
otool -L /Applications/KeVoiceInput.app/Contents/MacOS/kevoiceinput | grep onnx

# 查看 sherpa-onnx C API 依赖
otool -L /Applications/KeVoiceInput.app/Contents/Frameworks/libsherpa-onnx-c-api.dylib | grep onnx

# 查看 sherpa-onnx C++ API 依赖
otool -L /Applications/KeVoiceInput.app/Contents/Frameworks/libsherpa-onnx-cxx-api.dylib | grep onnx

# 查看 onnxruntime 库 ID
otool -L /Applications/KeVoiceInput.app/Contents/Frameworks/libonnxruntime.1.23.2.dylib | grep onnx

# 测试应用启动
open /Applications/KeVoiceInput.app
ps aux | grep "[k]evoiceinput"
```

### 验证结果

```
✅ 所有库文件版本号统一为 1.23.2
✅ 所有依赖路径指向 libonnxruntime.1.23.2.dylib
✅ 应用正常启动运行
✅ Paraformer/Zipformer/Conformer 模型不再崩溃
```

## 相关文件

- **已修改**:
  - `scripts/copy-dylibs.sh` - 构建时复制和修复动态库
  - `/Applications/KeVoiceInput.app` - 已安装的应用

- **相关文档**:
  - `MODEL_DELETE_FIX.md` - 模型删除功能修复
  - `LOCAL_MODELS.md` - 本地模型列表
  - `upgrade-onnxruntime-fast.sh` - onnxruntime 升级脚本

## 技术原理

### 为什么之前文件名是 1.17.1 但内容是 1.23.2？

1. **符号链接策略**:
   - 在 `src-tauri/target/release/` 创建了符号链接：
     ```bash
     libonnxruntime.1.17.1.dylib -> libonnxruntime.1.23.2.dylib
     ```
   - 目的：让依赖 1.17.1 的 sherpa-onnx 能找到文件

2. **复制时解析符号链接**:
   - `copy-dylibs.sh` 使用 `cp` 命令复制时
   - `cp` 默认会**跟随符号链接**，复制真实文件内容
   - 结果：复制了 1.23.2 的内容，但文件名保持为 1.17.1

3. **版本元数据**:
   - 使用 `otool -L` 查看时，显示 `current version 1.23.2`
   - 说明文件内容确实是 1.23.2 版本
   - 但路径名（install name）还是 1.17.1

### install_name_tool 的作用

macOS 动态库使用以下路径前缀：

- `@executable_path`: 可执行文件所在目录
- `@loader_path`: 加载库的文件所在目录
- `@rpath`: 运行时搜索路径

`install_name_tool` 可以修改：

1. **库的 ID** (`-id`): 库自身的标识路径
2. **依赖路径** (`-change`): 库依赖其他库的路径
3. **rpath** (`-add_rpath`, `-delete_rpath`): 运行时搜索路径

## 未来注意事项

1. **升级 onnxruntime 时**:
   - 更新 `scripts/copy-dylibs.sh` 中的版本号
   - 更新 `src-tauri/target/release/` 中的库文件
   - 不再需要创建 1.17.1 -> 1.23.2 的符号链接

2. **构建新版本时**:
   - 脚本会自动使用正确的版本号
   - 依赖路径会正确设置为 1.23.2

3. **验证版本**:
   - 始终检查 `otool -L` 输出
   - 确认文件名和 current version 一致

## 版本信息

- **修复版本**: 0.0.1
- **修复日期**: 2026-02-27
- **onnxruntime 版本**: 1.23.2
- **测试状态**: ✅ 已测试并验证

---

**修复确认** ✅

版本号已统一为 1.23.2：
- ✅ 文件名: `libonnxruntime.1.23.2.dylib`
- ✅ 依赖路径: `@loader_path/libonnxruntime.1.23.2.dylib`
- ✅ 库版本: `current version 1.23.2`
- ✅ 应用正常运行

测试通过！🎉
