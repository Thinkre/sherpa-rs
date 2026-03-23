# KeVoiceInput API 文档

本文档详细说明了 KeVoiceInput 应用的所有 Tauri 命令接口。

## 目录

- [API 概述](#api-概述)
- [模型管理 API](#模型管理-api)
- [历史记录 API](#历史记录-api)
- [音频设备 API](#音频设备-api)
- [转录 API](#转录-api)
- [设置 API](#设置-api)
- [快捷键 API](#快捷键-api)
- [LLM 后处理 API](#llm-后处理-api)
- [转录 API 配置](#转录-api-配置)
- [工具 API](#工具-api)
- [事件系统](#事件系统)
- [类型定义](#类型定义)

## API 概述

KeVoiceInput 使用 Tauri IPC 进行前后端通信。所有 API 调用都通过 `commands` 对象进行，返回 `Result<T, string>` 类型。

### 基本用法

```typescript
import { commands } from '@/bindings';

// 调用 API
const result = await commands.getAvailableModels();
if (result.status === 'ok') {
  console.log(result.data);
} else {
  console.error(result.error);
}
```

### 错误处理

所有 API 调用都可能返回错误。建议始终检查返回状态：

```typescript
const result = await commands.someCommand();
if (result.status === 'error') {
  // 处理错误
  console.error(result.error);
  return;
}
// 使用成功的数据
const data = result.data;
```

## 模型管理 API

### getAvailableModels

获取所有可用模型列表。

**签名**:
```typescript
getAvailableModels(): Promise<Result<ModelInfo[], string>>
```

**返回**:
- `ModelInfo[]`: 模型信息数组

**示例**:
```typescript
const result = await commands.getAvailableModels();
if (result.status === 'ok') {
  result.data.forEach(model => {
    console.log(`${model.name}: ${model.size_mb}MB`);
  });
}
```

### getModelInfo

获取指定模型的详细信息。

**签名**:
```typescript
getModelInfo(modelId: string): Promise<Result<ModelInfo | null, string>>
```

**参数**:
- `modelId`: 模型 ID

**返回**:
- `ModelInfo | null`: 模型信息，如果不存在则返回 null

**示例**:
```typescript
const result = await commands.getModelInfo('small');
if (result.status === 'ok' && result.data) {
  console.log(`Model: ${result.data.name}`);
  console.log(`Size: ${result.data.size_mb}MB`);
  console.log(`Downloaded: ${result.data.is_downloaded}`);
}
```

### downloadModel

下载指定模型。

**签名**:
```typescript
downloadModel(modelId: string): Promise<Result<null, string>>
```

**参数**:
- `modelId`: 要下载的模型 ID

**事件**:
- `model-download-progress`: 下载进度事件
- `model-state-changed`: 模型状态变化事件

**示例**:
```typescript
// 监听下载进度
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen('model-download-progress', (event) => {
  console.log(`Progress: ${event.payload.progress}%`);
});

await commands.downloadModel('small');
```

### deleteModel

删除指定模型。

**签名**:
```typescript
deleteModel(modelId: string): Promise<Result<null, string>>
```

**参数**:
- `modelId`: 要删除的模型 ID

**示例**:
```typescript
const result = await commands.deleteModel('small');
if (result.status === 'ok') {
  console.log('Model deleted successfully');
}
```

### setActiveModel

设置当前激活的模型。

**签名**:
```typescript
setActiveModel(modelId: string): Promise<Result<null, string>>
```

**参数**:
- `modelId`: 要激活的模型 ID

**说明**:
- 模型必须已下载
- 会自动加载模型到内存

**示例**:
```typescript
const result = await commands.setActiveModel('small');
if (result.status === 'ok') {
  console.log('Model activated');
}
```

### getCurrentModel

获取当前激活的模型 ID。

**签名**:
```typescript
getCurrentModel(): Promise<Result<string, string>>
```

**返回**:
- `string`: 当前模型 ID

**示例**:
```typescript
const result = await commands.getCurrentModel();
if (result.status === 'ok') {
  console.log(`Current model: ${result.data}`);
}
```

### cancelDownload

取消正在进行的模型下载。

**签名**:
```typescript
cancelDownload(modelId: string): Promise<Result<null, string>>
```

**参数**:
- `modelId`: 要取消下载的模型 ID

**示例**:
```typescript
await commands.cancelDownload('small');
```

### importLocalModelFolder

导入本地模型文件夹。

**签名**:
```typescript
importLocalModelFolder(sourcePath: string): Promise<Result<string, string>>
```

**参数**:
- `sourcePath`: 模型文件夹路径

**返回**:
- `string`: 导入后的模型 ID

**示例**:
```typescript
const result = await commands.importLocalModelFolder('/path/to/model');
if (result.status === 'ok') {
  console.log(`Imported model ID: ${result.data}`);
}
```

### hasAnyModelsAvailable

检查是否有任何已下载的模型。

**签名**:
```typescript
hasAnyModelsAvailable(): Promise<Result<boolean, string>>
```

**返回**:
- `boolean`: 是否有可用模型

**示例**:
```typescript
const result = await commands.hasAnyModelsAvailable();
if (result.status === 'ok' && result.data) {
  console.log('Models available');
}
```

## 历史记录 API

### getHistoryEntries

获取所有历史记录条目。

**签名**:
```typescript
getHistoryEntries(): Promise<Result<HistoryEntry[], string>>
```

**返回**:
- `HistoryEntry[]`: 历史记录数组

**示例**:
```typescript
const result = await commands.getHistoryEntries();
if (result.status === 'ok') {
  result.data.forEach(entry => {
    console.log(`${entry.text} - ${entry.timestamp}`);
  });
}
```

### toggleHistoryEntrySaved

切换历史记录条目的保存状态。

**签名**:
```typescript
toggleHistoryEntrySaved(id: number): Promise<Result<null, string>>
```

**参数**:
- `id`: 历史记录条目 ID

**示例**:
```typescript
await commands.toggleHistoryEntrySaved(123);
```

### deleteHistoryEntry

删除指定的历史记录条目。

**签名**:
```typescript
deleteHistoryEntry(id: number): Promise<Result<null, string>>
```

**参数**:
- `id`: 要删除的历史记录条目 ID

**示例**:
```typescript
await commands.deleteHistoryEntry(123);
```

### getAudioFilePath

获取历史记录条目的音频文件路径。

**签名**:
```typescript
getAudioFilePath(fileName: string): Promise<Result<string, string>>
```

**参数**:
- `fileName`: 音频文件名

**返回**:
- `string`: 音频文件的完整路径

**示例**:
```typescript
const result = await commands.getAudioFilePath('recording_123.wav');
if (result.status === 'ok') {
  console.log(`Audio file: ${result.data}`);
}
```

### updateHistoryLimit

更新历史记录数量限制。

**签名**:
```typescript
updateHistoryLimit(limit: number): Promise<Result<null, string>>
```

**参数**:
- `limit`: 新的历史记录数量限制

**示例**:
```typescript
await commands.updateHistoryLimit(100);
```

### updateRecordingRetentionPeriod

更新录音文件保留期限。

**签名**:
```typescript
updateRecordingRetentionPeriod(period: string): Promise<Result<null, string>>
```

**参数**:
- `period`: 保留期限，可选值：
  - `"never"`: 不保留
  - `"preserve_limit"`: 保留到限制数量
  - `"days3"`: 保留 3 天
  - `"weeks2"`: 保留 2 周
  - `"months3"`: 保留 3 个月

**示例**:
```typescript
await commands.updateRecordingRetentionPeriod('days3');
```

## 音频设备 API

### getAvailableMicrophones

获取所有可用的麦克风设备。

**签名**:
```typescript
getAvailableMicrophones(): Promise<Result<AudioDevice[], string>>
```

**返回**:
- `AudioDevice[]`: 音频设备数组

**示例**:
```typescript
const result = await commands.getAvailableMicrophones();
if (result.status === 'ok') {
  result.data.forEach(device => {
    console.log(`${device.name} (${device.index})`);
  });
}
```

### setSelectedMicrophone

设置选中的麦克风设备。

**签名**:
```typescript
setSelectedMicrophone(deviceName: string): Promise<Result<null, string>>
```

**参数**:
- `deviceName`: 设备名称，使用 `"default"` 表示默认设备

**示例**:
```typescript
await commands.setSelectedMicrophone('Built-in Microphone');
```

### getSelectedMicrophone

获取当前选中的麦克风设备。

**签名**:
```typescript
getSelectedMicrophone(): Promise<Result<string, string>>
```

**返回**:
- `string`: 设备名称，`"default"` 表示默认设备

**示例**:
```typescript
const result = await commands.getSelectedMicrophone();
if (result.status === 'ok') {
  console.log(`Selected microphone: ${result.data}`);
}
```

### getAvailableOutputDevices

获取所有可用的输出设备。

**签名**:
```typescript
getAvailableOutputDevices(): Promise<Result<AudioDevice[], string>>
```

**返回**:
- `AudioDevice[]`: 音频设备数组

**示例**:
```typescript
const result = await commands.getAvailableOutputDevices();
if (result.status === 'ok') {
  result.data.forEach(device => {
    console.log(`${device.name}`);
  });
}
```

### setSelectedOutputDevice

设置选中的输出设备。

**签名**:
```typescript
setSelectedOutputDevice(deviceName: string): Promise<Result<null, string>>
```

**参数**:
- `deviceName`: 设备名称，使用 `"default"` 表示默认设备

**示例**:
```typescript
await commands.setSelectedOutputDevice('Built-in Speakers');
```

### getSelectedOutputDevice

获取当前选中的输出设备。

**签名**:
```typescript
getSelectedOutputDevice(): Promise<Result<string, string>>
```

**返回**:
- `string`: 设备名称

**示例**:
```typescript
const result = await commands.getSelectedOutputDevice();
if (result.status === 'ok') {
  console.log(`Selected output: ${result.data}`);
}
```

### updateMicrophoneMode

更新麦克风模式。

**签名**:
```typescript
updateMicrophoneMode(alwaysOn: boolean): Promise<Result<null, string>>
```

**参数**:
- `alwaysOn`: 是否始终开启麦克风

**说明**:
- `true`: 始终开启模式，麦克风持续监听
- `false`: 按需开启模式，仅在录音时开启

**示例**:
```typescript
await commands.updateMicrophoneMode(true);
```

### getMicrophoneMode

获取当前麦克风模式。

**签名**:
```typescript
getMicrophoneMode(): Promise<Result<boolean, string>>
```

**返回**:
- `boolean`: `true` 表示始终开启，`false` 表示按需开启

**示例**:
```typescript
const result = await commands.getMicrophoneMode();
if (result.status === 'ok') {
  console.log(`Mode: ${result.data ? 'Always On' : 'On Demand'}`);
}
```

### isRecording

检查是否正在录音。

**签名**:
```typescript
isRecording(): Promise<boolean>
```

**返回**:
- `boolean`: 是否正在录音

**示例**:
```typescript
const isRecording = await commands.isRecording();
console.log(`Recording: ${isRecording}`);
```

### playTestSound

播放测试声音。

**签名**:
```typescript
playTestSound(soundType: string): Promise<void>
```

**参数**:
- `soundType`: 声音类型，`"start"` 或 `"stop"`

**示例**:
```typescript
await commands.playTestSound('start');
```

## 标点符号 API

### changePunctuationEnabledSetting

更新标点符号自动添加启用状态。

**签名**:
```typescript
changePunctuationEnabledSetting(enabled: boolean): Promise<Result<null, string>>
```

**参数**:
- `enabled`: 是否启用标点符号自动添加

**说明**:
- 启用后会自动下载和加载标点符号模型
- 模型会在首次使用时自动下载

**示例**:
```typescript
await commands.changePunctuationEnabledSetting(true);
```

### ensurePunctuationModel

确保标点符号模型已下载并加载。

**签名**:
```typescript
ensurePunctuationModel(): Promise<Result<null, string>>
```

**说明**:
- 如果模型未下载，会自动下载
- 如果模型未加载，会自动加载
- 通常在启用标点符号功能时自动调用

**示例**:
```typescript
const result = await commands.ensurePunctuationModel();
if (result.status === 'ok') {
  console.log('Punctuation model ready');
}
```

## 转录 API

### getModelLoadStatus

获取模型加载状态。

**签名**:
```typescript
getModelLoadStatus(): Promise<Result<ModelLoadStatus, string>>
```

**返回**:
- `ModelLoadStatus`: 模型加载状态

**类型定义**:
```typescript
interface ModelLoadStatus {
  is_loaded: boolean;
  current_model: string | null;
}
```

**示例**:
```typescript
const result = await commands.getModelLoadStatus();
if (result.status === 'ok') {
  console.log(`Loaded: ${result.data.is_loaded}`);
  console.log(`Model: ${result.data.current_model}`);
}
```

### unloadModelManually

手动卸载模型。

**签名**:
```typescript
unloadModelManually(): Promise<Result<null, string>>
```

**说明**:
- 立即卸载当前加载的模型
- 释放内存资源

**示例**:
```typescript
await commands.unloadModelManually();
```

### setModelUnloadTimeout

设置模型自动卸载超时时间。

**签名**:
```typescript
setModelUnloadTimeout(timeout: ModelUnloadTimeout): Promise<void>
```

**参数**:
- `timeout`: 超时设置

**类型定义**:
```typescript
type ModelUnloadTimeout = 
  | 'Immediately'      // 立即卸载
  | 'After10Seconds'   // 10 秒后卸载
  | 'After30Seconds'   // 30 秒后卸载
  | 'After1Minute'     // 1 分钟后卸载
  | 'After5Minutes'    // 5 分钟后卸载
  | 'After10Minutes'   // 10 分钟后卸载
  | 'Never';           // 永不卸载
```

**示例**:
```typescript
await commands.setModelUnloadTimeout('After5Minutes');
```

## 设置 API

### getAppSettings

获取应用所有设置。

**签名**:
```typescript
getAppSettings(): Promise<Result<AppSettings, string>>
```

**返回**:
- `AppSettings`: 应用设置对象

**示例**:
```typescript
const result = await commands.getAppSettings();
if (result.status === 'ok') {
  console.log(`Language: ${result.data.selected_language}`);
  console.log(`Model: ${result.data.selected_model}`);
}
```

### getDefaultSettings

获取默认设置。

**签名**:
```typescript
getDefaultSettings(): Promise<Result<AppSettings, string>>
```

**返回**:
- `AppSettings`: 默认设置对象

**示例**:
```typescript
const result = await commands.getDefaultSettings();
if (result.status === 'ok') {
  // 使用默认设置重置配置
}
```

### 设置更新 API

以下 API 用于更新特定设置项：

#### changeSelectedLanguageSetting

更新选中的语言。

```typescript
changeSelectedLanguageSetting(language: string): Promise<Result<null, string>>
```

#### changeItnEnabledSetting

更新 ITN（逆文本规范化）启用状态。

```typescript
changeItnEnabledSetting(enabled: boolean): Promise<Result<null, string>>
```

#### changePunctuationEnabledSetting

更新标点符号自动添加启用状态。

```typescript
changePunctuationEnabledSetting(enabled: boolean): Promise<Result<null, string>>
```

**参数**:
- `enabled`: 是否启用标点符号自动添加

**说明**:
- 启用后会自动下载和加载标点符号模型
- 模型会在首次使用时自动下载

**示例**:
```typescript
await commands.changePunctuationEnabledSetting(true);
```

#### ensurePunctuationModel

确保标点符号模型已下载并加载。

**签名**:
```typescript
ensurePunctuationModel(): Promise<Result<null, string>>
```

**说明**:
- 如果模型未下载，会自动下载
- 如果模型未加载，会自动加载
- 通常在启用标点符号功能时自动调用

**示例**:
```typescript
const result = await commands.ensurePunctuationModel();
if (result.status === 'ok') {
  console.log('Punctuation model ready');
}
```

#### changeTranslateToEnglishSetting

更新翻译到英文设置。

```typescript
changeTranslateToEnglishSetting(enabled: boolean): Promise<Result<null, string>>
```

#### changeAutostartSetting

更新自动启动设置。

```typescript
changeAutostartSetting(enabled: boolean): Promise<Result<null, string>>
```

#### changeStartHiddenSetting

更新启动时隐藏设置。

```typescript
changeStartHiddenSetting(enabled: boolean): Promise<Result<null, string>>
```

#### changeDebugModeSetting

更新调试模式设置。

```typescript
changeDebugModeSetting(enabled: boolean): Promise<Result<null, string>>
```

#### changeAppLanguageSetting

更新应用界面语言。

```typescript
changeAppLanguageSetting(language: string): Promise<Result<null, string>>
```

#### changeUpdateChecksSetting

更新更新检查设置。

```typescript
changeUpdateChecksSetting(enabled: boolean): Promise<Result<null, string>>
```

#### changeMuteWhileRecordingSetting

更新录音时静音设置。

```typescript
changeMuteWhileRecordingSetting(enabled: boolean): Promise<Result<null, string>>
```

#### changeAppendTrailingSpaceSetting

更新追加尾随空格设置。

```typescript
changeAppendTrailingSpaceSetting(enabled: boolean): Promise<Result<null, string>>
```

#### changePttSetting

更新 PTT（Push-to-Talk）设置。

```typescript
changePttSetting(enabled: boolean): Promise<Result<null, string>>
```

#### changeAudioFeedbackSetting

更新音频反馈设置。

```typescript
changeAudioFeedbackSetting(enabled: boolean): Promise<Result<null, string>>
```

#### changeAudioFeedbackVolumeSetting

更新音频反馈音量。

```typescript
changeAudioFeedbackVolumeSetting(volume: number): Promise<Result<null, string>>
```

**参数**:
- `volume`: 音量值（0.0 - 1.0）

#### changeSoundThemeSetting

更新声音主题设置。

```typescript
changeSoundThemeSetting(theme: string): Promise<Result<null, string>>
```

#### changeOverlayPositionSetting

更新覆盖层位置设置。

```typescript
changeOverlayPositionSetting(position: string): Promise<Result<null, string>>
```

#### changeWordCorrectionThresholdSetting

更新词汇纠正阈值。

```typescript
changeWordCorrectionThresholdSetting(threshold: number): Promise<Result<null, string>>
```

#### changePasteMethodSetting

更新粘贴方法设置。

```typescript
changePasteMethodSetting(method: string): Promise<Result<null, string>>
```

#### changeClipboardHandlingSetting

更新剪贴板处理设置。

```typescript
changeClipboardHandlingSetting(handling: string): Promise<Result<null, string>>
```

#### updateCustomWords

更新自定义词汇列表。

```typescript
updateCustomWords(words: string[]): Promise<Result<null, string>>
```

**参数**:
- `words`: 自定义词汇数组

**示例**:
```typescript
await commands.updateCustomWords(['专业术语1', '专业术语2']);
```

## 快捷键 API

### changeBinding

更改快捷键绑定。

**签名**:
```typescript
changeBinding(id: string, binding: string): Promise<Result<BindingResponse, string>>
```

**参数**:
- `id`: 快捷键 ID
- `binding`: 新的快捷键组合（如 `"CommandOrControl+Shift+T"`）

**返回**:
- `BindingResponse`: 绑定结果

**示例**:
```typescript
const result = await commands.changeBinding('transcribe', 'CommandOrControl+Shift+T');
if (result.status === 'ok') {
  console.log('Binding updated');
}
```

### resetBinding

重置快捷键绑定到默认值。

**签名**:
```typescript
resetBinding(id: string): Promise<Result<BindingResponse, string>>
```

**参数**:
- `id`: 快捷键 ID

**示例**:
```typescript
await commands.resetBinding('transcribe');
```

### suspendBinding

临时挂起快捷键绑定（编辑时使用）。

**签名**:
```typescript
suspendBinding(id: string): Promise<Result<null, string>>
```

**说明**:
- 在用户编辑快捷键时临时取消注册，避免触发动作

**示例**:
```typescript
await commands.suspendBinding('transcribe');
// 用户编辑快捷键...
await commands.resumeBinding('transcribe');
```

### resumeBinding

恢复快捷键绑定。

**签名**:
```typescript
resumeBinding(id: string): Promise<Result<null, string>>
```

**示例**:
```typescript
await commands.resumeBinding('transcribe');
```

## LLM 后处理 API

### updateLlmRoles

更新 LLM 角色列表。

**签名**:
```typescript
updateLlmRoles(roles: LLMRole[]): Promise<Result<null, string>>
```

**参数**:
- `roles`: LLM 角色数组

**类型定义**:
```typescript
interface LLMRole {
  id: string;
  name: string;
  prompt: string;
  output_method: 'Direct' | 'Toast';
  enabled: boolean;
}
```

**示例**:
```typescript
await commands.updateLlmRoles([
  {
    id: 'role1',
    name: '代码格式化',
    prompt: '将以下文本格式化为代码',
    output_method: 'Direct',
    enabled: true
  }
]);
```

### updateLlmGlobalConfig

更新全局 LLM 配置。

**签名**:
```typescript
updateLlmGlobalConfig(
  providerId: string | null,
  apiKey: string | null,
  model: string | null
): Promise<Result<null, string>>
```

**参数**:
- `providerId`: 提供商 ID
- `apiKey`: API Key
- `model`: 模型名称

**示例**:
```typescript
await commands.updateLlmGlobalConfig('openai', 'sk-...', 'gpt-4');
```

### updateLlmConfigs

更新 LLM 配置列表。

**签名**:
```typescript
updateLlmConfigs(configs: LLMConfig[]): Promise<Result<null, string>>
```

**参数**:
- `configs`: LLM 配置数组

### updateSelectedLlmConfigId

更新选中的 LLM 配置 ID。

**签名**:
```typescript
updateSelectedLlmConfigId(configId: string | null): Promise<Result<null, string>>
```

### testLlmConfig

测试 LLM 配置。

**签名**:
```typescript
testLlmConfig(configId: string): Promise<Result<string, string>>
```

**返回**:
- `string`: 测试结果消息

**示例**:
```typescript
const result = await commands.testLlmConfig('config1');
if (result.status === 'ok') {
  console.log(result.data);
}
```

### clearLlmMemory

清除 LLM 记忆（rectify 记录）。

**签名**:
```typescript
clearLlmMemory(): Promise<Result<null, string>>
```

**示例**:
```typescript
await commands.clearLlmMemory();
```

## 转录 API 配置

### updateTranscriptionApiConfigs

更新转录 API 配置列表。

**签名**:
```typescript
updateTranscriptionApiConfigs(configs: TranscriptionApiConfig[]): Promise<Result<null, string>>
```

**参数**:
- `configs`: 转录 API 配置数组

### updateSelectedTranscriptionApiConfigId

更新选中的转录 API 配置 ID。

**签名**:
```typescript
updateSelectedTranscriptionApiConfigId(configId: string | null): Promise<Result<null, string>>
```

### updateUseTranscriptionApi

更新是否使用转录 API。

**签名**:
```typescript
updateUseTranscriptionApi(enabled: boolean): Promise<Result<null, string>>
```

**参数**:
- `enabled`: 是否启用转录 API

**说明**:
- `true`: 使用外部 API 进行转录
- `false`: 使用本地模型进行转录

### testTranscriptionApiWithFile

使用本地音频文件测试转录 API 配置。

**签名**:
```typescript
testTranscriptionApiWithFile(
  providerId: string,
  apiKey: string,
  model: string,
  customApiUrl: string | null
): Promise<Result<string, string>>
```

**参数**:
- `providerId`: 提供商 ID
- `apiKey`: API Key
- `model`: 模型名称
- `customApiUrl`: 自定义 API URL（可选）

**返回**:
- `string`: 测试结果文本

**示例**:
```typescript
const result = await commands.testTranscriptionApiWithFile(
  'openai',
  'sk-...',
  'whisper-1',
  null
);
if (result.status === 'ok') {
  console.log(`Transcription: ${result.data}`);
}
```

## 后处理 API

### changePostProcessEnabledSetting

更新后处理启用状态。

**签名**:
```typescript
changePostProcessEnabledSetting(enabled: boolean): Promise<Result<null, string>>
```

### setPostProcessProvider

设置后处理提供商。

**签名**:
```typescript
setPostProcessProvider(providerId: string): Promise<Result<null, string>>
```

### changePostProcessBaseUrlSetting

更新后处理基础 URL。

**签名**:
```typescript
changePostProcessBaseUrlSetting(
  providerId: string,
  baseUrl: string
): Promise<Result<null, string>>
```

### changePostProcessApiKeySetting

更新后处理 API Key。

**签名**:
```typescript
changePostProcessApiKeySetting(
  providerId: string,
  apiKey: string
): Promise<Result<null, string>>
```

### changePostProcessModelSetting

更新后处理模型。

**签名**:
```typescript
changePostProcessModelSetting(
  providerId: string,
  model: string
): Promise<Result<null, string>>
```

### fetchPostProcessModels

获取后处理模型列表。

**签名**:
```typescript
fetchPostProcessModels(providerId: string): Promise<Result<string[], string>>
```

**返回**:
- `string[]`: 模型名称数组

### addPostProcessPrompt

添加后处理提示词。

**签名**:
```typescript
addPostProcessPrompt(name: string, prompt: string): Promise<Result<LLMPrompt, string>>
```

**返回**:
- `LLMPrompt`: 创建的提示词对象

### updatePostProcessPrompt

更新后处理提示词。

**签名**:
```typescript
updatePostProcessPrompt(
  id: string,
  name: string,
  prompt: string
): Promise<Result<null, string>>
```

### deletePostProcessPrompt

删除后处理提示词。

**签名**:
```typescript
deletePostProcessPrompt(id: string): Promise<Result<null, string>>
```

### setPostProcessSelectedPrompt

设置选中的后处理提示词。

**签名**:
```typescript
setPostProcessSelectedPrompt(id: string): Promise<Result<null, string>>
```

## 工具 API

### cancelOperation

取消当前操作。

**签名**:
```typescript
cancelOperation(): Promise<void>
```

**说明**:
- 取消正在进行的操作（如下载、转录等）

**示例**:
```typescript
await commands.cancelOperation();
```

### getAppDirPath

获取应用数据目录路径。

**签名**:
```typescript
getAppDirPath(): Promise<Result<string, string>>
```

**返回**:
- `string`: 应用数据目录路径

**示例**:
```typescript
const result = await commands.getAppDirPath();
if (result.status === 'ok') {
  console.log(`App directory: ${result.data}`);
}
```

### getLogDirPath

获取日志目录路径。

**签名**:
```typescript
getLogDirPath(): Promise<Result<string, string>>
```

**返回**:
- `string`: 日志目录路径

### openRecordingsFolder

打开录音文件夹。

**签名**:
```typescript
openRecordingsFolder(): Promise<Result<null, string>>
```

**说明**:
- 在系统文件管理器中打开录音文件夹

**示例**:
```typescript
await commands.openRecordingsFolder();
```

### openLogDir

打开日志目录。

**签名**:
```typescript
openLogDir(): Promise<Result<null, string>>
```

### openAppDataDir

打开应用数据目录。

**签名**:
```typescript
openAppDataDir(): Promise<Result<null, string>>
```

### setLogLevel

设置日志级别。

**签名**:
```typescript
setLogLevel(level: LogLevel): Promise<Result<null, string>>
```

**参数**:
- `level`: 日志级别

**类型定义**:
```typescript
type LogLevel = 'Trace' | 'Debug' | 'Info' | 'Warn' | 'Error';
```

**示例**:
```typescript
await commands.setLogLevel('Debug');
```

### initializeEnigo

初始化 Enigo（键盘/鼠标模拟）。

**签名**:
```typescript
initializeEnigo(): Promise<Result<null, string>>
```

**说明**:
- macOS 需要辅助功能权限
- 用于键盘输入功能

**示例**:
```typescript
const result = await commands.initializeEnigo();
if (result.status === 'error') {
  console.error('Failed to initialize Enigo:', result.error);
}
```

### checkAppleIntelligenceAvailable

检查 Apple Intelligence 是否可用。

**签名**:
```typescript
checkAppleIntelligenceAvailable(): Promise<boolean>
```

**返回**:
- `boolean`: 是否可用

**说明**:
- 仅在 macOS ARM64 设备上可用

**示例**:
```typescript
const available = await commands.checkAppleIntelligenceAvailable();
if (available) {
  console.log('Apple Intelligence is available');
}
```

### triggerUpdateCheck

触发更新检查。

**签名**:
```typescript
triggerUpdateCheck(): Promise<Result<null, string>>
```

**示例**:
```typescript
await commands.triggerUpdateCheck();
```

### checkCustomSounds

检查自定义声音文件是否存在。

**签名**:
```typescript
checkCustomSounds(): Promise<CustomSounds>
```

**返回**:
- `CustomSounds`: 自定义声音状态

**类型定义**:
```typescript
interface CustomSounds {
  start: boolean;
  stop: boolean;
}
```

## 事件系统

KeVoiceInput 使用 Tauri 事件系统进行前后端通信。

### 监听事件

```typescript
import { listen } from '@tauri-apps/api/event';

// 监听模型下载进度
const unlisten = await listen('model-download-progress', (event) => {
  console.log(`Progress: ${event.payload.progress}%`);
});

// 取消监听
unlisten();
```

### 可用事件

#### model-download-progress

模型下载进度事件。

**负载**:
```typescript
{
  model_id: string;
  progress: number;  // 0-100
}
```

#### model-state-changed

模型状态变化事件。

**负载**:
```typescript
{
  event_type: string;      // 'loading' | 'loaded' | 'loading_failed' | 'unloaded'
  model_id: string | null;
  model_name: string | null;
  error: string | null;
}
```

#### transcription-complete

转录完成事件。

**负载**:
```typescript
{
  text: string;
  model_name: string | null;
  is_api: boolean;
}
```

#### audio-levels

音频电平事件。

**负载**:
```typescript
{
  levels: number[];  // 音频电平数组
}
```

## 类型定义

### ModelInfo

```typescript
interface ModelInfo {
  id: string;
  name: string;
  description: string;
  filename: string;
  url: string | null;
  size_mb: number;
  is_downloaded: boolean;
  is_downloading: boolean;
  download_progress: number;
  engine_type: EngineType;
}
```

### EngineType

```typescript
type EngineType = 
  | 'Whisper'
  | 'Paraformer'
  | 'Transducer'
  | 'FireRedAsr'
  | 'SeacoParaformer'
  | 'Api';
```

### HistoryEntry

```typescript
interface HistoryEntry {
  id: number;
  text: string;
  timestamp: number;
  audio_file: string | null;
  is_saved: boolean;
}
```

### AudioDevice

```typescript
interface AudioDevice {
  index: string;
  name: string;
  is_default: boolean;
}
```

### AppSettings

应用设置对象，包含所有可配置的设置项。详细结构请参考源代码中的 `AppSettings` 类型定义。

### Result

所有 API 调用返回的结果类型：

```typescript
type Result<T, E> = 
  | { status: 'ok'; data: T }
  | { status: 'error'; error: E };
```

## 错误处理

### 常见错误

1. **模型不存在**: `"Model not found: {model_id}"`
2. **模型未下载**: `"Model not downloaded: {model_id}"`
3. **权限不足**: `"Failed to initialize input system: ..."`
4. **网络错误**: `"Failed to download model: ..."`
5. **文件系统错误**: `"Failed to open directory: ..."`

### 错误处理最佳实践

```typescript
async function safeApiCall() {
  try {
    const result = await commands.someCommand();
    if (result.status === 'error') {
      // 处理错误
      console.error('API Error:', result.error);
      // 显示用户友好的错误消息
      return;
    }
    // 使用成功的数据
    return result.data;
  } catch (error) {
    // 处理意外错误
    console.error('Unexpected error:', error);
  }
}
```

## 性能考虑

### 批量操作

对于需要更新多个设置的操作，建议批量更新而不是逐个调用：

```typescript
// 不推荐
await commands.changeSetting1(value1);
await commands.changeSetting2(value2);
await commands.changeSetting3(value3);

// 推荐：使用 getAppSettings 和更新整个设置对象
const settings = await commands.getAppSettings();
if (settings.status === 'ok') {
  // 更新多个设置
  // ...
}
```

### 事件监听

使用事件监听而不是轮询：

```typescript
// 不推荐：轮询
setInterval(async () => {
  const status = await commands.getModelLoadStatus();
  // ...
}, 1000);

// 推荐：事件监听
await listen('model-state-changed', (event) => {
  // 处理状态变化
});
```

## 版本兼容性

API 接口在不同版本之间可能发生变化。建议：

1. 查看 CHANGELOG 了解 API 变更
2. 使用 TypeScript 类型检查确保类型安全
3. 测试 API 调用以确保兼容性
