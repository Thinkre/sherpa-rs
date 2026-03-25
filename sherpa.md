现在我们可以清楚地看到KeVoiceInput中包含自定义修改的主要文件：
关键的 sherpa-onnx 修改文件:
1. csrc/offline-recognizer.cc - 添加对 SeACo-Paraformer 模式识别的支持
2. csrc/offline-paraformer-model-config.h - 添加 model_eb 字段定义
3. csrc/offline-paraformer-model-config.cc - 添加 model_eb 参数处理
4. csrc/offline-paraformer-model.cc - 实现对 model_eb.onnx 文件加载逻辑
5. c-api/c-api.h - API 层添加 model_eb 
6. c-api/c-api.cc - API 层处理 model_eb
7. python/csrc/offline-paraformer-model-config.cc - Python绑定添加model_eb
8. python/sherpa_onnx/offline_recognizer.py - Python API 添加model_eb支持
关键的 sherpa-rs 修改文件:
1. src/lib.rs - 为 OfflineModelConfig 添加 model_eb 场景
2. src/paraformer.rs - 添加对 model_eb 的 Rust 绑定和处理
