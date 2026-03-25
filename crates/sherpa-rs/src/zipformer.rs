use crate::{
    get_default_provider, safe_default_offline_model_config,
    utils::{cstr_to_string, cstring_from_str},
};
use eyre::{bail, Result};
use std::mem;

#[derive(Debug, Default)]
pub struct ZipFormerConfig {
    pub decoder: String,
    pub encoder: String,
    pub joiner: String,
    pub tokens: String,

    pub num_threads: Option<i32>,
    pub provider: Option<String>,
    pub debug: bool,
}

pub struct ZipFormer {
    recognizer: *const sherpa_rs_sys::SherpaOnnxOfflineRecognizer,
}

impl ZipFormer {
    pub fn new(config: ZipFormerConfig) -> Result<Self> {
        // Zipformer config
        let decoder_ptr = cstring_from_str(&config.decoder);
        let encoder_ptr = cstring_from_str(&config.encoder);
        let joiner_ptr = cstring_from_str(&config.joiner);
        let provider_ptr = cstring_from_str(&config.provider.unwrap_or(get_default_provider()));
        let tokens_ptr = cstring_from_str(&config.tokens);
        let decoding_method_ptr = cstring_from_str("greedy_search");

        let empty_ptr = cstring_from_str("");
        let mut model_config = unsafe { safe_default_offline_model_config(empty_ptr.as_ptr()) };

        // Set transducer configuration
        model_config.transducer = sherpa_rs_sys::SherpaOnnxOfflineTransducerModelConfig {
            decoder: decoder_ptr.as_ptr(),
            encoder: encoder_ptr.as_ptr(),
            joiner: joiner_ptr.as_ptr(),
        };

        // Set general configurations
        model_config.num_threads = config.num_threads.unwrap_or(1);
        model_config.debug = config.debug.into();
        model_config.provider = provider_ptr.as_ptr();
        model_config.tokens = tokens_ptr.as_ptr();

        // Recognizer config
        let recognizer_config = unsafe {
            sherpa_rs_sys::SherpaOnnxOfflineRecognizerConfig {
                model_config,
                decoding_method: decoding_method_ptr.as_ptr(),
                // Properly initialize feature config
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

        let recognizer =
            unsafe { sherpa_rs_sys::SherpaOnnxCreateOfflineRecognizer(&recognizer_config) };

        if recognizer.is_null() {
            bail!("Failed to create recognizer");
        }
        Ok(Self { recognizer })
    }

    pub fn decode(&mut self, sample_rate: u32, samples: Vec<f32>) -> String {
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
            let text = cstr_to_string(raw_result.text as _);

            // Free
            sherpa_rs_sys::SherpaOnnxDestroyOfflineRecognizerResult(result_ptr);
            sherpa_rs_sys::SherpaOnnxDestroyOfflineStream(stream);
            text
        }
    }
}

unsafe impl Send for ZipFormer {}
unsafe impl Sync for ZipFormer {}

impl Drop for ZipFormer {
    fn drop(&mut self) {
        unsafe {
            sherpa_rs_sys::SherpaOnnxDestroyOfflineRecognizer(self.recognizer);
        }
    }
}
