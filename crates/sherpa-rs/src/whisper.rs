use crate::{get_default_provider, utils::cstring_from_str};
use eyre::{bail, Result};
use std::mem;

#[derive(Debug)]
pub struct WhisperRecognizer {
    recognizer: *const sherpa_rs_sys::SherpaOnnxOfflineRecognizer,
}

pub type WhisperRecognizerResult = super::OfflineRecognizerResult;

#[derive(Debug, Clone)]
pub struct WhisperConfig {
    pub decoder: String,
    pub encoder: String,
    pub tokens: String,
    pub language: String,
    pub bpe_vocab: Option<String>,
    pub tail_paddings: Option<i32>,

    pub provider: Option<String>,
    pub num_threads: Option<i32>,
    pub debug: bool,
}

impl Default for WhisperConfig {
    fn default() -> Self {
        Self {
            decoder: String::new(),
            encoder: String::new(),
            tokens: String::new(),
            language: String::from("en"),
            bpe_vocab: None,
            tail_paddings: None,
            debug: false,
            provider: None,
            num_threads: Some(1),
        }
    }
}

impl WhisperRecognizer {
    pub fn new(config: WhisperConfig) -> Result<Self> {
        let debug = config.debug.into();
        let provider = config.provider.unwrap_or(get_default_provider());

        let provider_ptr = cstring_from_str(&provider);
        let num_threads = config.num_threads.unwrap_or(2);
        let bpe_vocab_ptr = cstring_from_str(&config.bpe_vocab.unwrap_or("".into()));
        let tail_paddings = config.tail_paddings.unwrap_or(0);
        let decoder_ptr = cstring_from_str(&config.decoder);
        let encoder_ptr = cstring_from_str(&config.encoder);
        let language_ptr = cstring_from_str(&config.language);
        let task_ptr = cstring_from_str("transcribe");
        let tokens_ptr = cstring_from_str(&config.tokens);
        let decoding_method_ptr = cstring_from_str("greedy_search");
        let empty = cstring_from_str("");

        let mut model_config = crate::safe_default_offline_model_config(empty.as_ptr());
        model_config.whisper = sherpa_rs_sys::SherpaOnnxOfflineWhisperModelConfig {
            decoder: decoder_ptr.as_ptr(),
            encoder: encoder_ptr.as_ptr(),
            language: language_ptr.as_ptr(),
            task: task_ptr.as_ptr(),
            tail_paddings,
        };
        model_config.debug = debug;
        model_config.num_threads = num_threads;
        model_config.provider = provider_ptr.as_ptr();
        model_config.bpe_vocab = bpe_vocab_ptr.as_ptr();
        model_config.tokens = tokens_ptr.as_ptr();

        let mut recognizer_config =
            crate::safe_default_offline_recognizer_config(model_config, empty.as_ptr());
        recognizer_config.decoding_method = decoding_method_ptr.as_ptr();
        recognizer_config.feat_config.feature_dim = 512;

        let recognizer =
            unsafe { sherpa_rs_sys::SherpaOnnxCreateOfflineRecognizer(&recognizer_config) };

        if recognizer.is_null() {
            bail!("Failed to create recognizer");
        }

        Ok(Self { recognizer })
    }

    pub fn transcribe(&mut self, sample_rate: u32, samples: &[f32]) -> WhisperRecognizerResult {
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
            let result = WhisperRecognizerResult::new(&raw_result);
            // Free
            sherpa_rs_sys::SherpaOnnxDestroyOfflineRecognizerResult(result_ptr);
            sherpa_rs_sys::SherpaOnnxDestroyOfflineStream(stream);
            result
        }
    }
}

unsafe impl Send for WhisperRecognizer {}
unsafe impl Sync for WhisperRecognizer {}

impl Drop for WhisperRecognizer {
    fn drop(&mut self) {
        unsafe {
            sherpa_rs_sys::SherpaOnnxDestroyOfflineRecognizer(self.recognizer);
        }
    }
}
