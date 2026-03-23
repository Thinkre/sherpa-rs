# 模型删除功能修复

## 修复日期
2026-02-26

## 问题描述

**之前的行为**：
- 用户点击"删除"按钮后
- ✅ 本地模型文件被删除
- ❌ 模型卡片也从界面上消失
- ❌ 用户无法再次下载该模型（因为模型信息被移除）

**期望的行为**：
- 用户点击"删除"按钮后
- ✅ 本地模型文件被删除
- ✅ 模型卡片保留在界面上
- ✅ 显示"下载"按钮（因为模型已不在本地）

## 根本原因

在 `src-tauri/src/managers/model.rs` 的 `delete_model` 方法中：

```rust
// 问题代码 (第 1249-1254 行)
{
    let mut models = self.available_models.lock().unwrap();
    models.remove(model_id);  // ❌ 错误：从 HashMap 中移除了模型
    info!("Removed model {} from available models", model_id);
}
```

这导致：
1. 模型信息从 `available_models` HashMap 中被移除
2. `get_available_models()` 返回的列表中不再包含该模型
3. 前端收到的模型列表中缺少该模型
4. 界面上的模型卡片消失

## 修复方案

**修改**：`src-tauri/src/managers/model.rs` (第 1245-1257 行)

```rust
// 修复后的代码
if !deleted_something {
    return Err(anyhow::anyhow!("No model files found to delete"));
}

// Don't remove model from available_models - just update its download status
// This allows the model card to remain visible in the UI with a "Download" button
info!("Model {} files deleted, updating download status", model_id);

// Update download status - this will mark the model as not downloaded
self.update_download_status()?;
debug!("ModelManager: download status updated");

Ok(())
```

**关键变化**：
1. ❌ 删除了 `models.remove(model_id)` 这一行
2. ✅ 保留模型在 `available_models` 中
3. ✅ 调用 `update_download_status()` 更新下载状态
4. ✅ 模型的 `is_downloaded` 字段会自动变为 `false`

## 工作原理

1. **删除文件**：
   - 删除模型文件/目录：`fs::remove_dir_all(&model_path)`
   - 删除部分下载文件：`fs::remove_file(&partial_path)`

2. **更新状态**：
   - 调用 `update_download_status()`
   - 该方法会扫描 models 目录
   - 自动更新每个模型的 `is_downloaded` 状态
   - 已删除的模型会被标记为 `is_downloaded: false`

3. **前端显示**：
   - `ModelCard` 组件根据 `is_downloaded` 显示按钮
   - `is_downloaded: true` → 显示"删除"按钮
   - `is_downloaded: false` → 显示"下载"按钮

## 测试步骤

### 1. 下载一个模型
```
1. 打开应用
2. 进入"模型"页面
3. 切换到"本地模型"标签
4. 选择任意模型（如 Paraformer）
5. 点击"下载"按钮
6. 等待下载完成
```

**预期结果**：
- ✅ 模型卡片显示"使用"和"删除"按钮
- ✅ `is_downloaded: true`

### 2. 删除模型
```
1. 点击"删除"按钮
2. 确认删除
```

**预期结果**：
- ✅ 模型文件被删除（可以检查 `~/Library/Application Support/com.kevoiceinput.app/models/`）
- ✅ 模型卡片仍然显示在界面上
- ✅ 显示"下载"按钮（不再显示"使用"和"删除"）
- ✅ `is_downloaded: false`

### 3. 重新下载
```
1. 点击"下载"按钮
2. 等待下载完成
```

**预期结果**：
- ✅ 模型重新下载
- ✅ 卡片恢复显示"使用"和"删除"按钮
- ✅ `is_downloaded: true`

## 验证命令

### 查看模型文件
```bash
ls -la ~/Library/Application\ Support/com.kevoiceinput.app/models/
```

### 检查应用日志
```bash
tail -f ~/Library/Logs/com.kevoiceinput.app/main.log | grep -E "delete|download"
```

## 相关文件

- `src-tauri/src/managers/model.rs` - 模型管理器（已修改）
- `src/components/pages/ModelsPage.tsx` - 模型页面组件
- `src/components/models/ModelCard.tsx` - 模型卡片组件

## 版本信息

- **修复版本**: 0.0.1
- **修复日期**: 2026-02-26
- **测试状态**: ✅ 已测试并验证

## 注意事项

1. **导入的本地模型**：
   - 通过"导入本地模型"功能添加的模型
   - 这些模型通常标记为 `custom-xxx`
   - 删除时也会保留卡片，可以重新导入

2. **正在下载的模型**：
   - 如果模型正在下载中
   - 删除会同时删除部分文件 (`.partial`)
   - 卡片保留，可以重新开始下载

3. **API 模型**：
   - API 类型的模型（如 OpenAI、DeepSeek）
   - 不受此修复影响
   - 它们没有本地文件，删除是从配置中移除

## 未来改进建议

1. **确认对话框**：
   - 添加删除确认对话框
   - 显示模型大小和名称
   - "确定要删除 [模型名] (XXX MB) 吗？"

2. **批量操作**：
   - 允许批量删除多个模型
   - "清理所有未使用的模型"选项

3. **回收站功能**：
   - 删除的模型不立即清除
   - 放入"最近删除"区域
   - 可以快速恢复

4. **存储空间显示**：
   - 显示每个模型占用的空间
   - 显示总存储空间使用情况
   - "释放空间"建议

---

## 修复确认

✅ **模型删除功能已修复**

删除模型后：
- ✅ 本地文件被清除
- ✅ 卡片保留在界面
- ✅ 显示下载按钮
- ✅ 可以重新下载

测试通过！🎉
