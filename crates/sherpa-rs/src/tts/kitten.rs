use std::ptr::null;

use crate::{utils::cstring_from_str, OnnxConfig};
use eyre::Result;
use sherpa_rs_sys;

use super::{CommonTtsConfig, TtsAudio};

pub struct KittenTts {
    tts: *const sherpa_rs_sys::SherpaOnnxOfflineTts,
}

#[derive(Default)]
pub struct KittenTtsConfig {
    pub model: String,
    pub voices: String,
    pub tokens: String,
    pub data_dir: String,
    pub length_scale: f32,
    pub onnx_config: OnnxConfig,
    pub common_config: CommonTtsConfig,
}

impl KittenTts {
    pub fn new(config: KittenTtsConfig) -> Self {
        let tts = unsafe {
            let model = cstring_from_str(&config.model);
            let voices = cstring_from_str(&config.voices);
            let tokens = cstring_from_str(&config.tokens);
            let data_dir = cstring_from_str(&config.data_dir);

            let provider = cstring_from_str(&config.onnx_config.provider);

            let tts_config = config.common_config.to_raw();

            // 使用默认函数来初始化结构体避免遗漏字段
            let empty = cstring_from_str("");
            let mut model_config = sherpa_rs_sys::SherpaOnnxOfflineTtsModelConfig {
                vits: sherpa_rs_sys::SherpaOnnxOfflineTtsVitsModelConfig {
                    model: empty.as_ptr(),
                    lexicon: empty.as_ptr(),
                    tokens: empty.as_ptr(),
                    data_dir: empty.as_ptr(),
                    noise_scale: 0.0,
                    noise_scale_w: 0.0,
                    length_scale: 0.0,
                    speakers: empty.as_ptr(),
                    speaker_id: 0,
                    lang: empty.as_ptr(),
                    rule_fsts: empty.as_ptr(),
                    rule_fars: empty.as_ptr(),
                    acoustic_model:
                        sherpa_rs_sys::SherpaOnnxOfflineTtsVitsModelAcousticModelConfig {
                            model: empty.as_ptr(),
                            tokens: empty.as_ptr(),
                            lexicon: empty.as_ptr(),
                            data_dir: empty.as_ptr(),
                        },
                },
                num_threads: config.onnx_config.num_threads,
                debug: config.onnx_config.debug.into(),
                provider: provider.as_ptr(),
                matcha: sherpa_rs_sys::SherpaOnnxOfflineTtsMatchaModelConfig {
                    model: empty.as_ptr(),
                    tokens: empty.as_ptr(),
                    lexicon: empty.as_ptr(),
                    data_dir: empty.as_ptr(),
                    lang: empty.as_ptr(),
                    speakers: empty.as_ptr(),
                },
                kokoro: sherpa_rs_sys::SherpaOnnxOfflineTtsKokoroModelConfig {
                    model: empty.as_ptr(),
                    tokens: empty.as_ptr(),
                    lexicon: empty.as_ptr(),
                    data_dir: empty.as_ptr(),
                    speakers: empty.as_ptr(),
                },
                kitten: sherpa_rs_sys::SherpaOnnxOfflineTtsKittenModelConfig {
                    model: model.as_ptr(),
                    voices: voices.as_ptr(),
                    tokens: tokens.as_ptr(),
                    data_dir: data_dir.as_ptr(),
                    length_scale: config.length_scale,
                },
                zipvoice: sherpa_rs_sys::SherpaOnnxOfflineTtsZipVoiceModelConfig {
                    model: empty.as_ptr(),
                    tokens: empty.as_ptr(),
                    lexicon: empty.as_ptr(),
                    data_dir: empty.as_ptr(),
                    speakers: empty.as_ptr(),
                },
                pocket: sherpa_rs_sys::SherpaOnnxOfflineTtsPocketModelConfig {
                    model: empty.as_ptr(),
                    tokens: empty.as_ptr(),
                    language: empty.as_ptr(),
                    normalize_g2p: 0,
                    normalize_numbers: 0,
                    normalize_romanization: 0,
                    normalize_spellout: 0,
                    normalize_date_time: 0,
                    normalize_measure: 0,
                    normalize_ordinal: 0,
                    normalize_fraction: 0,
                },
                supertonic: sherpa_rs_sys::SherpaOnnxOfflineTtsSupertonicModelConfig {
                    acoustic_model: empty.as_ptr(),
                    flow_model: empty.as_ptr(),
                    vocoder: empty.as_ptr(),
                    data_dir: empty.as_ptr(),
                    language: empty.as_ptr(),
                    lexicon: empty.as_ptr(),
                    onnx_acoustic_model: empty.as_ptr(),
                    onnx_flow_model: empty.as_ptr(),
                    onnx_vocoder: empty.as_ptr(),
                    onnx_pre_aligner: empty.as_ptr(),
                    onnx_text_encoder: empty.as_ptr(),
                },
            };

            let config = sherpa_rs_sys::SherpaOnnxOfflineTtsConfig {
                max_num_sentences: config.common_config.max_num_sentences,
                model: model_config,
                rule_fars: tts_config.rule_fars.map(|v| v.as_ptr()).unwrap_or(null()),
                rule_fsts: tts_config.rule_fsts.map(|v| v.as_ptr()).unwrap_or(null()),
                silence_scale: 1.0,
            };
            sherpa_rs_sys::SherpaOnnxCreateOfflineTts(&config)
        };

        Self { tts }
    }

    pub fn create(&mut self, text: &str, sid: i32, speed: f32) -> Result<TtsAudio> {
        unsafe { super::create(self.tts, text, sid, speed) }
    }
}

unsafe impl Send for KittenTts {}
unsafe impl Sync for KittenTts {}

impl Drop for KittenTts {
    fn drop(&mut self) {
        unsafe {
            sherpa_rs_sys::SherpaOnnxDestroyOfflineTts(self.tts);
        }
    }
}
