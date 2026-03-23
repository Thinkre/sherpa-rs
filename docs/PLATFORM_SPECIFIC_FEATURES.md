# 平台特定功能管理

## 概述

KeVoiceInput 的某些功能仅在特定平台上可用。本文档说明如何正确隐藏和管理这些平台特定功能。

## Apple Intelligence

### 功能说明

**Apple Intelligence** 是 macOS 15+ 上的本地 LLM 服务，可用于文本后处理。

**平台要求**:
- ✅ macOS 15.0+ (Sequoia)
- ✅ Apple Silicon (ARM64)
- ❌ Intel Mac
- ❌ Windows
- ❌ Linux

### 实现策略

#### 1. 后端条件编译

`src-tauri/src/settings.rs:600-614`:

```rust
fn default_post_process_providers() -> Vec<PostProcessProvider> {
    let mut providers = vec![
        // ... 其他提供商 ...
    ];

    // 仅在 macOS ARM64 上添加 Apple Intelligence
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        providers.push(PostProcessProvider {
            id: APPLE_INTELLIGENCE_PROVIDER_ID.to_string(),
            label: "Apple Intelligence".to_string(),
            base_url: "apple-intelligence://local".to_string(),
            allow_base_url_edit: false,
            models_endpoint: None,
        });
    }

    // Custom provider 总是最后
    providers.push(PostProcessProvider {
        id: "custom".to_string(),
        label: "Custom".to_string(),
        base_url: "http://localhost:11434/v1".to_string(),
        allow_base_url_edit: true,
        models_endpoint: Some("/models".to_string()),
    });

    providers
}
```

**关键点**:
- 使用 `#[cfg(all(target_os = "macos", target_arch = "aarch64"))]` 条件编译
- 非 ARM64 Mac 和其他平台不会在提供商列表中看到 Apple Intelligence
- 提供商列表在应用启动时生成，之后不会改变

#### 2. 可用性检查命令

`src-tauri/src/commands/mod.rs:157-170`:

```rust
/// 检查 Apple Intelligence 是否可用
#[specta::specta]
#[tauri::command]
pub fn check_apple_intelligence_available() -> bool {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        crate::apple_intelligence::check_apple_intelligence_availability()
    }

    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    {
        false  // 非 macOS ARM64 总是返回 false
    }
}
```

**行为**:
- **macOS ARM64**: 调用原生 API 检查实际可用性（取决于系统版本和配置）
- **其他平台**: 直接返回 `false`，无需运行时检查

#### 3. 原生 API 集成

`src-tauri/src/apple_intelligence.rs:23-25`:

```rust
pub fn check_apple_intelligence_availability() -> bool {
    unsafe { is_apple_intelligence_available() == 1 }
}
```

调用 Objective-C 原生 API (`apple_intelligence.m`):

```objc
// 检查 SystemLanguageModel 是否可用（macOS 15.1+）
BOOL is_apple_intelligence_available() {
    if (@available(macOS 15.1, *)) {
        @try {
            // 尝试访问 SystemLanguageModel
            Class slmClass = NSClassFromString(@"SystemLanguageModel");
            if (slmClass == nil) {
                return NO;
            }
            return YES;
        } @catch (NSException *exception) {
            return NO;
        }
    }
    return NO;
}
```

#### 4. 运行时安全检查

`src-tauri/src/actions.rs:850-856`:

```rust
if provider.id == APPLE_INTELLIGENCE_PROVIDER_ID {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        // 再次检查可用性，防止系统更新后失效
        if !apple_intelligence::check_apple_intelligence_availability() {
            debug!("Apple Intelligence selected but not currently available");
            return None;
        }

        let token_limit = model.trim().parse::<i32>().unwrap_or(0);
        return match apple_intelligence::process_text(&processed_prompt, token_limit) {
            Ok(result) => Some(result),
            Err(e) => {
                error!("Apple Intelligence processing failed: {}", e);
                None
            }
        };
    }

    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    {
        // 不应该到达这里，但作为安全措施
        error!("Apple Intelligence requested on unsupported platform");
        return None;
    }
}
```

#### 5. 前端自适应 UI

`src/components/settings/PostProcessingSettingsApi/usePostProcessProviderState.ts:78-97`:

```typescript
const handleProviderSelect = useCallback(
  async (providerId: string) => {
    // 清除之前的错误状态
    setAppleIntelligenceUnavailable(false);

    if (providerId === selectedProviderId) return;

    // 如果选择 Apple Intelligence，检查可用性
    if (providerId === APPLE_PROVIDER_ID) {
      const available = await commands.checkAppleIntelligenceAvailable();
      if (!available) {
        setAppleIntelligenceUnavailable(true);
        // 仍然设置提供商，让后端优雅处理
      }
    }

    void setPostProcessProvider(providerId);
  },
  [selectedProviderId, setPostProcessProvider],
);
```

**UI 反馈**:
```typescript
{state.appleIntelligenceUnavailable && (
  <div className="text-sm text-yellow-600">
    {t("settings.postProcessing.api.appleIntelligence.unavailable")}
  </div>
)}
```

### 用户体验

| 平台 | 提供商列表 | 选择后行为 |
|------|-----------|----------|
| macOS ARM64 (15.1+) | ✅ 显示 Apple Intelligence | ✅ 正常工作 |
| macOS ARM64 (<15.1) | ✅ 显示 Apple Intelligence | ⚠️ 显示不可用提示 |
| macOS Intel | ❌ 不显示 | N/A |
| Windows | ❌ 不显示 | N/A |
| Linux | ❌ 不显示 | N/A |

### 测试

#### macOS ARM64 测试

```bash
# 1. 检查是否出现在提供商列表
# 前端设置 → 后处理 → LLM 提供商下拉菜单应包含 "Apple Intelligence"

# 2. 测试可用性检查
# 打开 Web Inspector，执行：
window.__TAURI__.invoke('check_apple_intelligence_available')
  .then(available => console.log('Available:', available));

# 3. 测试实际处理
# 选择 Apple Intelligence 提供商
# 启用后处理
# 进行语音转录
# 检查日志输出
```

#### 其他平台测试

```bash
# 验证不出现在提供商列表
# 前端设置 → 后处理 → LLM 提供商下拉菜单不应包含 "Apple Intelligence"

# 验证命令返回 false
window.__TAURI__.invoke('check_apple_intelligence_available')
  .then(available => console.log('Available:', available));  // 应返回 false
```

## 其他平台特定功能

### 辅助功能权限 (macOS)

**功能**: 键盘输入模拟需要辅助功能权限。

**平台要求**:
- ✅ macOS (必需)
- ⚠️ Windows (通常无需，某些场景需管理员权限)
- ⚠️ Linux (通常无需，Wayland 可能受限)

**实现**: 见 [CROSS_PLATFORM_PERMISSIONS.md](CROSS_PLATFORM_PERMISSIONS.md)

### 托盘菜单

**功能**: 系统托盘图标和菜单。

**平台差异**:
- macOS: 菜单栏（顶部）
- Windows: 系统托盘（右下角）
- Linux: 取决于桌面环境

**实现**: Tauri 自动处理平台差异。

### 自动启动

**功能**: 系统启动时自动运行应用。

**平台实现**:
- macOS: Launch Agents (`~/Library/LaunchAgents`)
- Windows: 注册表 (`HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`)
- Linux: XDG Autostart (`~/.config/autostart`)

**实现**: 使用 `tauri-plugin-autostart`，自动处理平台差异。

## 添加新的平台特定功能

### 步骤

#### 1. 后端条件编译

```rust
// src-tauri/src/my_feature.rs

#[cfg(target_os = "macos")]
pub fn my_feature_impl() -> Result<String, String> {
    // macOS 实现
    Ok("macOS result".to_string())
}

#[cfg(target_os = "windows")]
pub fn my_feature_impl() -> Result<String, String> {
    // Windows 实现
    Ok("Windows result".to_string())
}

#[cfg(target_os = "linux")]
pub fn my_feature_impl() -> Result<String, String> {
    // Linux 实现
    Ok("Linux result".to_string())
}
```

#### 2. 导出 Tauri 命令

```rust
// src-tauri/src/commands/mod.rs

#[specta::specta]
#[tauri::command]
pub fn my_feature() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        crate::my_feature::my_feature_impl()
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err("Feature not available on this platform".to_string())
    }
}
```

#### 3. 前端适配

```typescript
// src/components/MyFeature.tsx
import { type } from "@tauri-apps/plugin-os";
import { commands } from "@/bindings";

export const MyFeature: React.FC = () => {
  const [osType] = useState(type());

  // 非 macOS 不显示
  if (osType !== "macos") {
    return null;
  }

  const handleClick = async () => {
    try {
      const result = await commands.myFeature();
      console.log(result);
    } catch (error) {
      console.error("Feature not available:", error);
    }
  };

  return (
    <button onClick={handleClick}>
      Use Platform Feature
    </button>
  );
};
```

#### 4. 可选：运行时检查

对于需要特定系统版本或配置的功能：

```rust
#[tauri::command]
pub fn check_feature_available() -> bool {
    #[cfg(target_os = "macos")]
    {
        // 检查 macOS 版本
        // 检查系统配置
        // ...
        true
    }

    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}
```

### 最佳实践

1. **编译时排除优先**: 尽可能使用 `#[cfg]` 条件编译，减少运行时开销
2. **多层防御**: 编译时 + 运行时双重检查，提高健壮性
3. **优雅降级**: 功能不可用时提供友好的提示信息
4. **UI 自适应**: 不可用的功能不应显示在 UI 中
5. **文档说明**: 在用户文档中明确说明平台限制

## 条件编译参考

### 操作系统

```rust
#[cfg(target_os = "macos")]     // macOS
#[cfg(target_os = "windows")]   // Windows
#[cfg(target_os = "linux")]     // Linux
#[cfg(target_os = "ios")]       // iOS (移动)
#[cfg(target_os = "android")]   // Android (移动)
```

### 架构

```rust
#[cfg(target_arch = "x86_64")]  // x86-64 (Intel/AMD)
#[cfg(target_arch = "aarch64")] // ARM64 (Apple Silicon, ARM服务器)
#[cfg(target_arch = "x86")]     // 32位 x86
```

### 组合条件

```rust
// macOS ARM64 (Apple Silicon)
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]

// 非 macOS
#[cfg(not(target_os = "macos"))]

// Windows 或 Linux
#[cfg(any(target_os = "windows", target_os = "linux"))]

// macOS 或 iOS (所有 Apple 平台)
#[cfg(any(target_os = "macos", target_os = "ios"))]
```

### Feature Flags

```rust
// Cargo.toml
[features]
my-feature = []

// 代码中
#[cfg(feature = "my-feature")]
pub fn my_feature_impl() { }
```

## 前端平台检测

### 使用 @tauri-apps/plugin-os

```typescript
import { type, arch, platform } from "@tauri-apps/plugin-os";

const osType = type();        // "macos" | "windows" | "linux"
const osArch = arch();        // "x86_64" | "aarch64"
const platformInfo = platform(); // "macos" | "windows" | "linux"

// 条件渲染
if (osType === "macos" && osArch === "aarch64") {
  // 显示 Apple Silicon 特定功能
}
```

### 常见检测模式

```typescript
// 检测 macOS
const isMacOS = type() === "macos";

// 检测 Windows
const isWindows = type() === "windows";

// 检测 Linux
const isLinux = type() === "linux";

// 检测 Apple Silicon
const isAppleSilicon = type() === "macos" && arch() === "aarch64";

// 检测 Intel Mac
const isIntelMac = type() === "macos" && arch() === "x86_64";
```

## 总结

✅ **平台特定功能管理完善**

**关键策略**:
- 使用条件编译在二进制级别排除不支持的功能
- 运行时检查确保功能实际可用
- 前端根据平台动态调整 UI
- 提供友好的错误提示

**Apple Intelligence 状态**: ✅ 已正确隐藏在非 macOS ARM64 平台
- 后端: 条件编译 + 可用性检查
- 前端: 自适应提供商列表 + 运行时验证
- 用户体验: 无感知，非支持平台不显示

## 相关文件

### 后端
- `src-tauri/src/apple_intelligence.rs` - Apple Intelligence 原生集成
- `src-tauri/src/apple_intelligence.m` - Objective-C 实现
- `src-tauri/src/commands/mod.rs` - 平台检查命令
- `src-tauri/src/settings.rs` - 提供商列表配置

### 前端
- `src/components/settings/PostProcessingSettingsApi/usePostProcessProviderState.ts` - 提供商状态管理
- `src/components/settings/post-processing/PostProcessingSettings.tsx` - 后处理设置 UI

## 相关文档

- [CROSS_PLATFORM_PERMISSIONS.md](CROSS_PLATFORM_PERMISSIONS.md) - 权限管理
- [Rust 条件编译](https://doc.rust-lang.org/reference/conditional-compilation.html)
- [Tauri Platform APIs](https://tauri.app/v1/api/js/modules/os/)
