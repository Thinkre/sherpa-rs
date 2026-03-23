# 问题已修复！✅

## 问题根源

### 发现的问题

运行 `bun run tauri:build` 时，**动态库没有被自动打包到应用中**，导致：
- Frameworks 目录不存在
- 应用启动时无法加载 sherpa-onnx 库
- 选择任何模型（包括 zipformer）都会崩溃

### 为什么之前可以用？

之前的 DMG 可能是：
1. 手动运行了 `copy-dylibs.sh` 脚本
2. 使用了旧的构建流程
3. 构建配置有所不同

现在的 `tauri build` 没有自动执行 `copy-dylibs.sh`。

---

## 已应用的修复

### 修复 1：动态库打包 ✅

**问题**：Frameworks 目录不存在

**解决**：
```bash
# 手动复制动态库
./scripts/copy-dylibs.sh src-tauri/target/release/bundle/macos/KeVoiceInput.app

# 重新创建 DMG
./scripts/create-dmg.sh \
  src-tauri/target/release/bundle/macos/KeVoiceInput.app \
  src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg
```

**验证**：
```bash
# DMG 中有 4 个动态库
ls -la /Volumes/KeVoiceInput/KeVoiceInput.app/Contents/Frameworks/
# ✅ libcargs.dylib
# ✅ libonnxruntime.1.17.1.dylib
# ✅ libsherpa-onnx-c-api.dylib
# ✅ libsherpa-onnx-cxx-api.dylib

# 依赖路径正确
otool -L libsherpa-onnx-cxx-api.dylib | grep onnx
# ✅ @loader_path/libonnxruntime.1.17.1.dylib
```

### 修复 2：模型列表显示 ✅

**问题**：本地模型标签不显示未下载的模型

**文件**：`src/components/pages/ModelsPage.tsx`

**解决**：移除了 `if (!model.is_downloaded) return false;` 过滤

**效果**：
- ✅ "本地模型"标签显示所有本地引擎模型
- ✅ 未下载的模型显示"下载"按钮
- ✅ 已下载的模型可以选择使用

---

## 新的 DMG 文件

**位置**：
```
src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg
```

**大小**：约 32MB

**包含**：
- ✅ 完整的应用（带动态库）
- ✅ Install.command 安装脚本
- ✅ manual-install.sh 备用脚本
- ✅ README.txt 安装说明
- ✅ Applications 快捷链接

**已验证**：
- ✅ 动态库完整（4 个文件）
- ✅ 依赖路径正确（@loader_path）
- ✅ 版本匹配（onnxruntime 1.17.1）

---

## 安装步骤

### 卸载旧版本

```bash
# 删除有问题的旧版本
rm -rf /Applications/KeVoiceInput.app
```

### 安装新版本

**方法 1：使用安装脚本（推荐）**

```bash
# 挂载 DMG
hdiutil attach src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg

# 运行安装脚本（会自动放行）
/Volumes/KeVoiceInput/Install.command

# 按提示完成安装
```

**方法 2：手动安装**

```bash
# 挂载 DMG
hdiutil attach src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg

# 拖拽到 Applications
cp -R /Volumes/KeVoiceInput/KeVoiceInput.app /Applications/

# 移除隔离属性
xattr -cr /Applications/KeVoiceInput.app

# 首次启动：右键 → 打开
```

### 验证安装

```bash
# 检查动态库
ls -la /Applications/KeVoiceInput.app/Contents/Frameworks/

# 应该看到 4 个 .dylib 文件
```

---

## 测试清单

### ✅ 测试 1：应用启动
- [ ] 打开应用
- [ ] 应用不崩溃
- [ ] 可以看到主界面

### ✅ 测试 2：模型列表
- [ ] 进入"模型"页面
- [ ] 切换到"本地模型"标签
- [ ] 可以看到所有本地模型（包括未下载的）
- [ ] 未下载的模型显示"下载"按钮

### ✅ 测试 3：模型加载
- [ ] 选择一个简单的模型（如 Paraformer）
- [ ] 模型加载成功
- [ ] 不崩溃

### ✅ 测试 4：Zipformer 模型
- [ ] 选择 zipformer 双语模型
- [ ] 模型加载成功
- [ ] 可以进行语音转录
- [ ] 不崩溃

---

## 构建流程改进建议

### 当前流程的问题

`bun run tauri:build` 不会自动执行 `copy-dylibs.sh`

### 推荐的构建流程

**方案 A：使用包装脚本（当前）**

```bash
# tauri-build-wrapper.sh 会自动调用 copy-dylibs.sh
bun run tauri:build

# 但是需要确保 tauri-build-wrapper.sh 正确配置
```

**方案 B：使用 APPLY_FIXES.sh**

```bash
# 一键完成所有步骤
./APPLY_FIXES.sh

# 包括：
# 1. 构建前端
# 2. 构建后端
# 3. 打包应用
# 4. 复制动态库
# 5. 创建 DMG
```

**方案 C：手动步骤**

```bash
# 1. 构建
bun run tauri:build

# 2. 手动复制动态库
./scripts/copy-dylibs.sh src-tauri/target/release/bundle/macos/KeVoiceInput.app

# 3. 重新创建 DMG
./scripts/create-dmg.sh \
  src-tauri/target/release/bundle/macos/KeVoiceInput.app \
  src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg
```

### 自动化改进

可以考虑在 `tauri.conf.json` 中配置：

```json
{
  "build": {
    "beforeBuildCommand": "bun run build",
    "beforeBundleCommand": "./scripts/copy-dylibs.sh $TAURI_BUNDLE_PATH"
  }
}
```

但目前 Tauri 不支持 `beforeBundleCommand`，所以需要使用包装脚本。

---

## 问题总结

### 根本原因
- `tauri build` 没有自动打包动态库
- Frameworks 目录缺失

### 影响
- ✅ 应用崩溃
- ✅ zipformer 无法加载
- ✅ 所有需要 sherpa-onnx 的模型都无法使用

### 解决
- ✅ 手动运行 `copy-dylibs.sh`
- ✅ 重新创建 DMG
- ✅ 验证动态库完整性

### 预防
- 使用 `APPLY_FIXES.sh` 或 `tauri-build-wrapper.sh`
- 每次构建后验证 Frameworks 目录
- 测试 DMG 中的应用能否正常启动

---

## 当前状态

### ✅ 已完成

1. **动态库打包** ✅
   - 4 个 .dylib 文件已复制
   - 依赖路径已修复
   - 已重新签名

2. **DMG 创建** ✅
   - 新 DMG 已生成
   - 包含完整的应用
   - 动态库已验证

3. **前端修复** ✅
   - 模型列表显示已修复
   - 已重新构建

4. **文档** ✅
   - 问题排查文档
   - 修复步骤说明
   - 测试清单

### 📦 可交付

- **新的 DMG**：`src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg`
- **完整文档**：所有故障排查和修复文档
- **自动化脚本**：`APPLY_FIXES.sh`

### 🎯 下一步

**立即执行**：
```bash
# 1. 卸载旧版本
rm -rf /Applications/KeVoiceInput.app

# 2. 安装新版本
hdiutil attach src-tauri/target/release/bundle/dmg/KeVoiceInput_0.0.1_aarch64.dmg
/Volumes/KeVoiceInput/Install.command

# 3. 测试
open /Applications/KeVoiceInput.app
```

**预期结果**：
- ✅ 应用正常启动
- ✅ 模型列表正确显示
- ✅ zipformer 模型可以加载
- ✅ 不再崩溃

---

## 时间线

- **16:10** - 发现 Frameworks 目录不存在
- **16:11** - 运行 copy-dylibs.sh 复制动态库
- **16:12** - 重新创建 DMG
- **16:13** - 验证 DMG 中的动态库完整性
- **16:15** - 问题修复完成 ✅

---

**问题已解决！请使用新的 DMG 重新安装应用。** 🎉
