# 跨平台动态库加载机制

## 概述

KeVoiceInput 使用 sherpa-rs (vendored) 来集成 sherpa-onnx 语音识别库。动态库加载机制已经完全跨平台，无需额外的平台特定代码。

## 动态库命名约定

| 平台 | 静态库 | 动态库 | 导入库 (Windows) |
|------|--------|--------|------------------|
| Windows | `.lib` (静态) | `.dll` | `.lib` (导入) |
| macOS | `.a` | `.dylib` | N/A |
| Linux | `.a` | `.so` | N/A |

## sherpa-onnx 库文件

### macOS
位置: `$SHERPA_LIB_PATH/`
```
libcargs.dylib
libonnxruntime.1.17.1.dylib
libsherpa-onnx-c-api.dylib
libsherpa-onnx-cxx-api.dylib
```

### Windows
位置: `%SHERPA_LIB_PATH%\`
```
cargs.dll
onnxruntime.dll
sherpa-onnx-c-api.dll
sherpa-onnx-cxx-api.dll
```

对应的导入库 (用于链接):
```
cargs.lib
onnxruntime.lib
sherpa-onnx-c-api.lib
sherpa-onnx-cxx-api.lib
```

### Linux
位置: `$SHERPA_LIB_PATH/`
```
libcargs.so
libonnxruntime.so.1.17.1
libsherpa-onnx-c-api.so
libsherpa-onnx-cxx-api.so
```

## build.rs 自动化处理

`vendor/sherpa-rs/crates/sherpa-rs-sys/build.rs` 处理所有平台的库加载：

### 1. 库名称提取 (行 99-141)

```rust
fn extract_lib_names(out_dir: &Path, is_dynamic: bool, target_os: &str) -> Vec<String> {
    let lib_pattern = if target_os == "windows" {
        "*.lib"  // Windows 导入库
    } else if target_os == "macos" {
        if is_dynamic {
            "*.dylib"
        } else {
            "*.a"
        }
    } else if is_dynamic {
        "*.so"
    } else {
        "*.a"
    };
    // ... 扫描并提取库名称
}
```

### 2. 共享库资源提取 (行 143-167)

```rust
fn extract_lib_assets(out_dir: &Path, target_os: &str) -> Vec<PathBuf> {
    let shared_lib_pattern = if target_os == "windows" {
        "*.dll"
    } else if target_os == "macos" {
        "*.dylib"
    } else {
        "*.so"
    };
    // ... 查找所有共享库文件
}
```

### 3. Windows 特定配置 (行 447-449)

```rust
if target_os == "windows" {
    config.static_crt(static_crt);
}
```

### 4. 动态库自动复制 (行 530-573)

构建时自动将动态库复制到：
- `target/release/` (主可执行文件目录)
- `target/release/examples/` (示例目录)
- `target/release/deps/` (测试依赖目录)

```rust
if is_dynamic {
    let mut libs_assets = extract_lib_assets(&out_dir, &target_os);
    if let Ok(sherpa_lib_path) = env::var("SHERPA_LIB_PATH") {
        libs_assets.extend(extract_lib_assets(Path::new(&sherpa_lib_path), &target_os));
    }

    for asset in libs_assets {
        let filename = asset.file_name().unwrap();
        let dst = target_dir.join(filename);
        if !dst.exists() {
            copy_file(asset.clone(), dst);
        }
        // ... 复制到 examples 和 deps
    }
}
```

## 环境变量

### SHERPA_LIB_PATH

指定 sherpa-onnx 库的位置。

**macOS**:
```bash
export SHERPA_LIB_PATH=/path/to/sherpa-onnx/install/lib
```

**Windows**:
```powershell
$env:SHERPA_LIB_PATH = "C:\path\to\sherpa-onnx\install\bin"
```

**Linux**:
```bash
export SHERPA_LIB_PATH=/path/to/sherpa-onnx/install/lib
```

### DYLD_LIBRARY_PATH (macOS 开发模式)

macOS 开发模式需要设置运行时库搜索路径：

```bash
export DYLD_LIBRARY_PATH=$SHERPA_LIB_PATH:$DYLD_LIBRARY_PATH
```

**注意**: 生产环境的 macOS .app bundle 不需要此变量，因为库会被打包到 `Contents/Frameworks/` 并使用 `@rpath`。

### PATH (Windows 运行时)

Windows 运行时 DLL 搜索顺序：
1. 可执行文件所在目录 (推荐)
2. 系统目录 (`C:\Windows\System32`)
3. `%PATH%` 环境变量中的目录

**推荐方式**: 将 DLL 放在 `.exe` 同目录，无需修改 `PATH`。

## 打包策略

### macOS (.app bundle)

使用 `scripts/copy-dylibs.sh` 后处理脚本：

1. 复制动态库到 `Contents/Frameworks/`:
   ```bash
   KeVoiceInput.app/
   └── Contents/
       ├── MacOS/
       │   └── kevoiceinput
       └── Frameworks/
           ├── libcargs.dylib
           ├── libonnxruntime.1.17.1.dylib
           ├── libsherpa-onnx-c-api.dylib
           └── libsherpa-onnx-cxx-api.dylib
   ```

2. 使用 `install_name_tool` 修改库路径为 `@rpath`:
   ```bash
   install_name_tool -change /old/path/lib.dylib @rpath/lib.dylib binary
   install_name_tool -add_rpath @executable_path/../Frameworks binary
   ```

3. 重新签名所有组件:
   ```bash
   codesign --force --deep -s - KeVoiceInput.app
   ```

### Windows (.exe + MSI)

**方式 1: 可执行文件同目录** (推荐)
```
kevoiceinput.exe
cargs.dll
onnxruntime.dll
sherpa-onnx-c-api.dll
sherpa-onnx-cxx-api.dll
```

**方式 2: MSI 安装到 Program Files**
- MSI 安装程序会将 DLL 和 .exe 安装到同一目录
- 无需修改系统 PATH

**当前实现**: `build.rs` 自动复制 DLL 到可执行文件目录，无需额外步骤。

### Linux (AppImage/deb/rpm)

**AppImage**:
```
AppDir/
├── usr/
│   ├── bin/
│   │   └── kevoiceinput
│   └── lib/
│       ├── libcargs.so
│       ├── libonnxruntime.so.1.17.1
│       ├── libsherpa-onnx-c-api.so
│       └── libsherpa-onnx-cxx-api.so
└── AppRun
```

使用 `$ORIGIN` rpath:
```bash
patchelf --set-rpath '$ORIGIN/../lib' kevoiceinput
```

## 编译 sherpa-onnx

### macOS

```bash
git clone https://github.com/k2-fsa/sherpa-onnx.git
cd sherpa-onnx
mkdir build && cd build

cmake -DCMAKE_BUILD_TYPE=Release \
      -DCMAKE_INSTALL_PREFIX=../install \
      -DSHERPA_ONNX_ENABLE_PYTHON=OFF \
      -DSHERPA_ONNX_ENABLE_TESTS=OFF \
      -DBUILD_SHARED_LIBS=ON \
      ..

make -j$(sysctl -n hw.ncpu)
make install

export SHERPA_LIB_PATH=$(pwd)/../install/lib
```

### Windows

```powershell
git clone https://github.com/k2-fsa/sherpa-onnx.git
cd sherpa-onnx
mkdir build
cd build

# CMake 配置 (Visual Studio 2022)
cmake -G "Visual Studio 17 2022" -A x64 `
      -DCMAKE_BUILD_TYPE=Release `
      -DCMAKE_INSTALL_PREFIX=..\install `
      -DSHERPA_ONNX_ENABLE_PYTHON=OFF `
      -DSHERPA_ONNX_ENABLE_TESTS=OFF `
      -DSHERPA_ONNX_ENABLE_CHECK=OFF `
      -DBUILD_SHARED_LIBS=ON `
      ..

# 构建
cmake --build . --config Release

# 安装
cmake --install . --config Release

# 设置环境变量
$env:SHERPA_LIB_PATH = "$(pwd)\..\install\bin"
```

**注意**: Windows 安装目录是 `install/bin`，而 macOS/Linux 是 `install/lib`。

### Linux

```bash
git clone https://github.com/k2-fsa/sherpa-onnx.git
cd sherpa-onnx
mkdir build && cd build

cmake -DCMAKE_BUILD_TYPE=Release \
      -DCMAKE_INSTALL_PREFIX=../install \
      -DSHERPA_ONNX_ENABLE_PYTHON=OFF \
      -DSHERPA_ONNX_ENABLE_TESTS=OFF \
      -DBUILD_SHARED_LIBS=ON \
      ..

make -j$(nproc)
make install

export SHERPA_LIB_PATH=$(pwd)/../install/lib
```

## 故障排查

### 问题: DLL 未找到 (Windows)

**症状**: 应用启动失败，提示 "无法找到 XXX.dll"

**解决方案**:
```powershell
# 1. 检查 SHERPA_LIB_PATH
echo $env:SHERPA_LIB_PATH
dir $env:SHERPA_LIB_PATH\*.dll

# 2. 手动复制 DLL
Copy-Item "$env:SHERPA_LIB_PATH\*.dll" -Destination ".\src-tauri\target\release\"

# 3. 使用 Dependencies 工具检查
# 下载: https://github.com/lucasg/Dependencies
dependencies.exe .\src-tauri\target\release\kevoiceinput.exe
```

### 问题: dylib 加载失败 (macOS)

**症状**: 应用崩溃，提示 "Library not loaded"

**解决方案**:
```bash
# 1. 检查库是否在 Frameworks
ls -la KeVoiceInput.app/Contents/Frameworks/

# 2. 检查 rpath
otool -L KeVoiceInput.app/Contents/MacOS/kevoiceinput

# 3. 运行修复脚本
./scripts/copy-dylibs.sh src-tauri/target/release/bundle/macos/KeVoiceInput.app

# 4. 开发模式设置 DYLD_LIBRARY_PATH
export DYLD_LIBRARY_PATH=$SHERPA_LIB_PATH:$DYLD_LIBRARY_PATH
```

### 问题: .so 加载失败 (Linux)

**症状**: 应用启动失败，提示 "error while loading shared libraries"

**解决方案**:
```bash
# 1. 检查 LD_LIBRARY_PATH
echo $LD_LIBRARY_PATH

# 2. 临时添加库路径
export LD_LIBRARY_PATH=$SHERPA_LIB_PATH:$LD_LIBRARY_PATH

# 3. 检查 rpath
patchelf --print-rpath kevoiceinput

# 4. 设置 rpath (如果缺失)
patchelf --set-rpath '$ORIGIN:$ORIGIN/../lib' kevoiceinput
```

## 验证清单

### 开发环境

- [ ] `SHERPA_LIB_PATH` 环境变量已设置
- [ ] sherpa-onnx 库文件存在于指定路径
- [ ] Rust 代码编译成功 (`cargo build`)
- [ ] 应用可以启动并加载模型

### 生产构建

**macOS**:
- [ ] .app bundle 包含所有 dylib 在 `Contents/Frameworks/`
- [ ] 二进制的 rpath 指向 `@executable_path/../Frameworks`
- [ ] 代码签名有效
- [ ] DMG 可以在其他 Mac 上安装和运行

**Windows**:
- [ ] 可执行文件目录包含所有 DLL
- [ ] MSI 安装包创建成功
- [ ] 安装后应用可以启动
- [ ] 在干净的 Windows 系统上测试 (无开发工具)

**Linux**:
- [ ] 共享库在正确位置 (AppImage: `usr/lib/`, deb/rpm: 系统路径)
- [ ] rpath 设置正确
- [ ] 在其他 Linux 发行版上测试

## 总结

✅ **动态库加载机制已完全跨平台**

- `build.rs` 自动处理所有平台的库名称、扩展名和复制
- 使用环境变量 `SHERPA_LIB_PATH` 统一配置
- 开发模式和生产构建均已测试
- Windows 支持已内置，无需额外代码

**Windows 适配状态**: ✅ 已完全支持，只需正确设置 `SHERPA_LIB_PATH`

## 相关脚本

- **macOS**: `scripts/copy-dylibs.sh` - 后处理脚本
- **Windows**: `scripts/build-windows.ps1` - 构建脚本 (可选 DLL 复制)
- **Build**: `vendor/sherpa-rs/crates/sherpa-rs-sys/build.rs` - 核心构建逻辑

## 相关文档

- [sherpa-onnx 文档](https://k2-fsa.github.io/sherpa/onnx/)
- [Rust 链接文档](https://doc.rust-lang.org/reference/linkage.html)
- [Windows DLL 搜索顺序](https://docs.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order)
- [macOS dyld 文档](https://developer.apple.com/library/archive/documentation/DeveloperTools/Conceptual/DynamicLibraries/)
