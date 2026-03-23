use crate::{get_default_provider, utils::cstring_from_str};
use eyre::{bail, Result};
use std::ffi::CString;
use std::{mem, ptr::null};
use sherpa_rs_sys::SherpaOnnxOfflineRecognizer;

#[derive(Debug)]
pub struct ParaformerRecognizer {
    recognizer: *mut SherpaOnnxOfflineRecognizer,
    _model_cstr: CString,
    _tokens_cstr: CString,
    _model_eb_cstr: Option<CString>,  // 新增：保存 model_eb CString
    _hotwords_file_cstr: Option<CString>,  // 新增：保存 hotwords_file CString
}

pub type ParaformerRecognizerResult = super::OfflineRecognizerResult;

#[derive(Debug, Clone)]
pub struct ParaformerConfig {
    pub model: String,
    pub tokens: String,
    pub model_eb: Option<String>,  // 新增：SeACo Paraformer 嵌入模型路径
    pub hotwords_file: Option<String>,  // 新增：热词文件路径
    pub hotwords_score: f32,  // 新增：热词分数
    pub provider: Option<String>,
    pub num_threads: Option<i32>,
    pub debug: bool,
}

impl Default for ParaformerConfig {
    fn default() -> Self {
        Self {
            model: String::new(),
            tokens: String::new(),
            model_eb: None,  // 新增：默认没有 model_eb
            hotwords_file: None,  // 新增：默认没有热词文件
            hotwords_score: 0.0,  // 新增：默认热词分数
            debug: false,
            provider: None,
            num_threads: Some(1),
        }
    }
}

impl ParaformerRecognizer {
    pub fn new(config: ParaformerConfig) -> Result<Self> {
        let debug = config.debug.into();
        let provider = config.provider.unwrap_or(get_default_provider());

        // Prepare C strings - 需要保存生命周期
        let provider_cstr = cstring_from_str(&provider);
        let model_cstr = cstring_from_str(&config.model);
        let tokens_cstr = cstring_from_str(&config.tokens);
        let decoding_method_cstr = cstring_from_str("greedy_search");

        // 创建 model_eb CString（如果提供）
        let model_eb_cstr = config.model_eb.as_ref()
            .map(|s| CString::new(s.clone()))
            .transpose()?;

        // Paraformer model config - 设置 model 和 model_eb
        let mut paraformer_config = unsafe { std::mem::zeroed::<sherpa_rs_sys::SherpaOnnxOfflineParaformerModelConfig>() };
        paraformer_config.model = model_cstr.as_ptr();
        
        // 设置 model_eb（如果提供）
        if let Some(ref model_eb_cstr) = model_eb_cstr {
            paraformer_config.model_eb = model_eb_cstr.as_ptr();
        } else {
            paraformer_config.model_eb = std::ptr::null();
        }

        // Offline model config
        let model_config = unsafe {
            sherpa_rs_sys::SherpaOnnxOfflineModelConfig {
                debug,
                num_threads: config.num_threads.unwrap_or(1),
                provider: provider_cstr.as_ptr(),
                tokens: tokens_cstr.as_ptr(),
                paraformer: paraformer_config,

                // Null other model types
                bpe_vocab: mem::zeroed::<_>(),
                model_type: mem::zeroed::<_>(),
                modeling_unit: mem::zeroed::<_>(),
                nemo_ctc: mem::zeroed::<_>(),
                tdnn: mem::zeroed::<_>(),
                telespeech_ctc: null(),
                fire_red_asr: mem::zeroed::<_>(),
                transducer: mem::zeroed::<_>(),
                whisper: mem::zeroed::<_>(),
                sense_voice: mem::zeroed::<_>(),
                moonshine: mem::zeroed::<_>(),
                dolphin: mem::zeroed::<_>(),
                zipformer_ctc: mem::zeroed::<_>(),
                canary: mem::zeroed::<_>(),
                funasr_nano: mem::zeroed::<_>(),
                medasr: mem::zeroed::<_>(),
                omnilingual: mem::zeroed::<_>(),
                wenet_ctc: mem::zeroed::<_>(),
            }
        };


        // 创建 hotwords_file CString（如果提供）
        let hotwords_file_cstr = config.hotwords_file.as_ref()
            .map(|s| cstring_from_str(s));

        // Recognizer config
        let recognizer_config = unsafe {
            sherpa_rs_sys::SherpaOnnxOfflineRecognizerConfig {
                decoding_method: decoding_method_cstr.as_ptr(),
                feat_config: sherpa_rs_sys::SherpaOnnxFeatureConfig {
                    sample_rate: 16000,
                    feature_dim: 80,
                },
                model_config,
                hotwords_file: hotwords_file_cstr.as_ref()
                    .map(|s| s.as_ptr())
                    .unwrap_or(null()),
                hotwords_score: config.hotwords_score,
                lm_config: mem::zeroed::<_>(),
                max_active_paths: 0,
                rule_fars: null(),
                rule_fsts: null(),
                blank_penalty: 0.0,
                hr: mem::zeroed::<_>(),
            }
        };

        let recognizer =
            unsafe { sherpa_rs_sys::SherpaOnnxCreateOfflineRecognizer(&recognizer_config) };
        if recognizer.is_null() {
            bail!("Failed to create Paraformer recognizer");
        }

        // 保存所有 CString 以确保生命周期
        // 转换 const 指针为 mut 指针（实际上在运行时是可变的）
        Ok(Self {
            recognizer: recognizer as *mut _,
            _model_cstr: model_cstr,
            _tokens_cstr: tokens_cstr,
            _model_eb_cstr: model_eb_cstr,
            _hotwords_file_cstr: hotwords_file_cstr,
        })
    }

    pub fn transcribe(&mut self, sample_rate: u32, samples: &[f32]) -> ParaformerRecognizerResult {
        unsafe {
            let stream = sherpa_rs_sys::SherpaOnnxCreateOfflineStream(self.recognizer);
            sherpa_rs_sys::SherpaOnnxAcceptWaveformOffline(
                stream,
                sample_rate as i32,
                samples.as_ptr(),
                samples.len() as i32,
            );
            sherpa_rs_sys::SherpaOnnxDecodeOfflineStream(self.recognizer, stream);
            let result_ptr = sherpa_rs_sys::SherpaOnnxGetOfflineStreamResult(stream);
            let raw_result = result_ptr.read();
            let result = ParaformerRecognizerResult::new(&raw_result);

            sherpa_rs_sys::SherpaOnnxDestroyOfflineRecognizerResult(result_ptr);
            sherpa_rs_sys::SherpaOnnxDestroyOfflineStream(stream);

            result
        }
    }
}

unsafe impl Send for ParaformerRecognizer {}
unsafe impl Sync for ParaformerRecognizer {}

impl Drop for ParaformerRecognizer {
    fn drop(&mut self) {
        unsafe {
            sherpa_rs_sys::SherpaOnnxDestroyOfflineRecognizer(self.recognizer);
        }
    }
}
