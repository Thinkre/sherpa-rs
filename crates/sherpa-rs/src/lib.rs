pub mod audio_tag;
pub mod diarize;
pub mod dolphin;
pub mod embedding_manager;
pub mod keyword_spot;
pub mod language_id;
pub mod moonshine;
pub mod paraformer;
pub mod punctuate;
pub mod sense_voice;
pub mod silero_vad;
pub mod speaker_id;
pub mod ten_vad;
pub mod transducer;
pub mod whisper;
pub mod zipformer;

mod utils;

#[cfg(feature = "tts")]
pub mod tts;

use std::ffi::CStr;

#[cfg(feature = "sys")]
pub use sherpa_rs_sys;

use eyre::{bail, Result};
use utils::cstr_to_string;

/// Create a safe default SherpaOnnxOfflineModelConfig with all char* fields set to empty strings.
/// sherpa-onnx calls strlen() on all char* fields unconditionally, so null pointers cause SIGSEGV.
pub(crate) fn safe_default_offline_model_config(empty: *const std::os::raw::c_char) -> sherpa_rs_sys::SherpaOnnxOfflineModelConfig {
    unsafe {
        sherpa_rs_sys::SherpaOnnxOfflineModelConfig {
            debug: 0,
            num_threads: 1,
            provider: empty,
            tokens: empty,
            model_type: empty,
            modeling_unit: empty,
            bpe_vocab: empty,
            telespeech_ctc: empty,
            paraformer: sherpa_rs_sys::SherpaOnnxOfflineParaformerModelConfig {
                model: empty,
                model_eb: empty,
            },
            transducer: sherpa_rs_sys::SherpaOnnxOfflineTransducerModelConfig {
                encoder: empty,
                decoder: empty,
                joiner: empty,
            },
            whisper: sherpa_rs_sys::SherpaOnnxOfflineWhisperModelConfig {
                encoder: empty,
                decoder: empty,
                language: empty,
                task: empty,
                tail_paddings: 0,
            },
            tdnn: sherpa_rs_sys::SherpaOnnxOfflineTdnnModelConfig { model: empty },
            nemo_ctc: sherpa_rs_sys::SherpaOnnxOfflineNemoEncDecCtcModelConfig { model: empty },
            fire_red_asr: sherpa_rs_sys::SherpaOnnxOfflineFireRedAsrModelConfig { encoder: empty, decoder: empty },
            sense_voice: sherpa_rs_sys::SherpaOnnxOfflineSenseVoiceModelConfig { model: empty, language: empty, use_itn: 0 },
            moonshine: sherpa_rs_sys::SherpaOnnxOfflineMoonshineModelConfig { preprocessor: empty, encoder: empty, uncached_decoder: empty, cached_decoder: empty },
            dolphin: sherpa_rs_sys::SherpaOnnxOfflineDolphinModelConfig { model: empty },
            zipformer_ctc: sherpa_rs_sys::SherpaOnnxOfflineZipformerCtcModelConfig { model: empty },
            canary: sherpa_rs_sys::SherpaOnnxOfflineCanaryModelConfig { encoder: empty, decoder: empty, src_lang: empty, tgt_lang: empty, use_pnc: 0 },
            funasr_nano: sherpa_rs_sys::SherpaOnnxOfflineFunASRNanoModelConfig { encoder_adaptor: empty, llm: empty, embedding: empty, tokenizer: empty, system_prompt: empty, user_prompt: empty, max_new_tokens: 0, temperature: 0.0, top_p: 0.0, seed: 0 },
            medasr: sherpa_rs_sys::SherpaOnnxOfflineMedAsrCtcModelConfig { model: empty },
            omnilingual: sherpa_rs_sys::SherpaOnnxOfflineOmnilingualAsrCtcModelConfig { model: empty },
            wenet_ctc: sherpa_rs_sys::SherpaOnnxOfflineWenetCtcModelConfig { model: empty },
        }
    }
}

/// Create a safe default SherpaOnnxOfflineRecognizerConfig with all char* fields set to empty strings.
pub(crate) fn safe_default_offline_recognizer_config(
    model_config: sherpa_rs_sys::SherpaOnnxOfflineModelConfig,
    empty: *const std::os::raw::c_char,
) -> sherpa_rs_sys::SherpaOnnxOfflineRecognizerConfig {
    sherpa_rs_sys::SherpaOnnxOfflineRecognizerConfig {
        feat_config: sherpa_rs_sys::SherpaOnnxFeatureConfig { sample_rate: 16000, feature_dim: 80 },
        model_config,
        lm_config: sherpa_rs_sys::SherpaOnnxOfflineLMConfig { model: empty, scale: 1.0 },
        decoding_method: empty,
        max_active_paths: 0,
        hotwords_file: empty,
        hotwords_score: 0.0,
        rule_fsts: empty,
        rule_fars: empty,
        blank_penalty: 0.0,
        hr: sherpa_rs_sys::SherpaOnnxHomophoneReplacerConfig { dict_dir: empty, lexicon: empty, rule_fsts: empty },
    }
}

pub fn get_default_provider() -> String {
    "cpu".into()
    // Other providers has many issues with different models!!
    // if cfg!(feature = "cuda") {
    //     "cuda"
    // } else if cfg!(target_os = "macos") {
    //     "coreml"
    // } else if cfg!(feature = "directml") {
    //     "directml"
    // } else {
    //     "cpu"
    // }
    // .into()
}

pub fn read_audio_file(path: &str) -> Result<(Vec<f32>, u32)> {
    let mut reader = hound::WavReader::open(path)?;
    let sample_rate = reader.spec().sample_rate;

    // Check if the sample rate is 16000
    if sample_rate != 16000 {
        bail!("The sample rate must be 16000.");
    }

    // Collect samples into a Vec<f32>
    let samples: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| (s.unwrap() as f32) / (i16::MAX as f32))
        .collect();

    Ok((samples, sample_rate))
}

pub fn write_audio_file(path: &str, samples: &[f32], sample_rate: u32) -> Result<()> {
    // Create a WAV file writer
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(path, spec)?;

    // Convert samples from f32 to i16 and write them to the WAV file
    for &sample in samples {
        let scaled_sample =
            (sample * (i16::MAX as f32)).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        writer.write_sample(scaled_sample)?;
    }

    writer.finalize()?;
    Ok(())
}

pub struct OnnxConfig {
    pub provider: String,
    pub debug: bool,
    pub num_threads: i32,
}

#[derive(Debug, Clone)]
pub struct OfflineRecognizerResult {
    pub lang: String,
    pub text: String,
    pub timestamps: Vec<f32>,
    pub tokens: Vec<String>,
}

impl OfflineRecognizerResult {
    fn new(result: &sherpa_rs_sys::SherpaOnnxOfflineRecognizerResult) -> Self {
        let lang = unsafe { cstr_to_string(result.lang) };
        let text = unsafe { cstr_to_string(result.text) };
        let count = result.count.try_into().unwrap();
        let timestamps = if result.timestamps.is_null() {
            Vec::new()
        } else {
            unsafe { std::slice::from_raw_parts(result.timestamps, count).to_vec() }
        };
        let mut tokens = Vec::with_capacity(count);
        let mut next_token = result.tokens;

        for _ in 0..count {
            let token = unsafe { CStr::from_ptr(next_token) };
            tokens.push(token.to_string_lossy().into_owned());
            next_token = next_token
                .wrapping_byte_offset(token.to_bytes_with_nul().len().try_into().unwrap());
        }

        Self {
            lang,
            text,
            timestamps,
            tokens,
        }
    }
}

impl Default for OnnxConfig {
    fn default() -> Self {
        Self {
            provider: get_default_provider(),
            debug: false,
            num_threads: 1,
        }
    }
}
