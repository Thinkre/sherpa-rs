use crate::utils::cstr_to_string;
use crate::{get_default_provider, utils::cstring_from_str};
use eyre::{bail, Result};
use std::mem;

pub struct TransducerRecognizer {
    recognizer: *const sherpa_rs_sys::SherpaOnnxOfflineRecognizer,
}

#[derive(Debug, Clone)]
pub struct TransducerConfig {
    pub decoder: String,
    pub encoder: String,
    pub joiner: String,
    pub tokens: String,
    pub num_threads: i32,
    pub sample_rate: i32,
    pub feature_dim: i32,
    pub decoding_method: String,
    pub hotwords_file: String,
    pub hotwords_score: f32,
    pub modeling_unit: String,
    pub bpe_vocab: String,
    pub blank_penalty: f32,
    pub model_type: String,
    pub debug: bool,
    pub provider: Option<String>,
}

impl Default for TransducerConfig {
    fn default() -> Self {
        TransducerConfig {
            decoder: String::new(),
            encoder: String::new(),
            joiner: String::new(),
            tokens: String::new(),
            model_type: String::from("transducer"),
            num_threads: 1,
            sample_rate: 0,
            feature_dim: 0,
            decoding_method: String::new(),
            hotwords_file: String::new(),
            hotwords_score: 0.0,
            modeling_unit: String::new(),
            bpe_vocab: String::new(),
            blank_penalty: 0.0,
            debug: false,
            provider: None,
        }
    }
}

impl TransducerRecognizer {
    pub fn new(config: TransducerConfig) -> Result<Self> {
        let recognizer = unsafe {
            let debug = config.debug.into();
            let provider = config.provider.unwrap_or(get_default_provider());
            let provider_ptr = cstring_from_str(&provider);

            let encoder = cstring_from_str(&config.encoder);
            let decoder = cstring_from_str(&config.decoder);
            let joiner = cstring_from_str(&config.joiner);
            let model_type = cstring_from_str(&config.model_type);
            let modeling_unit = cstring_from_str(&config.modeling_unit);
            let bpe_vocab = cstring_from_str(&config.bpe_vocab);
            let hotwords_file = cstring_from_str(&config.hotwords_file);
            let tokens = cstring_from_str(&config.tokens);
            let decoding_method = cstring_from_str(&config.decoding_method);
            let empty = cstring_from_str("");

            let mut offline_model_config = crate::safe_default_offline_model_config(empty.as_ptr());
            offline_model_config.transducer =
                sherpa_rs_sys::SherpaOnnxOfflineTransducerModelConfig {
                    encoder: encoder.as_ptr(),
                    decoder: decoder.as_ptr(),
                    joiner: joiner.as_ptr(),
                };
            offline_model_config.tokens = tokens.as_ptr();
            offline_model_config.num_threads = config.num_threads;
            offline_model_config.debug = debug;
            offline_model_config.provider = provider_ptr.as_ptr();
            offline_model_config.model_type = model_type.as_ptr();
            offline_model_config.modeling_unit = modeling_unit.as_ptr();
            offline_model_config.bpe_vocab = bpe_vocab.as_ptr();

            let mut recognizer_config =
                crate::safe_default_offline_recognizer_config(offline_model_config, empty.as_ptr());
            recognizer_config.feat_config.sample_rate = config.sample_rate;
            recognizer_config.feat_config.feature_dim = config.feature_dim;
            recognizer_config.hotwords_file = hotwords_file.as_ptr();
            recognizer_config.blank_penalty = config.blank_penalty;
            recognizer_config.decoding_method = decoding_method.as_ptr();
            recognizer_config.hotwords_score = config.hotwords_score;

            let recognizer = sherpa_rs_sys::SherpaOnnxCreateOfflineRecognizer(&recognizer_config);
            if recognizer.is_null() {
                bail!("SherpaOnnxCreateOfflineRecognizer failed");
            }
            recognizer
        };

        Ok(Self { recognizer })
    }

    pub fn transcribe(&mut self, sample_rate: u32, samples: &[f32]) -> String {
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

unsafe impl Send for TransducerRecognizer {}
unsafe impl Sync for TransducerRecognizer {}

impl Drop for TransducerRecognizer {
    fn drop(&mut self) {
        unsafe {
            sherpa_rs_sys::SherpaOnnxDestroyOfflineRecognizer(self.recognizer);
        }
    }
}
