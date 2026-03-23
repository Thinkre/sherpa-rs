// 测试 sherpa-onnx API 是否正常工作的独立测试程序
// 运行方式: cargo run --bin test_sherpa_api -- <model_dir>

use anyhow::Result;
use sherpa_rs_sys::*;
use std::env;
use std::ffi::{CStr, CString};
use std::mem;
use std::path::Path;

fn main() -> Result<()> {
    env_logger::init();
    
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("用法: cargo run --bin test_sherpa_api -- <model_dir>");
        eprintln!("例如: cargo run --bin test_sherpa_api -- ~/Library/Application\\ Support/com.kevoiceinput.app/models/conformer-zh-stateless2");
        std::process::exit(1);
    }
    
    let model_dir = Path::new(&args[1]);
    println!("测试模型目录: {:?}", model_dir);
    
    // 测试 1: 验证库是否加载
    println!("\n=== 测试 1: 验证库是否加载 ===");
    test_library_loaded()?;
    
    // 测试 2: 验证基本函数指针
    println!("\n=== 测试 2: 验证基本函数指针 ===");
    test_function_pointers()?;
    
    // 测试 3: 创建简单的 recognizer（如果模型文件存在）
    if model_dir.exists() {
        println!("\n=== 测试 3: 创建 Recognizer ===");
        test_create_recognizer(model_dir)?;
    } else {
        println!("\n=== 测试 3: 跳过（模型目录不存在）===");
    }
    
    println!("\n✅ 所有测试完成！");
    Ok(())
}

fn test_library_loaded() -> Result<()> {
    println!("检查库是否已加载...");
    
    unsafe {
        let create_fn = SherpaOnnxCreateOfflineRecognizer as *const ();
        if create_fn.is_null() {
            return Err(anyhow::anyhow!("库函数指针为 null，库可能未正确加载"));
        }
        println!("✅ 库函数指针有效: {:p}", create_fn);
    }
    
    // 检查环境变量
    println!("环境变量:");
    println!("  SHERPA_LIB_PATH: {:?}", env::var("SHERPA_LIB_PATH").ok());
    println!("  DYLD_LIBRARY_PATH: {:?}", env::var("DYLD_LIBRARY_PATH").ok());
    println!("  LD_LIBRARY_PATH: {:?}", env::var("LD_LIBRARY_PATH").ok());
    
    Ok(())
}

fn test_function_pointers() -> Result<()> {
    println!("检查关键函数指针...");
    
    unsafe {
        let functions = [
            ("SherpaOnnxCreateOfflineRecognizer", SherpaOnnxCreateOfflineRecognizer as *const ()),
            ("SherpaOnnxDestroyOfflineRecognizer", SherpaOnnxDestroyOfflineRecognizer as *const ()),
            ("SherpaOnnxCreateOfflineStream", SherpaOnnxCreateOfflineStream as *const ()),
            ("SherpaOnnxDestroyOfflineStream", SherpaOnnxDestroyOfflineStream as *const ()),
            ("SherpaOnnxAcceptWaveformOffline", SherpaOnnxAcceptWaveformOffline as *const ()),
            ("SherpaOnnxDecodeOfflineStream", SherpaOnnxDecodeOfflineStream as *const ()),
            ("SherpaOnnxGetOfflineStreamResult", SherpaOnnxGetOfflineStreamResult as *const ()),
        ];
        
        for (name, ptr) in &functions {
            if ptr.is_null() {
                return Err(anyhow::anyhow!("函数 {} 的指针为 null", name));
            }
            println!("✅ {}: {:p}", name, ptr);
        }
    }
    
    Ok(())
}

fn test_create_recognizer(model_dir: &Path) -> Result<()> {
    println!("尝试创建 Recognizer...");
    
    // 自动检测模型类型
    let model_type = detect_model_type(model_dir)?;
    println!("检测到的模型类型: {:?}", model_type);
    
    match model_type {
        ModelType::Transducer => test_transducer_recognizer(model_dir),
        ModelType::Paraformer => {
            let has_model_eb = model_dir.join("model_eb.onnx").exists();
            test_paraformer_recognizer(model_dir, has_model_eb)
        }
        ModelType::FireRedAsr => test_fireredasr_recognizer(model_dir),
        ModelType::Unknown => Err(anyhow::anyhow!("无法识别模型类型")),
    }
}

#[derive(Debug)]
enum ModelType {
    Transducer,
    Paraformer,  // Includes SeACo (model_eb.onnx) when present
    FireRedAsr,
    Unknown,
}

fn detect_model_type(model_dir: &Path) -> Result<ModelType> {
    // 检查 Transducer 模型（encoder, decoder, joiner）
    let has_encoder = model_dir.join("encoder-epoch-99-avg-1.onnx").exists()
        || model_dir.join("encoder-epoch-34-avg-19.onnx").exists()
        || model_dir.join("encoder.int8.onnx").exists()
        || model_dir.join("encoder.onnx").exists();
    let has_decoder = model_dir.join("decoder-epoch-99-avg-1.onnx").exists()
        || model_dir.join("decoder-epoch-34-avg-19.onnx").exists()
        || model_dir.join("decoder.int8.onnx").exists()
        || model_dir.join("decoder.onnx").exists();
    let has_joiner = model_dir.join("joiner-epoch-99-avg-1.onnx").exists()
        || model_dir.join("joiner-epoch-34-avg-19.onnx").exists()
        || model_dir.join("joiner.int8.onnx").exists()
        || model_dir.join("joiner.onnx").exists();
    
    // 检查 FireRedAsr（encoder.int8.onnx, decoder.int8.onnx）
    let has_fireredasr_encoder = model_dir.join("encoder.int8.onnx").exists();
    let has_fireredasr_decoder = model_dir.join("decoder.int8.onnx").exists();
    
    // 检查 Paraformer（model.onnx，可选 model_eb.onnx）
    let has_paraformer = model_dir.join("model.onnx").exists();
    
    if has_fireredasr_encoder && has_fireredasr_decoder {
        return Ok(ModelType::FireRedAsr);
    }
    
    if has_encoder && has_decoder && has_joiner {
        return Ok(ModelType::Transducer);
    }
    
    if has_paraformer {
        return Ok(ModelType::Paraformer);
    }
    
    Ok(ModelType::Unknown)
}

fn test_transducer_recognizer(model_dir: &Path) -> Result<()> {
    println!("测试 Transducer 模型...");
    
    // 查找 encoder 文件
    let encoder_file = find_file(model_dir, &["encoder-epoch-99-avg-1.onnx", "encoder-epoch-34-avg-19.onnx", "encoder.onnx"])
        .ok_or_else(|| anyhow::anyhow!("找不到 encoder 文件"))?;
    
    // 查找 decoder 文件
    let decoder_file = find_file(model_dir, &["decoder-epoch-99-avg-1.onnx", "decoder-epoch-34-avg-19.onnx", "decoder.onnx"])
        .ok_or_else(|| anyhow::anyhow!("找不到 decoder 文件"))?;
    
    // 查找 joiner 文件
    let joiner_file = find_file(model_dir, &["joiner-epoch-99-avg-1.onnx", "joiner-epoch-34-avg-19.onnx", "joiner.onnx"])
        .ok_or_else(|| anyhow::anyhow!("找不到 joiner 文件"))?;
    
    let tokens_file = model_dir.join("tokens.txt");
    
    println!("检查模型文件:");
    println!("  encoder: {:?} (存在: {})", encoder_file, encoder_file.exists());
    println!("  decoder: {:?} (存在: {})", decoder_file, decoder_file.exists());
    println!("  joiner: {:?} (存在: {})", joiner_file, joiner_file.exists());
    println!("  tokens: {:?} (存在: {})", tokens_file, tokens_file.exists());
    
    if !tokens_file.exists() {
        return Err(anyhow::anyhow!("缺少 tokens.txt 文件"));
    }
    
    
    // 创建 CString（必须保持有效直到函数调用完成）
    let encoder_path = CString::new(encoder_file.to_str().unwrap())?;
    let decoder_path = CString::new(decoder_file.to_str().unwrap())?;
    let joiner_path = CString::new(joiner_file.to_str().unwrap())?;
    let tokens_path = CString::new(tokens_file.to_str().unwrap())?;
    let provider = CString::new("cpu")?;
    let model_type_str = CString::new("transducer")?;
    let modeling_unit = CString::new("cjkchar")?;
    let decoding_method = CString::new("modified_beam_search")?;
    
    // 配置特征提取
    let feat_config = SherpaOnnxFeatureConfig {
        sample_rate: 16000,
        feature_dim: 80,
    };
    
    // 配置 Transducer 模型（使用 zeroed 初始化）
    let mut transducer_config: SherpaOnnxOfflineTransducerModelConfig = unsafe { mem::zeroed() };
    transducer_config.encoder = encoder_path.as_ptr();
    transducer_config.decoder = decoder_path.as_ptr();
    transducer_config.joiner = joiner_path.as_ptr();
    
    // 配置模型（使用 zeroed 初始化，然后设置字段）
    let mut model_config: SherpaOnnxOfflineModelConfig = unsafe { mem::zeroed() };
    model_config.transducer = transducer_config;
    model_config.tokens = tokens_path.as_ptr();
    model_config.num_threads = 1;
    model_config.debug = 0; // 使用 0 而不是 1，与实际代码一致
    model_config.provider = provider.as_ptr();
    model_config.model_type = model_type_str.as_ptr();
    model_config.modeling_unit = modeling_unit.as_ptr();
    // 显式设置未使用的指针字段为 NULL
    model_config.bpe_vocab = std::ptr::null();
    model_config.telespeech_ctc = std::ptr::null();
    // 其他模型类型的字段已经通过 zeroed() 初始化为零
    
    // 配置 Recognizer（使用 zeroed 初始化，然后设置字段）
    let mut recognizer_config: SherpaOnnxOfflineRecognizerConfig = unsafe { mem::zeroed() };
    recognizer_config.feat_config = feat_config;
    recognizer_config.model_config = model_config;
    recognizer_config.decoding_method = decoding_method.as_ptr();
    recognizer_config.max_active_paths = 4;
    recognizer_config.blank_penalty = 0.0;
    // 显式设置指针字段为 NULL
    recognizer_config.hotwords_file = std::ptr::null();
    recognizer_config.hotwords_score = 0.0; // 没有 hotwords 时设为 0.0
    recognizer_config.rule_fsts = std::ptr::null();
    recognizer_config.rule_fars = std::ptr::null();
    // 显式初始化 lm_config 结构体字段
    recognizer_config.lm_config.model = std::ptr::null();
    recognizer_config.lm_config.scale = 0.0;
    // 显式初始化 hr (HomophoneReplacer) 结构体字段
    recognizer_config.hr.dict_dir = std::ptr::null();
    recognizer_config.hr.lexicon = std::ptr::null();
    recognizer_config.hr.rule_fsts = std::ptr::null();
    
    // 打印配置信息用于调试
    println!("配置信息:");
    println!("  encoder: {:?}", unsafe { std::ffi::CStr::from_ptr(recognizer_config.model_config.transducer.encoder) });
    println!("  decoder: {:?}", unsafe { std::ffi::CStr::from_ptr(recognizer_config.model_config.transducer.decoder) });
    println!("  joiner: {:?}", unsafe { std::ffi::CStr::from_ptr(recognizer_config.model_config.transducer.joiner) });
    println!("  tokens: {:?}", unsafe { std::ffi::CStr::from_ptr(recognizer_config.model_config.tokens) });
    println!("  provider: {:?}", unsafe { std::ffi::CStr::from_ptr(recognizer_config.model_config.provider) });
    println!("  model_type: {:?}", unsafe { std::ffi::CStr::from_ptr(recognizer_config.model_config.model_type) });
    println!("  modeling_unit: {:?}", unsafe { std::ffi::CStr::from_ptr(recognizer_config.model_config.modeling_unit) });
    println!("  decoding_method: {:?}", unsafe { std::ffi::CStr::from_ptr(recognizer_config.decoding_method) });
    println!("  num_threads: {}", recognizer_config.model_config.num_threads);
    println!("  debug: {}", recognizer_config.model_config.debug);
    println!("  max_active_paths: {}", recognizer_config.max_active_paths);
    
    // 验证模型文件存在（与实际代码一致）
    println!("验证模型文件可读...");
    if let Ok(metadata) = std::fs::metadata(&encoder_file) {
        println!("  encoder 文件大小: {} 字节", metadata.len());
    }
    if let Ok(metadata) = std::fs::metadata(&decoder_file) {
        println!("  decoder 文件大小: {} 字节", metadata.len());
    }
    if let Ok(metadata) = std::fs::metadata(&joiner_file) {
        println!("  joiner 文件大小: {} 字节", metadata.len());
    }
    if let Ok(metadata) = std::fs::metadata(&tokens_file) {
        println!("  tokens 文件大小: {} 字节", metadata.len());
    }
    
    // 调用函数（CString 在函数返回前必须保持有效）
    // 注意：CString 会在函数返回后自动 drop，但此时 recognizer 已经创建完成
    create_and_test_recognizer(&recognizer_config)
    
    // CString 会在这里自动 drop，但此时 recognizer 已经创建并销毁完成
}

fn test_paraformer_recognizer(model_dir: &Path, is_seaco: bool) -> Result<()> {
    if is_seaco {
        println!("测试 SeACo Paraformer 模型（包含 model_eb.onnx）...");
    } else {
        println!("测试 Paraformer 模型...");
    }
    
    let model_file = model_dir.join("model.onnx");
    let tokens_file = model_dir.join("tokens.txt");
    let model_eb_file = model_dir.join("model_eb.onnx");
    
    println!("检查模型文件:");
    println!("  model.onnx: {:?} (存在: {})", model_file, model_file.exists());
    println!("  tokens.txt: {:?} (存在: {})", tokens_file, tokens_file.exists());
    if is_seaco {
        println!("  model_eb.onnx: {:?} (存在: {})", model_eb_file, model_eb_file.exists());
    }
    
    if !model_file.exists() {
        return Err(anyhow::anyhow!("缺少 model.onnx 文件"));
    }
    if !tokens_file.exists() {
        return Err(anyhow::anyhow!("缺少 tokens.txt 文件"));
    }
    if is_seaco && !model_eb_file.exists() {
        return Err(anyhow::anyhow!("SeACo Paraformer 需要 model_eb.onnx 文件"));
    }
    
    // 创建 CString
    let model_path = CString::new(model_file.to_str().unwrap())?;
    let tokens_path = CString::new(tokens_file.to_str().unwrap())?;
    let model_eb_path = if is_seaco {
        Some(CString::new(model_eb_file.to_str().unwrap())?)
    } else {
        None
    };
    let provider = CString::new("cpu")?;
    let model_type_str = CString::new("paraformer")?;
    let decoding_method = CString::new("greedy_search")?;
    
    // 配置 Paraformer 模型
    let mut paraformer_config: SherpaOnnxOfflineParaformerModelConfig = unsafe { mem::zeroed() };
    paraformer_config.model = model_path.as_ptr();
    
    // 设置 model_eb（如果 bindings.rs 包含此字段）
    if is_seaco {
        if let Some(eb_path) = &model_eb_path {
            paraformer_config.model_eb = eb_path.as_ptr();
            println!("  设置 model_eb: {:?}", unsafe { CStr::from_ptr(eb_path.as_ptr()) });
        }
    } else {
        // 标准 Paraformer 不需要 model_eb，设置为 null
        paraformer_config.model_eb = std::ptr::null();
    }
    
    // 配置模型
    let mut model_config: SherpaOnnxOfflineModelConfig = unsafe { mem::zeroed() };
    model_config.paraformer = paraformer_config;
    model_config.tokens = tokens_path.as_ptr();
    model_config.num_threads = 1;
    model_config.debug = 0;
    model_config.provider = provider.as_ptr();
    model_config.model_type = model_type_str.as_ptr();
    // 显式设置未使用的指针字段为 NULL
    model_config.bpe_vocab = std::ptr::null();
    model_config.telespeech_ctc = std::ptr::null();
    
    // 配置 Recognizer
    let mut recognizer_config: SherpaOnnxOfflineRecognizerConfig = unsafe { mem::zeroed() };
    recognizer_config.model_config = model_config;
    recognizer_config.decoding_method = decoding_method.as_ptr();
    recognizer_config.max_active_paths = 4;
    recognizer_config.blank_penalty = 0.0;
    recognizer_config.hotwords_file = std::ptr::null();
    recognizer_config.hotwords_score = 0.0;
    recognizer_config.rule_fsts = std::ptr::null();
    recognizer_config.rule_fars = std::ptr::null();
    recognizer_config.lm_config.model = std::ptr::null();
    recognizer_config.lm_config.scale = 0.0;
    recognizer_config.hr.dict_dir = std::ptr::null();
    recognizer_config.hr.lexicon = std::ptr::null();
    recognizer_config.hr.rule_fsts = std::ptr::null();
    
    // 打印配置信息
    println!("配置信息:");
    println!("  model: {:?}", unsafe { CStr::from_ptr(recognizer_config.model_config.paraformer.model) });
    if is_seaco {
        let eb_ptr = recognizer_config.model_config.paraformer.model_eb;
        if eb_ptr.is_null() {
            println!("  model_eb: NULL");
        } else {
            println!("  model_eb: {:?}", unsafe { CStr::from_ptr(eb_ptr) });
        }
    }
    println!("  tokens: {:?}", unsafe { CStr::from_ptr(recognizer_config.model_config.tokens) });
    println!("  provider: {:?}", unsafe { CStr::from_ptr(recognizer_config.model_config.provider) });
    println!("  model_type: {:?}", unsafe { CStr::from_ptr(recognizer_config.model_config.model_type) });
    println!("  decoding_method: {:?}", unsafe { CStr::from_ptr(recognizer_config.decoding_method) });
    println!("  num_threads: {}", recognizer_config.model_config.num_threads);
    println!("  debug: {}", recognizer_config.model_config.debug);
    
    // 验证文件大小
    println!("验证模型文件可读...");
    if let Ok(metadata) = std::fs::metadata(&model_file) {
        println!("  model.onnx 文件大小: {} 字节", metadata.len());
    }
    if let Ok(metadata) = std::fs::metadata(&tokens_file) {
        println!("  tokens.txt 文件大小: {} 字节", metadata.len());
    }
    if is_seaco {
        if let Ok(metadata) = std::fs::metadata(&model_eb_file) {
            println!("  model_eb.onnx 文件大小: {} 字节", metadata.len());
        }
    }
    
    // 调用函数创建 recognizer
    create_and_test_recognizer(&recognizer_config)
}

fn test_fireredasr_recognizer(model_dir: &Path) -> Result<()> {
    println!("测试 FireRedAsr 模型...");
    println!("⚠️  FireRedAsr 模型测试尚未实现");
    Err(anyhow::anyhow!("FireRedAsr 模型测试尚未实现"))
}

fn find_file(dir: &Path, patterns: &[&str]) -> Option<std::path::PathBuf> {
    for pattern in patterns {
        let path = dir.join(pattern);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

fn create_and_test_recognizer(recognizer_config: &SherpaOnnxOfflineRecognizerConfig) -> Result<()> {
    use std::io::Write;
    
    println!("调用 SherpaOnnxCreateOfflineRecognizer...");
    println!("配置结构体大小: {} 字节", std::mem::size_of::<SherpaOnnxOfflineRecognizerConfig>());
    println!("配置结构体地址: {:p}", recognizer_config);
    
    // 刷新输出以确保日志可见
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().flush();
    
    // 添加小延迟以确保输出已写入
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    unsafe {
        println!("进入 unsafe 块...");
        let _ = std::io::stdout().flush();
        let _ = std::io::stderr().flush();
        
        let config_ptr = recognizer_config as *const _;
        println!("配置指针: {:p}", config_ptr);
        let _ = std::io::stdout().flush();
        
        println!("调用 SherpaOnnxCreateOfflineRecognizer...");
        let _ = std::io::stdout().flush();
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        let recognizer = SherpaOnnxCreateOfflineRecognizer(recognizer_config);
        
        let _ = std::io::stdout().flush();
        let _ = std::io::stderr().flush();
        
        if recognizer.is_null() {
            return Err(anyhow::anyhow!("创建 Recognizer 失败，返回 null"));
        }
        
        println!("✅ Recognizer 创建成功: {:p}", recognizer);
        let _ = std::io::stdout().flush();
        
        // 测试创建 stream
        println!("测试创建 Stream...");
        let stream = SherpaOnnxCreateOfflineStream(recognizer);
        if stream.is_null() {
            SherpaOnnxDestroyOfflineRecognizer(recognizer);
            return Err(anyhow::anyhow!("创建 Stream 失败"));
        }
        println!("✅ Stream 创建成功: {:p}", stream);
        
        // 清理
        println!("清理资源...");
        SherpaOnnxDestroyOfflineStream(stream);
        SherpaOnnxDestroyOfflineRecognizer(recognizer);
        println!("✅ 资源清理完成");
    }
    
    Ok(())
}
