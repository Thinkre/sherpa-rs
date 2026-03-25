use std::ptr::null;

use crate::{utils::cstring_from_str, OnnxConfig};
use eyre::Result;
use sherpa_rs_sys;

use super::{CommonTtsConfig, TtsAudio};

pub struct VitsTts {
    tts: *const sherpa_rs_sys::SherpaOnnxOfflineTts,
}

#[derive(Default)]
pub struct VitsTtsConfig {
    pub model: String,
    pub lexicon: String,
    pub dict_dir: String,
    pub tokens: String,
    pub data_dir: String,
    pub length_scale: f32,
    pub noise_scale: f32,
    pub noise_scale_w: f32,
    pub silence_scale: f32,

    pub onnx_config: OnnxConfig,
    pub tts_config: CommonTtsConfig,
}

impl VitsTts {
    pub fn new(config: VitsTtsConfig) -> Self {
        let tts = unsafe {
            let model = cstring_from_str(&config.model);
            let tokens = cstring_from_str(&config.tokens);
            let data_dir = cstring_from_str(&config.data_dir);
            let lexicon = cstring_from_str(&config.lexicon);
            let dict_dir = cstring_from_str(&config.dict_dir);

            let provider = cstring_from_str(&config.onnx_config.provider);

            let tts_config = config.tts_config.to_raw();

            let empty = cstring_from_str("");
            let mut model_config = sherpa_rs_sys::SherpaOnnxOfflineTtsModelConfig {
                vits: sherpa_rs_sys::SherpaOnnxOfflineTtsVitsModelConfig {
                    model: model.as_ptr(),
                    lexicon: lexicon.as_ptr(),
                    tokens: tokens.as_ptr(),
                    data_dir: data_dir.as_ptr(),
                    noise_scale: config.noise_scale,
                    noise_scale_w: config.noise_scale_w,
                    length_scale: config.length_scale,
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
                    voices: empty.as_ptr(),
                    tokens: empty.as_ptr(),
                    data_dir: empty.as_ptr(),
                    length_scale: 0.0,
                    dict_dir: empty.as_ptr(),
                    lexicon: empty.as_ptr(),
                    lang: empty.as_ptr(),
                },
                kitten: sherpa_rs_sys::SherpaOnnxOfflineTtsKittenModelConfig {
                    model: empty.as_ptr(),
                    voices: empty.as_ptr(),
                    tokens: empty.as_ptr(),
                    data_dir: empty.as_ptr(),
                    length_scale: 0.0,
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
                max_num_sentences: config.tts_config.max_num_sentences,
                model: model_config,
                rule_fasts: tts_config.rule_fars.map(|v| v.as_ptr()).unwrap_or(null()),
                rule_fsts: tts_config.rule_fsts.map(|v| v.as_ptr()).unwrap_or(null()),
                silence_scale: config.silence_scale,
            };
            sherpa_rs_sys::SherpaOnnxCreateOfflineTts(&config)
        };

        Self { tts }
    }

    pub fn create(&mut self, text: &str, sid: i32, speed: f32) -> Result<TtsAudio> {
        unsafe { super::create(self.tts, text, sid, speed) }
    }
}

unsafe impl Send for VitsTts {}
unsafe impl Sync for VitsTts {}

impl Drop for VitsTts {
    fn drop(&mut self) {
        unsafe {
            sherpa_rs_sys::SherpaOnnxDestroyOfflineTts(self.tts);
        }
    }
}
