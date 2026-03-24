  1. sherpa-rs 的 mem::zeroed() 导致 null 指针崩溃：vendor 里的 sherpa-rs 在构建 SherpaOnnxOfflineRecognizerConfig 时，用 mem::zeroed() 初始化未使用的模型配置子结构体，所有 char* 字段变成 null。sherpa-onnx C++ 端的 GetOfflineRecognizerConfig 对这些字段无条件调用
  strlen()，导致 SIGSEGV。dev 模式不崩溃是因为本地编译的 sherpa-onnx 有 SHERPA_ONNX_OR(field, "") 宏做了 null 保护。修复方式是在 lib.rs 里加了 safe_default_offline_model_config 和 safe_default_offline_recognizer_config 辅助函数，把所有 char* 字段设为空字符串。
  2. dylib 版本不匹配和打包问题：sherpa-rs-sys 默认启用了 download-binaries feature，从缓存下载了旧的预编译包（sherpa-onnx v1.12.9 + onnxruntime 1.17.1），而不是从 vendor 源码编译。这个旧版本没有 model_eb 支持，且与项目不兼容。修复方式是禁用 download-binaries，并把参考
   DMG 里验证过的 dylib 放到 vendor/libs/macos-arm64/，由 copy-dylibs.sh 直接复制到 app bundle，同时用 install_name_tool 修正 @rpath 为 @executable_path/../Frameworks/。
