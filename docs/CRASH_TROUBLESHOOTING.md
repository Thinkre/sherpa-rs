# 应用崩溃 / 无法打开 问题排查

## 症状

- **应用无法打开**：双击没反应，或被系统阻止（「无法验证开发者」等）
- **意外退出**：安装后双击显示「kevoiceinput 意外退出」或闪退无窗口

---

## 无法打开时先试这两步

1. **首次打开**：在 Finder 里 **右键点击** KeVoiceInput.app → 选择「打开」→ 在弹窗中再点「打开」。
2. **若仍打不开**：打开终端，执行后再次右键 → 打开：
   ```bash
   xattr -cr /Applications/KeVoiceInput.app
   ```
若打开后立刻闪退，属于下面的「意外退出」，按下一节处理。

### 仍然打不开：从终端看报错（必做）

在终端里直接运行可执行文件，会看到具体错误信息（如缺库、签名等）：

```bash
/Applications/KeVoiceInput.app/Contents/MacOS/kevoiceinput
```

常见输出含义：
- **`dyld: Library not loaded: ... libonnxruntime.1.17.1.dylib`** → 需用**最新构建的 DMG** 重装（见下方「意外退出」）
- **`Reason: image not found`** → 同上，动态库缺失或路径错误
- **`Killed: 9`** 或 无输出即退出 → 可能是签名/公证问题，或同上的库问题
- 若有其他英文报错，把**整段终端输出**复制下来便于排查

---

## 意外退出（闪退）症状

安装后双击应用显示：
> "kevoiceinput" 意外退出

或应用闪退，没有任何窗口显示。

## 原因

这是由于动态库加载失败导致的。主要原因包括：

1. **动态库未打包**：Frameworks 目录缺失或不完整
2. **库依赖路径错误**：动态库之间的依赖指向错误的版本
3. **权限问题**：应用或库文件被隔离

## 快速诊断

使用诊断脚本检查问题：

```bash
# 如果应用在 /Applications
./scripts/diagnose-crash.sh

# 或指定路径
./scripts/diagnose-crash.sh /path/to/KeVoiceInput.app
```

脚本会检查：
- 应用结构是否完整
- 必需的动态库是否存在
- 库依赖是否正确配置
- 尝试启动应用

## 解决方案

### 方案 1：使用最新 DMG（推荐）✅

**2026-02-22 之后构建的 DMG** 已修复动态库依赖问题。

检查你的 DMG 版本：
```bash
# 构建日期应该是 2026-02-22 或之后
ls -l KeVoiceInput_0.0.1_aarch64.dmg
```

如果使用的是旧版 DMG：
1. 下载最新版本
2. 重新安装

### 方案 2：重新安装

如果已安装但崩溃，完全重新安装：

```bash
# 1. 删除旧版本
rm -rf /Applications/KeVoiceInput.app

# 2. 挂载 DMG
hdiutil attach KeVoiceInput_0.0.1_aarch64.dmg

# 3. 使用安装脚本
/Volumes/KeVoiceInput/Install.command

# 或手动复制
cp -R /Volumes/KeVoiceInput/KeVoiceInput.app /Applications/
xattr -cr /Applications/KeVoiceInput.app
```

### 方案 3：验证库文件

检查应用的 Frameworks 目录：

```bash
ls -la /Applications/KeVoiceInput.app/Contents/Frameworks/

# 应该看到 4 个 .dylib 文件：
# - libcargs.dylib
# - libonnxruntime.1.23.2.dylib
# - libsherpa-onnx-c-api.dylib
# - libsherpa-onnx-cxx-api.dylib
```

如果缺少文件或目录不存在，说明安装不完整。

### 方案 4：检查库依赖

验证动态库依赖是否正确：

```bash
# 检查主二进制
otool -L /Applications/KeVoiceInput.app/Contents/MacOS/kevoiceinput | grep Frameworks

# 应该看到类似：
# @executable_path/../Frameworks/libonnxruntime.1.23.2.dylib
# @executable_path/../Frameworks/libsherpa-onnx-c-api.dylib

# 检查 sherpa 对 onnxruntime 的引用（必须是 1.23.2，不能是 1.17.1）
otool -L /Applications/KeVoiceInput.app/Contents/Frameworks/libsherpa-onnx-c-api.dylib | grep onnxruntime

# 应显示：@loader_path/libonnxruntime.1.23.2.dylib
# 若显示 libonnxruntime.1.17.1.dylib 会导致「意外退出」，需用最新 DMG 重装。
```

若 sherpa 仍引用 `libonnxruntime.1.17.1.dylib` 而 Frameworks 里只有 `1.23.2.dylib`，dyld 找不到库会直接崩溃。请用**最新一次构建**的 DMG（包含 copy-dylibs 对 1.17.1→1.23.2 的修正）重新安装。

## 查看崩溃日志

如果问题仍未解决，查看系统崩溃日志：

### 方法 1：使用 Console.app

1. 打开 Console.app（应用程序 → 实用工具 → 控制台）
2. 在左侧选择"崩溃报告"或"Crash Reports"
3. 查找 `kevoiceinput` 相关的崩溃日志
4. 双击查看详细信息

### 方法 2：终端查看

```bash
# 查看最近的崩溃日志
ls -lt ~/Library/Logs/DiagnosticReports/kevoiceinput* | head -1

# 查看内容
cat $(ls -t ~/Library/Logs/DiagnosticReports/kevoiceinput* | head -1)
```

### 关键信息

在崩溃日志中查找：

1. **Exception Type**：
   ```
   Exception Type:  EXC_CRASH (SIGABRT)
   Exception Codes: dyld_fatal_error
   ```
   表示动态库加载失败

2. **Dyld Error Message**：
   ```
   Dyld Error Message:
     Library not loaded: @rpath/libonnxruntime.1.23.2.dylib
     Reason: image not found
   ```
   表示找不到指定的动态库

3. **Thread 0 Crashed**：
   ```
   0   dyld   0x... dyld4::prepare
   ```
   表示在启动加载器阶段崩溃

## 常见错误和解决方法

### 错误 1：Library not loaded: @rpath/...

**症状**：崩溃日志显示无法加载 `@rpath/` 路径的库

**原因**：动态库路径配置错误

**解决**：使用 2026-02-22 或之后的 DMG

---

### 错误 2：Frameworks directory not found

**症状**：诊断脚本显示 Frameworks 目录不存在

**原因**：安装时未正确复制动态库

**解决**：
1. 删除应用
2. 使用 DMG 中的 `Install.command` 重新安装
3. 或手动复制后运行诊断脚本

---

### 错误 3：Reason: image not found

**症状**：崩溃日志显示 "Reason: image not found"

**原因**：动态库文件缺失或版本不匹配

**解决**：
```bash
# 检查库文件是否存在
ls /Applications/KeVoiceInput.app/Contents/Frameworks/

# 如果缺失，重新安装
```

---

### 错误 4：Code signature invalid

**症状**：崩溃日志显示代码签名无效

**原因**：手动修改了应用文件

**解决**：
```bash
# 重新签名（需要 Xcode Command Line Tools）
codesign --force --deep --sign - /Applications/KeVoiceInput.app

# 或重新安装
```

---

### 错误 5：Permission denied

**症状**：无法启动，提示权限错误

**原因**：文件权限不正确

**解决**：
```bash
# 修复权限
chmod -R 755 /Applications/KeVoiceInput.app
xattr -cr /Applications/KeVoiceInput.app
```

## 技术细节

### 动态库依赖链

KeVoiceInput 的动态库依赖关系：

```
kevoiceinput (主二进制)
├── libcargs.dylib
├── libonnxruntime.1.17.1.dylib
└── libsherpa-onnx-c-api.dylib
    ├── libonnxruntime.1.17.1.dylib
    └── libsherpa-onnx-cxx-api.dylib
        ├── libonnxruntime.1.17.1.dylib
        └── libsherpa-onnx-c-api.dylib
```

### 路径类型说明

- **@executable_path**：相对于主二进制文件
- **@loader_path**：相对于当前库文件
- **@rpath**：需要额外配置，容易出错（已弃用）

### 修复历史

**2026-02-22**：修复动态库依赖路径问题
- 将 `@rpath/libonnxruntime.1.23.2.dylib` 改为 `@loader_path/libonnxruntime.1.17.1.dylib`
- 所有库之间的依赖使用 `@loader_path`
- 确保打包的 onnxruntime 版本与依赖匹配

## 预防措施

### 开发者

1. **每次构建后验证**：
   ```bash
   ./scripts/diagnose-crash.sh src-tauri/target/release/bundle/macos/KeVoiceInput.app
   ```

2. **测试 DMG 安装**：
   ```bash
   # 在干净环境测试
   rm -rf /Applications/KeVoiceInput.app
   hdiutil attach *.dmg
   /Volumes/KeVoiceInput/Install.command
   ```

3. **检查库依赖**：
   ```bash
   # 自动检查脚本
   ./scripts/copy-dylibs.sh path/to/app
   ```

### 用户

1. **使用最新 DMG**：确保使用 2026-02-22 或之后构建的版本
2. **按照安装指南**：参考 DMG 中的 README.txt
3. **报告问题**：附带崩溃日志和诊断脚本输出

## 报告问题

如果问题仍未解决，请在 GitHub 提交 Issue，附带：

1. **系统信息**：
   ```bash
   sw_vers
   uname -m
   ```

2. **诊断输出**：
   ```bash
   ./scripts/diagnose-crash.sh > diagnostic.txt 2>&1
   ```

3. **崩溃日志**：从 Console.app 导出

4. **安装方法**：使用的哪种安装方式

## 参考资料

- [动态库打包脚本](../scripts/copy-dylibs.sh)
- [崩溃诊断脚本](../scripts/diagnose-crash.sh)
- [DMG 创建脚本](../scripts/create-dmg.sh)
- [Apple 动态库编程指南](https://developer.apple.com/library/archive/documentation/DeveloperTools/Conceptual/DynamicLibraries/)
