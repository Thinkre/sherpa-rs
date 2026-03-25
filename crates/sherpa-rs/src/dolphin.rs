use crate::{get_default_provider, safe_default_offline_model_config, utils::cstring_from_str};
use eyre::{bail, Result};
use std::mem;

#[derive(Debug)]
pub struct DolphinRecognizer {
    recognizer: *const sherpa_rs_sys::SherpaOnnxOfflineRecognizer,
}

pub type DolphinRecognizerResult = super::OfflineRecognizerResult;

#[derive(Debug, Clone)]
pub struct DolphinConfig {
    pub model: String,
    pub tokens: String,
    pub decoding_method: String,

    pub provider: Option<String>,
    pub num_threads: Option<i32>,
    pub debug: bool,
}

impl Default for DolphinConfig {
    fn default() -> Self {
        Self {
            model: String::new(),
            tokens: String::new(),
            decoding_method: String::from("greedy_search"),
            debug: false,
            provider: None,
            num_threads: Some(1),
        }
    }
}

impl DolphinRecognizer {
    pub fn new(config: DolphinConfig) -> Result<Self> {
        let debug = config.debug.into();
        let provider = config.provider.unwrap_or(get_default_provider());

        let provider_ptr = cstring_from_str(&provider);
        let num_threads = config.num_threads.unwrap_or(2);
        let model_ptr = cstring_from_str(&config.model);
        let tokens_ptr = cstring_from_str(&config.tokens);
        let decoding_method_ptr = cstring_from_str(&config.decoding_method);

        let empty_ptr = cstring_from_str("");
        let mut model_config = unsafe { safe_default_offline_model_config(empty_ptr.as_ptr()) };

        model_config.debug = debug;
        model_config.num_threads = num_threads;
        model_config.provider = provider_ptr.as_ptr();
        model_config.dolphin = sherpa_rs_sys::SherpaOnnxOfflineDolphinModelConfig {
            model: model_ptr.as_ptr(),
        };
        model_config.tokens = tokens_ptr.as_ptr();

        let config = unsafe {
            sherpa_rs_sys::SherpaOnnxOfflineRecognizerConfig {
                decoding_method: decoding_method_ptr.as_ptr(),
                model_config,
                feat_config: sherpa_rs_sys::SherpaOnnxFeatureConfig {
                    sample_rate: 16000,
                    feature_dim: 80,
                },
                hotwords_file: empty_ptr.as_ptr(),
                hotwords_score: 0.0,
                lm_config: sherpa_rs_sys::SherpaOnnxOfflineLMConfig {
                    model: empty_ptr.as_ptr(),
                    scale: 1.0,
                },
                max_active_paths: 4,
                rule_fars: empty_ptr.as_ptr(),
                rule_fsts: empty_ptr.as_ptr(),
                blank_penalty: 0.0,
                hr: sherpa_rs_sys::SherpaOnnxHomophoneReplacerConfig {
                    dict_dir: empty_ptr.as_ptr(),
                    lexicon: empty_ptr.as_ptr(),
                    rule_fsts: empty_ptr.as_ptr(),
                },
            }
        };

        let recognizer = unsafe { sherpa_rs_sys::SherpaOnnxCreateOfflineRecognizer(&config) };

        if recognizer.is_null() {
            bail!("Failed to create recognizer");
        }

        Ok(Self { recognizer })
    }

    pub fn transcribe(&mut self, sample_rate: u32, samples: &[f32]) -> DolphinRecognizerResult {
        unsafe {
            let stream = sherpa_rs_sys::SherpaOnnxCreateOfflineStream(self.recognizer);
            sherpa_rs_sys::SherpaOnnxAcceptWaveformOffline(
                stream,
                sample_rate as i32,
                samples.as_ptr(),
                samples.len().try_into().unwrap(),
            );
            sherpa_rs_sys::SherpaOnnxDecodeOfflineStream(self.recognizer, stream);
            let result_ptr = sherpa_rs_sys::SherpaOnnxGetOfflineStreamResult(stream);
            let raw_result = result_ptr.read();
            let result = DolphinRecognizerResult::new(&raw_result);
            // Free
            sherpa_rs_sys::SherpaOnnxDestroyOfflineRecognizerResult(result_ptr);
            sherpa_rs_sys::SherpaOnnxDestroyOfflineStream(stream);
            result
        }
    }
}

unsafe impl Send for DolphinRecognizer {}
unsafe impl Sync for DolphinRecognizer {}

impl Drop for DolphinRecognizer {
    fn drop(&mut self) {
        unsafe {
            sherpa_rs_sys::SherpaOnnxDestroyOfflineRecognizer(self.recognizer);
        }
    }
}
