use crate::audio_toolkit::{
    apply_custom_words, apply_hot_rules, filter_transcription_output, load_hot_rules,
};
use crate::managers::model::{EngineType, ModelManager};
use crate::settings::{get_settings, ModelUnloadTimeout};
use crate::transcription_api_client;
use anyhow::Result;
use log::{debug, error, info, warn};
use serde::Serialize;
use sherpa_rs::paraformer::{ParaformerConfig, ParaformerRecognizer};
use sherpa_rs::transducer::{TransducerConfig, TransducerRecognizer};
use sherpa_rs_sys::*;
use std::ffi::CString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};
use tauri::{AppHandle, Emitter, Manager};
use transcribe_rs::{
    engines::whisper::{WhisperEngine, WhisperInferenceParams},
    TranscriptionEngine,
};

#[derive(Clone, Debug, Serialize)]
pub struct ModelStateEvent {
    pub event_type: String,
    pub model_id: Option<String>,
    pub model_name: Option<String>,
    pub error: Option<String>,
}


#[derive(Clone, Debug)]
pub struct TranscriptionResult {
    pub original_text: String,      // Original transcription (before ITN)
    pub itn_text: Option<String>,   // After ITN (if enabled)
    pub final_text: String,         // Final result (after hot rules)
    pub model_name: Option<String>, // Model name used (for API: actual model like "qwen3-asr-flash", for local: model_id)
    pub is_api: bool,               // Whether this transcription used API
}

enum LoadedEngine {
    Whisper(WhisperEngine),
    Paraformer(ParaformerRecognizer),  // 包括标准 Paraformer 和 SeACo Paraformer（都使用 ParaformerRecognizer）
    Transducer(TransducerEngine),
    FireRedAsr(FireRedAsrEngine), // FireRedAsr uses C API directly (no high-level API available)
}

// Wrapper for transducer engine (supports hotwords)
struct TransducerEngine {
    recognizer: TransducerRecognizer,
    hotwords_file: Option<std::path::PathBuf>, // Keep path for cleanup
}

// Wrapper for FireRedAsr engine (uses C API directly)
struct FireRedAsrEngine {
    recognizer: *const SherpaOnnxOfflineRecognizer,
    // Keep CStrings alive for the lifetime of the engine
    _encoder_path: CString,
    _decoder_path: CString,
    _tokens_path: CString,
    _decoding_method: CString,
    _provider: CString,
    _model_type: CString,
}

unsafe impl Send for FireRedAsrEngine {}
unsafe impl Sync for FireRedAsrEngine {}

impl Drop for FireRedAsrEngine {
    fn drop(&mut self) {
        unsafe {
            if !self.recognizer.is_null() {
                SherpaOnnxDestroyOfflineRecognizer(self.recognizer);
            }
        }
    }
}


unsafe impl Send for TransducerEngine {}
unsafe impl Sync for TransducerEngine {}

impl Drop for TransducerEngine {
    fn drop(&mut self) {
        // TransducerRecognizer will be dropped automatically, which cleans up the recognizer
        // Clean up hotwords file if it was created temporarily
        if let Some(ref hotwords_path) = self.hotwords_file {
            if hotwords_path.starts_with(std::env::temp_dir()) {
                let _ = std::fs::remove_file(hotwords_path);
            }
        }
    }
}

#[derive(Clone)]
pub struct TranscriptionManager {
    engine: Arc<Mutex<Option<LoadedEngine>>>,
    model_manager: Arc<ModelManager>,
    app_handle: AppHandle,
    current_model_id: Arc<Mutex<Option<String>>>,
    last_activity: Arc<AtomicU64>,
    shutdown_signal: Arc<AtomicBool>,
    watcher_handle: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
    is_loading: Arc<Mutex<bool>>,
    loading_condvar: Arc<Condvar>,
    cancellation_flag: Arc<AtomicBool>, // Flag to cancel ongoing transcription
}

impl TranscriptionManager {
    pub fn new(app_handle: &AppHandle, model_manager: Arc<ModelManager>) -> Result<Self> {
        let manager = Self {
            engine: Arc::new(Mutex::new(None)),
            model_manager,
            app_handle: app_handle.clone(),
            current_model_id: Arc::new(Mutex::new(None)),
            last_activity: Arc::new(AtomicU64::new(
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            )),
            shutdown_signal: Arc::new(AtomicBool::new(false)),
            watcher_handle: Arc::new(Mutex::new(None)),
            is_loading: Arc::new(Mutex::new(false)),
            loading_condvar: Arc::new(Condvar::new()),
            cancellation_flag: Arc::new(AtomicBool::new(false)),
        };

        // Start the idle watcher
        {
            let app_handle_cloned = app_handle.clone();
            let manager_cloned = manager.clone();
            let shutdown_signal = manager.shutdown_signal.clone();
            let handle = thread::spawn(move || {
                while !shutdown_signal.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_secs(10)); // Check every 10 seconds

                    // Check shutdown signal again after sleep
                    if shutdown_signal.load(Ordering::Relaxed) {
                        break;
                    }

                    let settings = get_settings(&app_handle_cloned);
                    let timeout_seconds = settings.model_unload_timeout.to_seconds();

                    if let Some(limit_seconds) = timeout_seconds {
                        // Skip polling-based unloading for immediate timeout since it's handled directly in transcribe()
                        if settings.model_unload_timeout == ModelUnloadTimeout::Immediately {
                            continue;
                        }

                        let last = manager_cloned.last_activity.load(Ordering::Relaxed);
                        let now_ms = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64;

                        if now_ms.saturating_sub(last) > limit_seconds * 1000 {
                            // idle -> unload
                            if manager_cloned.is_model_loaded() {
                                let unload_start = std::time::Instant::now();
                                debug!("Starting to unload model due to inactivity");

                                if let Ok(()) = manager_cloned.unload_model() {
                                    let _ = app_handle_cloned.emit(
                                        "model-state-changed",
                                        ModelStateEvent {
                                            event_type: "unloaded".to_string(),
                                            model_id: None,
                                            model_name: None,
                                            error: None,
                                        },
                                    );
                                    let unload_duration = unload_start.elapsed();
                                    debug!(
                                        "Model unloaded due to inactivity (took {}ms)",
                                        unload_duration.as_millis()
                                    );
                                }
                            }
                        }
                    }
                }
                debug!("Idle watcher thread shutting down gracefully");
            });
            *manager.watcher_handle.lock().unwrap() = Some(handle);
        }

        Ok(manager)
    }

    /// Export bpe.vocab from bbpe.model using sentencepiece
    /// This is needed for hotwords support in transducer models
    /// Returns Ok(()) if successful, Err if failed (but model can still work without hotwords)
    ///
    /// Tries multiple methods in order:
    /// 1. Use sherpa-onnx's export_bpe_vocab.py script if available
    /// 2. Use embedded Python script with sentencepiece
    fn export_bpe_vocab(bbpe_model_path: &Path, bpe_vocab_path: &Path) -> Result<()> {
        use std::process::Command;

        // Check if bpe.vocab already exists
        if bpe_vocab_path.exists() {
            return Ok(());
        }

        // Method 1: Try to use sherpa-onnx's export_bpe_vocab.py script
        // The script expects: python scripts/bpe/export_bpe_vocab.py --bpe-model <bbpe.model>
        // It outputs bpe.vocab in the same directory as bbpe.model
        // Script can be downloaded from: https://github.com/k2-fsa/sherpa-onnx/blob/master/scripts/bpe/export_bpe_vocab.py
        let model_dir = bbpe_model_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid bbpe.model path: no parent directory"))?;

        // Try common locations for sherpa-onnx script (relative to current working directory or model directory)
        let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let home_dir = std::env::var("HOME").ok().map(std::path::PathBuf::from);

        let mut possible_script_paths = vec![
            // Relative to current working directory (if user cloned sherpa-onnx repo)
            current_dir.join("scripts/bpe/export_bpe_vocab.py"),
            current_dir.join("scripts/export_bpe_vocab.py"),
            current_dir.join("export_bpe_vocab.py"),
            // Relative to model directory
            model_dir.join("../../scripts/bpe/export_bpe_vocab.py"),
            model_dir.join("../scripts/bpe/export_bpe_vocab.py"),
            model_dir.join("scripts/bpe/export_bpe_vocab.py"),
        ];

        // Add home directory paths if available
        if let Some(ref home) = home_dir {
            possible_script_paths.push(home.join("sherpa-onnx/scripts/bpe/export_bpe_vocab.py"));
            possible_script_paths.push(home.join("sherpa-onnx/scripts/export_bpe_vocab.py"));
        }

        // Absolute paths (check common installation locations)
        possible_script_paths.push(std::path::PathBuf::from(
            "/usr/local/share/sherpa-onnx/scripts/bpe/export_bpe_vocab.py",
        ));
        possible_script_paths.push(std::path::PathBuf::from(
            "/opt/sherpa-onnx/scripts/bpe/export_bpe_vocab.py",
        ));

        for script_path in &possible_script_paths {
            if script_path.exists() {
                info!("Found sherpa-onnx export script at: {:?}", script_path);

                // Run the script: python scripts/bpe/export_bpe_vocab.py --bpe-model <bbpe.model>
                let output = Command::new("python3")
                    .arg(script_path)
                    .arg("--bpe-model")
                    .arg(bbpe_model_path)
                    .output();

                match output {
                    Ok(result) if result.status.success() => {
                        // Script outputs to same directory as bbpe.model, check if it exists
                        let expected_output = model_dir.join("bpe.vocab");
                        if expected_output.exists() {
                            // Copy to target location if different
                            if expected_output != bpe_vocab_path {
                                fs::copy(&expected_output, bpe_vocab_path)?;
                            }
                            info!("Successfully exported bpe.vocab using sherpa-onnx script");
                            return Ok(());
                        } else {
                            debug!("Script ran successfully but bpe.vocab not found at expected location: {:?}", expected_output);
                        }
                    }
                    Ok(result) => {
                        let error_msg = String::from_utf8_lossy(&result.stderr);
                        let stdout_msg = String::from_utf8_lossy(&result.stdout);
                        debug!(
                            "sherpa-onnx script failed: stderr={}, stdout={}",
                            error_msg, stdout_msg
                        );
                    }
                    Err(e) => {
                        debug!("Failed to run sherpa-onnx script: {}", e);
                        // Continue to next method
                    }
                }
            }
        }

        // Method 2: Use embedded Python script with sentencepiece
        // This is the fallback method
        let script_content = r#"
import sentencepiece as spm
import sys

if len(sys.argv) != 3:
    print("Usage: python script.py <bbpe.model> <bpe.vocab>", file=sys.stderr)
    sys.exit(1)

bbpe_model = sys.argv[1]
bpe_vocab = sys.argv[2]

try:
    sp = spm.SentencePieceProcessor()
    sp.load(bbpe_model)

    with open(bpe_vocab, 'w', encoding='utf-8') as f:
        for i in range(sp.get_piece_size()):
            piece = sp.id_to_piece(i)
            score = sp.get_score(i)
            f.write(f"{piece}\t{score}\n")
except Exception as e:
    print(f"Error: {e}", file=sys.stderr)
    sys.exit(1)
"#;

        // Create temporary Python script with unique name to avoid conflicts
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_script = std::env::temp_dir().join(format!("export_bpe_vocab_{}.py", timestamp));

        // Write script and handle errors
        if let Err(e) = fs::write(&temp_script, script_content) {
            return Err(anyhow::anyhow!("Failed to create temporary script: {}", e));
        }

        // Try to run Python script
        let output = Command::new("python3")
            .arg(&temp_script)
            .arg(bbpe_model_path)
            .arg(bpe_vocab_path)
            .output();

        // Clean up temp script
        let _ = fs::remove_file(&temp_script);

        match output {
            Ok(result) if result.status.success() => {
                Ok(())
            }
            Ok(result) => {
                let error_msg = String::from_utf8_lossy(&result.stderr);
                // Provide helpful error message
                if error_msg.contains("No module named 'sentencepiece'") {
                    Err(anyhow::anyhow!(
                        "sentencepiece module not found. Install it with: pip install sentencepiece\n\
                        Alternatively, download sherpa-onnx's export script from https://github.com/k2-fsa/sherpa-onnx/blob/master/scripts/bpe/export_bpe_vocab.py\n\
                        Then run: python scripts/bpe/export_bpe_vocab.py --bpe-model <bbpe.model>"
                    ))
                } else {
                    Err(anyhow::anyhow!("{}", error_msg.trim()))
                }
            }
            Err(e) => {
                Err(anyhow::anyhow!("Python3 not available: {}. Install Python3 or use sherpa-onnx's export script manually.", e))
            }
        }
    }

    /// Create hotwords file from custom words list
    /// Returns the path to the created hotwords file
    fn create_hotwords_file(
        _app_handle: &AppHandle,
        custom_words: &[String],
        vocab_path: &Path,
        modeling_unit: &str,
    ) -> Result<PathBuf> {
        use std::process::Command;

        // Create temporary hotwords file with unique name
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let hotwords_file = std::env::temp_dir().join(format!("hotwords_{}.txt", timestamp));

        // For cjkchar modeling unit, we can write characters directly (space-separated)
        // For cjkchar+bpe modeling unit, we need to tokenize the words into BPE tokens
        let mut hotwords_content = String::new();

        // Get model directory and related files
        let model_dir = vocab_path.parent().unwrap();
        let tokens_file = model_dir.join("tokens.txt");
        let bbpe_model_file = model_dir.join("bbpe.model");

        info!(
            "[Hotwords] Tokenizing {} words using modeling_unit={}, vocab_path={:?}",
            custom_words.len(),
            modeling_unit,
            vocab_path
        );

        let mut success_count = 0;
        let mut fail_count = 0;

        for word in custom_words {
            let mut tokenized = None;

            match modeling_unit {
                "cjkchar" => {
                    // For cjkchar, use the word directly without space separation
                    // Example: "贝壳" -> "贝壳"
                    if !word.is_empty() {
                        tokenized = Some(word.clone());
                        info!(
                            "[Hotwords] Using '{}' directly (cjkchar: no tokenization needed)",
                            word
                        );
                    }
                }
                "cjkchar+bpe" => {
                    // Method 1: Try sherpa-onnx-cli text2token (requires both tokens.txt and bbpe.model)
                    if tokens_file.exists() && bbpe_model_file.exists() {
                        let output = Command::new("sherpa-onnx-cli")
                            .arg("text2token")
                            .arg("--tokens")
                            .arg(&tokens_file)
                            .arg("--tokens-type")
                            .arg("cjkchar+bpe")
                            .arg("--bpe-model")
                            .arg(&bbpe_model_file)
                            .arg("--text")
                            .arg(word)
                            .output();

                        match output {
                            Ok(result) if result.status.success() => {
                                let tokenized_str =
                                    String::from_utf8_lossy(&result.stdout).trim().to_string();
                                if !tokenized_str.is_empty() {
                                    tokenized = Some(tokenized_str);
                                    info!(
                                        "[Hotwords] Tokenized '{}' -> '{}'",
                                        word,
                                        tokenized.as_ref().unwrap()
                                    );
                                } else {
                                    warn!(
                                        "[Hotwords] sherpa-onnx-cli returned empty output for '{}'",
                                        word
                                    );
                                }
                            }
                            Ok(result) => {
                                let stderr = String::from_utf8_lossy(&result.stderr);
                                let stdout = String::from_utf8_lossy(&result.stdout);
                                warn!("[Hotwords] sherpa-onnx-cli failed for '{}': stderr={}, stdout={}", word, stderr, stdout);
                            }
                            Err(e) => {
                                debug!("[Hotwords] sherpa-onnx-cli not available or failed for '{}': {}", word, e);
                            }
                        }
                    }

                    // Method 2: Fallback to Python + sentencepiece (if sherpa-onnx-cli failed)
                    if tokenized.is_none() && bbpe_model_file.exists() {
                        // Use Python script to tokenize using sentencepiece
                        let python_script = format!(
                            r#"
import sentencepiece as spm
import sys

if len(sys.argv) != 3:
    print("Usage: python script.py <bbpe.model> <text>", file=sys.stderr)
    sys.exit(1)

bbpe_model = sys.argv[1]
text = sys.argv[2]

try:
    sp = spm.SentencePieceProcessor()
    sp.load(bbpe_model)
    tokens = sp.encode(text, out_type=str)
    print(' '.join(tokens))
except Exception as e:
    print(f"Error: {{e}}", file=sys.stderr)
    sys.exit(1)
"#
                        );

                        let temp_script =
                            std::env::temp_dir().join(format!("tokenize_hotword_{}.py", timestamp));
                        if fs::write(&temp_script, python_script).is_ok() {
                            let output = Command::new("python3")
                                .arg(&temp_script)
                                .arg(&bbpe_model_file)
                                .arg(word)
                                .output();

                            let _ = fs::remove_file(&temp_script); // Clean up

                            match output {
                                Ok(result) if result.status.success() => {
                                    let tokenized_str =
                                        String::from_utf8_lossy(&result.stdout).trim().to_string();
                                    if !tokenized_str.is_empty() {
                                        tokenized = Some(tokenized_str);
                                        info!("[Hotwords] Tokenized '{}' -> '{}' (using Python+sentencepiece)", word, tokenized.as_ref().unwrap());
                                    }
                                }
                                Ok(result) => {
                                    let stderr = String::from_utf8_lossy(&result.stderr);
                                    debug!(
                                        "[Hotwords] Python tokenization failed for '{}': {}",
                                        word, stderr
                                    );
                                }
                                Err(_) => {
                                    debug!(
                                        "[Hotwords] Python3 not available for tokenizing '{}'",
                                        word
                                    );
                                }
                            }
                        }
                    }
                }
                _ => {
                    warn!(
                        "[Hotwords] Unknown modeling unit: {}, cannot tokenize '{}'",
                        modeling_unit, word
                    );
                }
            }

            // Write tokenized result or skip if failed
            if let Some(tokenized_str) = tokenized {
                hotwords_content.push_str(&format!("{}\n", tokenized_str));
                success_count += 1;
            } else {
                warn!(
                    "[Hotwords] Failed to tokenize '{}', skipping this hotword.",
                    word
                );
                fail_count += 1;
            }
        }

        if success_count == 0 {
            return Err(anyhow::anyhow!(
                "Failed to tokenize any hotwords. For cjkchar+bpe models, please install sherpa-onnx-cli or Python sentencepiece (pip install sentencepiece)"
            ));
        }

        fs::write(&hotwords_file, hotwords_content)?;
        info!(
            "[Hotwords] Created hotwords file with {}/{} successfully tokenized words at: {:?}",
            success_count,
            custom_words.len(),
            hotwords_file
        );

        if fail_count > 0 {
            warn!(
                "[Hotwords] {} words failed to tokenize and were skipped",
                fail_count
            );
        }

        Ok(hotwords_file)
    }

    /// Check if Python and funasr-onnx are available
    fn check_python_environment(&self) -> Result<()> {
        use std::process::Command;

        // Check Python availability
        let python_check = Command::new("python3").arg("--version").output();

        match python_check {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                debug!("Python version: {}", version);
            }
            Ok(_) => {
                return Err(anyhow::anyhow!("Python3 is not available or not working"));
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to check Python: {}", e));
            }
        }

        // Check funasr-onnx availability
        let funasr_check = Command::new("python3")
            .arg("-c")
            .arg("import funasr_onnx; print('OK')")
            .output();

        match funasr_check {
            Ok(output) if output.status.success() => {
                debug!("funasr-onnx is available");
                Ok(())
            }
            Ok(_) => Err(anyhow::anyhow!(
                "funasr-onnx is not installed. Install with: pip install -U funasr-onnx"
            )),
            Err(e) => Err(anyhow::anyhow!("Failed to check funasr-onnx: {}", e)),
        }
    }

    /// Create Python inference script for SeACo Paraformer
    fn create_seaco_inference_script(&self) -> Result<PathBuf> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let script_path = std::env::temp_dir().join(format!("seaco_inference_{}.py", timestamp));

        let script_content = r#"#!/usr/bin/env python3
# -*- encoding: utf-8 -*-
# SeACo Paraformer inference script for KeVoiceInput
import sys
import json
import numpy as np
from pathlib import Path

try:
    from funasr_onnx import SeacoParaformer
except ImportError:
    print("ERROR: funasr-onnx not installed. Install with: pip install -U funasr-onnx", file=sys.stderr)
    sys.exit(1)

def main():
    if len(sys.argv) < 4:
        print("Usage: python script.py <model_dir> <audio_file> <hotwords>", file=sys.stderr)
        sys.exit(1)
    
    model_dir = sys.argv[1]
    audio_file = sys.argv[2]
    hotwords = sys.argv[3] if len(sys.argv) > 3 else ""
    
    try:
        # Load model
        model = SeacoParaformer(
            model_dir=model_dir,
            batch_size=1,
            device_id="-1",  # CPU
            quantize=False,
        )
        
        # Load audio (expects f32 samples at 16kHz)
        # SeacoParaformer expects numpy array or file path
        audio_data = np.fromfile(audio_file, dtype=np.float32)
        
        # Convert to WAV file temporarily (SeacoParaformer works better with file paths)
        # Or use numpy array directly - let's try numpy array first
        # SeacoParaformer.__call__ accepts List[str] (file paths) or List[np.ndarray]
        # So we can pass [audio_data] directly
        
        # Run inference
        if hotwords:
            results = model([audio_data], hotwords=hotwords)
        else:
            results = model([audio_data], hotwords="")
        
        # Output result as JSON
        if results and len(results) > 0:
            result = results[0]
            output = {
                "text": result.get("preds", ""),
                "success": True
            }
        else:
            output = {
                "text": "",
                "success": False,
                "error": "No results returned"
            }
        
        print(json.dumps(output, ensure_ascii=False))
        
    except Exception as e:
        import traceback
        error_output = {
            "text": "",
            "success": False,
            "error": str(e),
            "traceback": traceback.format_exc()
        }
        print(json.dumps(error_output, ensure_ascii=False), file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
"#;

        fs::write(&script_path, script_content)?;

        // Make script executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms)?;
        }

        Ok(script_path)
    }

    pub fn is_model_loaded(&self) -> bool {
        let engine = self.engine.lock().unwrap();
        engine.is_some()
    }

    pub fn unload_model(&self) -> Result<()> {
        let unload_start = std::time::Instant::now();
        debug!("Starting to unload model");

        {
            let mut engine = self.engine.lock().unwrap();
            if let Some(ref mut loaded_engine) = *engine {
                match loaded_engine {
                    LoadedEngine::Whisper(ref mut e) => e.unload_model(),
                    LoadedEngine::Paraformer(_) => {
                        // Paraformer engine is dropped automatically when set to None
                    }
                    LoadedEngine::Transducer(_) => {
                        // Transducer engine is dropped automatically when set to None
                    }
                    LoadedEngine::FireRedAsr(_) => {
                        // FireRedAsr engine is dropped automatically when set to None
                    }
                    // SeacoParaformer now uses ParaformerRecognizer, handled above
                }
            }
            *engine = None; // Drop the engine to free memory
        }
        {
            let mut current_model = self.current_model_id.lock().unwrap();
            *current_model = None;
        }

        // Emit unloaded event
        let _ = self.app_handle.emit(
            "model-state-changed",
            ModelStateEvent {
                event_type: "unloaded".to_string(),
                model_id: None,
                model_name: None,
                error: None,
            },
        );

        let unload_duration = unload_start.elapsed();
        debug!(
            "Model unloaded manually (took {}ms)",
            unload_duration.as_millis()
        );
        Ok(())
    }

    /// Cancel ongoing transcription
    pub fn cancel_transcription(&self) {
        self.cancellation_flag.store(true, Ordering::Relaxed);
        info!("Transcription cancellation flag set");
    }

    /// Check if transcription is cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancellation_flag.load(Ordering::Relaxed)
    }

    /// Reset cancellation flag
    pub fn reset_cancellation_flag(&self) {
        self.cancellation_flag.store(false, Ordering::Relaxed);
    }

    /// Unloads the model immediately if the setting is enabled and the model is loaded
    pub fn maybe_unload_immediately(&self, context: &str) {
        let settings = get_settings(&self.app_handle);
        if settings.model_unload_timeout == ModelUnloadTimeout::Immediately
            && self.is_model_loaded()
        {
            info!("Immediately unloading model after {}", context);
            if let Err(e) = self.unload_model() {
                warn!("Failed to immediately unload model: {}", e);
            }
        }
    }

    pub fn load_model(&self, model_id: &str) -> Result<()> {
        let load_start = std::time::Instant::now();
        debug!("Starting to load model: {}", model_id);

        // Emit loading started event
        let _ = self.app_handle.emit(
            "model-state-changed",
            ModelStateEvent {
                event_type: "loading_started".to_string(),
                model_id: Some(model_id.to_string()),
                model_name: None,
                error: None,
            },
        );

        let model_info = self
            .model_manager
            .get_model_info(model_id)
            .ok_or_else(|| anyhow::anyhow!("Model not found: {}", model_id))?;

        if !model_info.is_downloaded {
            let error_msg = "Model not downloaded";
            let _ = self.app_handle.emit(
                "model-state-changed",
                ModelStateEvent {
                    event_type: "loading_failed".to_string(),
                    model_id: Some(model_id.to_string()),
                    model_name: Some(model_info.name.clone()),
                    error: Some(error_msg.to_string()),
                },
            );
            return Err(anyhow::anyhow!(error_msg));
        }

        let model_path = self.model_manager.get_model_path(model_id)?;

        // Create appropriate engine based on model type
        let loaded_engine = match model_info.engine_type {
            EngineType::Api => {
                // API models don't need to be loaded locally
                return Err(anyhow::anyhow!(
                    "API models cannot be loaded as local engines"
                ));
            }
            EngineType::Whisper => {
                let mut engine = WhisperEngine::new();
                engine.load_model(&model_path).map_err(|e| {
                    let error_msg = format!("Failed to load whisper model {}: {}", model_id, e);
                    let _ = self.app_handle.emit(
                        "model-state-changed",
                        ModelStateEvent {
                            event_type: "loading_failed".to_string(),
                            model_id: Some(model_id.to_string()),
                            model_name: Some(model_info.name.clone()),
                            error: Some(error_msg.clone()),
                        },
                    );
                    anyhow::anyhow!(error_msg)
                })?;
                LoadedEngine::Whisper(engine)
            }
            EngineType::Paraformer => {
                // Paraformer models are directory-based, need model.onnx and tokens.txt
                let model_dir = &model_path;

                // Try to find model file (prefer int8, fallback to fp32)
                let model_file = if model_dir.join("model.int8.onnx").exists() {
                    model_dir.join("model.int8.onnx")
                } else if model_dir.join("model.onnx").exists() {
                    model_dir.join("model.onnx")
                } else {
                    return Err(anyhow::anyhow!(
                        "Paraformer model file not found in: {:?}",
                        model_dir
                    ));
                };

                let tokens_file = model_dir.join("tokens.txt");

                if !tokens_file.exists() {
                    return Err(anyhow::anyhow!(
                        "Paraformer tokens file not found: {:?}",
                        tokens_file
                    ));
                }

                // Check for model_eb.onnx (SeACo Paraformer support)
                let model_eb_file = model_dir.join("model_eb.onnx");
                let model_eb_path = if model_eb_file.exists() {
                    info!("Found model_eb.onnx - SeACo Paraformer with hotword support");
                    Some(model_eb_file.to_string_lossy().to_string())
                } else {
                    None
                };

                // Load hotwords for standard Paraformer (if model_eb exists)
                let settings = get_settings(&self.app_handle);
                let mut all_hotwords = settings.custom_words.clone();
                
                // Load hotwords from selected files
                for file_path in &settings.selected_hotword_files {
                    if let Ok(content) = fs::read_to_string(file_path) {
                        let words: Vec<String> = content
                            .lines()
                            .map(|line| line.trim().to_string())
                            .filter(|line| !line.is_empty())
                            .collect();
                        info!("[Hotwords] Loaded {} words from file: {:?}", words.len(), file_path);
                        all_hotwords.extend(words);
                    } else {
                        warn!("[Hotwords] Failed to read hotword file: {:?}", file_path);
                    }
                }
                
                // Remove duplicates while preserving order
                let mut seen = std::collections::HashSet::new();
                all_hotwords.retain(|word| seen.insert(word.clone()));
                
                info!("[Hotwords] Total hotwords for Paraformer: {} (custom_words: {}, from files: {})", 
                    all_hotwords.len(), 
                    settings.custom_words.len(),
                    all_hotwords.len() - settings.custom_words.len());

                // Create hotwords file for Paraformer (one word per line format, only if model_eb exists)
                let hotwords_file = if !all_hotwords.is_empty() && model_eb_path.is_some() {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos();
                    let hotwords_file_path = std::env::temp_dir().join(format!("paraformer_hotwords_{}.txt", timestamp));
                    
                    // Paraformer uses one word per line format
                    let hotwords_content = all_hotwords.join("\n");
                    
                    if let Err(e) = fs::write(&hotwords_file_path, hotwords_content) {
                        warn!("[Hotwords] Failed to create hotwords file for Paraformer: {}", e);
                        None
                    } else {
                        info!("[Hotwords] Created Paraformer hotwords file with {} words at: {:?}", 
                            all_hotwords.len(), hotwords_file_path);
                        Some(hotwords_file_path.to_string_lossy().to_string())
                    }
                } else {
                    if all_hotwords.is_empty() {
                        info!("[Hotwords] No hotwords to load for Paraformer");
                    } else if model_eb_path.is_none() {
                        info!("[Hotwords] model_eb.onnx not found, Paraformer will work without hotwords");
                    }
                    None
                };

                // Paraformer config - supports both standard Paraformer and SeACo Paraformer
                let config = ParaformerConfig {
                    model: model_file.to_string_lossy().to_string(),
                    tokens: tokens_file.to_string_lossy().to_string(),
                    model_eb: model_eb_path,  // 新增：SeACo Paraformer 支持
                    hotwords_file: hotwords_file.clone(),  // 热词文件路径
                    hotwords_score: if hotwords_file.is_some() { 2.0 } else { 0.0 },  // 热词分数
                    num_threads: Some(1),
                    provider: None,
                    debug: false,
                };

                let recognizer = ParaformerRecognizer::new(config).map_err(|e| {
                    let error_msg = format!("Failed to load paraformer model {}: {}", model_id, e);
                    let _ = self.app_handle.emit(
                        "model-state-changed",
                        ModelStateEvent {
                            event_type: "loading_failed".to_string(),
                            model_id: Some(model_id.to_string()),
                            model_name: Some(model_info.name.clone()),
                            error: Some(error_msg.clone()),
                        },
                    );
                    anyhow::anyhow!(error_msg)
                })?;

                LoadedEngine::Paraformer(recognizer)
            }
            EngineType::Transducer => {
                // Verify library is loaded before proceeding
                crate::managers::sherpa_debug::verify_sherpa_library();
                
                // Transducer models (e.g., zipformer, conformer) are directory-based
                // Need encoder.onnx, decoder.onnx, joiner.onnx, tokens.txt, and optionally bbpe.model
                let model_dir = &model_path;

                // Try to find encoder file (support multiple naming patterns)
                let encoder_file = if model_dir.join("encoder-epoch-99-avg-1.onnx").exists() {
                    // Conformer model pattern
                    model_dir.join("encoder-epoch-99-avg-1.onnx")
                } else if model_dir.join("encoder-epoch-34-avg-19.int8.onnx").exists() {
                    // Zipformer int8 pattern
                    model_dir.join("encoder-epoch-34-avg-19.int8.onnx")
                } else if model_dir.join("encoder-epoch-34-avg-19.onnx").exists() {
                    // Zipformer fp32 pattern
                    model_dir.join("encoder-epoch-34-avg-19.onnx")
                } else {
                    return Err(anyhow::anyhow!(
                        "Transducer encoder file not found in: {:?}. Supported patterns: encoder-epoch-99-avg-1.onnx, encoder-epoch-34-avg-19.onnx",
                        model_dir
                    ));
                };

                // Try to find decoder file
                let decoder_file = if model_dir.join("decoder-epoch-99-avg-1.onnx").exists() {
                    // Conformer model pattern
                    model_dir.join("decoder-epoch-99-avg-1.onnx")
                } else if model_dir.join("decoder-epoch-34-avg-19.onnx").exists() {
                    // Zipformer pattern
                    model_dir.join("decoder-epoch-34-avg-19.onnx")
                } else {
                    return Err(anyhow::anyhow!(
                        "Transducer decoder file not found in: {:?}. Supported patterns: decoder-epoch-99-avg-1.onnx, decoder-epoch-34-avg-19.onnx",
                        model_dir
                    ));
                };

                // Try to find joiner file
                let joiner_file = if model_dir.join("joiner-epoch-99-avg-1.onnx").exists() {
                    // Conformer model pattern
                    model_dir.join("joiner-epoch-99-avg-1.onnx")
                } else if model_dir.join("joiner-epoch-34-avg-19.int8.onnx").exists() {
                    // Zipformer int8 pattern
                    model_dir.join("joiner-epoch-34-avg-19.int8.onnx")
                } else if model_dir.join("joiner-epoch-34-avg-19.onnx").exists() {
                    // Zipformer fp32 pattern
                    model_dir.join("joiner-epoch-34-avg-19.onnx")
                } else {
                    return Err(anyhow::anyhow!(
                        "Transducer joiner file not found in: {:?}. Supported patterns: joiner-epoch-99-avg-1.onnx, joiner-epoch-34-avg-19.onnx",
                        model_dir
                    ));
                };

                let tokens_file = model_dir.join("tokens.txt");
                let bbpe_model_file = model_dir.join("bbpe.model");

                // Determine modeling unit: if bbpe.model exists, use cjkchar+bpe, otherwise cjkchar
                let modeling_unit = if bbpe_model_file.exists() {
                    "cjkchar+bpe"
                } else {
                    "cjkchar"
                };
                info!(
                    "[Transducer] Detected modeling unit: {} (bbpe.model exists: {})",
                    modeling_unit,
                    bbpe_model_file.exists()
                );

                if !tokens_file.exists() {
                    return Err(anyhow::anyhow!(
                        "Transducer tokens file not found: {:?}",
                        tokens_file
                    ));
                }

                // Export bpe.vocab from bbpe.model if it doesn't exist (only for cjkchar+bpe models)
                let bpe_vocab_file = if modeling_unit == "cjkchar+bpe" {
                    let bpe_vocab_file = model_dir.join("bpe.vocab");
                    let bpe_vocab_exists = bpe_vocab_file.exists();
                    info!(
                        "bpe.vocab exists: {} at {:?}",
                        bpe_vocab_exists, bpe_vocab_file
                    );

                    if !bpe_vocab_exists {
                        match Self::export_bpe_vocab(&bbpe_model_file, &bpe_vocab_file) {
                            Ok(()) => {
                                info!("Successfully exported bpe.vocab from bbpe.model");
                            }
                            Err(e) => {
                                warn!("Failed to export bpe.vocab: {}. Model will work without hotwords support. To enable hotwords, install Python sentencepiece: pip install sentencepiece", e);
                            }
                        }
                    }
                    Some(bpe_vocab_file)
                } else {
                    None
                };

                // Collect all hotwords from custom_words and selected hotword files
                let settings = get_settings(&self.app_handle);
                let mut all_hotwords = settings.custom_words.clone();
                
                // Load hotwords from selected files
                for file_path in &settings.selected_hotword_files {
                    if let Ok(content) = fs::read_to_string(file_path) {
                        let words: Vec<String> = content
                            .lines()
                            .map(|line| line.trim().to_string())
                            .filter(|line| !line.is_empty())
                            .collect();
                        info!("[Hotwords] Loaded {} words from file: {:?}", words.len(), file_path);
                        all_hotwords.extend(words);
                    } else {
                        warn!("[Hotwords] Failed to read hotword file: {:?}", file_path);
                    }
                }
                
                // Remove duplicates while preserving order
                let mut seen = std::collections::HashSet::new();
                all_hotwords.retain(|word| seen.insert(word.clone()));
                
                info!("[Hotwords] Total hotwords (custom_words + files): {} (custom_words: {}, from files: {})", 
                    all_hotwords.len(), 
                    settings.custom_words.len(),
                    all_hotwords.len() - settings.custom_words.len());
                info!("[Hotwords] Checking conditions: total_hotwords={}, modeling_unit={}, bpe.vocab={:?}", 
                    all_hotwords.len(), modeling_unit, bpe_vocab_file);

                let hotwords_file = if !all_hotwords.is_empty() {
                    // For cjkchar models, we can create hotwords file directly (no bpe.vocab needed)
                    // For cjkchar+bpe models, we need bpe.vocab
                    let can_create_hotwords = match modeling_unit {
                        "cjkchar" => {
                            info!("[Hotwords] Using cjkchar modeling unit, can create hotwords file directly");
                            true
                        }
                        "cjkchar+bpe" => {
                            if let Some(ref bpe_vocab) = bpe_vocab_file {
                                bpe_vocab.exists()
                            } else {
                                false
                            }
                        }
                        _ => false,
                    };

                    if can_create_hotwords {
                        info!("[Hotwords] All conditions met, creating hotwords file...");
                        // For cjkchar, pass tokens.txt path; for cjkchar+bpe, pass bpe.vocab path
                        let vocab_path = if modeling_unit == "cjkchar" {
                            &tokens_file
                        } else {
                            bpe_vocab_file.as_ref().unwrap()
                        };

                        match Self::create_hotwords_file(
                            &self.app_handle,
                            &all_hotwords,
                            vocab_path,
                            modeling_unit,
                        ) {
                            Ok(path) => {
                                info!("[Hotwords] Successfully created hotwords file with {} words at: {:?}", all_hotwords.len(), path);
                                // Verify file was created and read its content for debugging
                                if let Ok(content) = fs::read_to_string(&path) {
                                    let lines: Vec<&str> =
                                        content.lines().filter(|l| !l.is_empty()).collect();
                                    info!(
                                        "[Hotwords] Hotwords file contains {} lines (non-empty)",
                                        lines.len()
                                    );
                                    if lines.len() <= 10 {
                                        info!("[Hotwords] Hotwords file content: {:?}", lines);
                                    } else {
                                        info!(
                                            "[Hotwords] Hotwords file content (first 10): {:?}",
                                            &lines[..10]
                                        );
                                    }
                                }
                                Some(path)
                            }
                            Err(e) => {
                                warn!("[Hotwords] Failed to create hotwords file: {}. Continuing without hotwords.", e);
                                None
                            }
                        }
                    } else {
                        if modeling_unit == "cjkchar+bpe" {
                            warn!("[Hotwords] bpe.vocab not found (path: {:?}, exists: {}), hotwords disabled. To enable hotwords, ensure bpe.vocab exists.", 
                                bpe_vocab_file.as_ref().map(|p| p.as_path()), bpe_vocab_file.as_ref().map(|p| p.exists()).unwrap_or(false));
                        }
                        None
                    }
                } else {
                    info!(
                        "[Hotwords] No hotwords configured (custom_words: {}, selected_files: {}), hotwords disabled",
                        settings.custom_words.len(),
                        settings.selected_hotword_files.len()
                    );
                    None
                };

                // Build TransducerConfig using sherpa-rs high-level API
                let mut config = TransducerConfig {
                    encoder: encoder_file.to_string_lossy().to_string(),
                    decoder: decoder_file.to_string_lossy().to_string(),
                    joiner: joiner_file.to_string_lossy().to_string(),
                    tokens: tokens_file.to_string_lossy().to_string(),
                    num_threads: 1,
                    sample_rate: 16000,
                    feature_dim: 80,
                    decoding_method: "modified_beam_search".to_string(),
                    hotwords_file: hotwords_file.as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    hotwords_score: if hotwords_file.is_some() { 2.0 } else { 0.0 },
                    modeling_unit: modeling_unit.to_string(),
                    bpe_vocab: bpe_vocab_file.as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    blank_penalty: 0.0,
                    model_type: "transducer".to_string(),
                    debug: false,
                    provider: Some("cpu".to_string()),
                };

                info!("[Transducer] Creating recognizer with config: encoder={:?}, decoder={:?}, joiner={:?}, tokens={:?}, modeling_unit={}, hotwords_file={:?}, bpe_vocab={:?}",
                    config.encoder, config.decoder, config.joiner, config.tokens,
                    config.modeling_unit, config.hotwords_file, config.bpe_vocab);

                // Create recognizer using sherpa-rs high-level API
                let recognizer = TransducerRecognizer::new(config).map_err(|e| {
                    let error_msg = format!("Failed to load transducer model {}: {}", model_id, e);
                    let _ = self.app_handle.emit(
                        "model-state-changed",
                        ModelStateEvent {
                            event_type: "loading_failed".to_string(),
                            model_id: Some(model_id.to_string()),
                            model_name: Some(model_info.name.clone()),
                            error: Some(error_msg.clone()),
                        },
                    );
                    anyhow::anyhow!(error_msg)
                })?;

                info!("[Transducer] Recognizer created successfully");

                LoadedEngine::Transducer(TransducerEngine {
                    recognizer,
                    hotwords_file,
                })
            }
            EngineType::FireRedAsr => {
                // FireRedAsr models are directory-based, need encoder.onnx, decoder.onnx, and tokens.txt
                let model_dir = &model_path;

                // Try to find encoder file (prefer int8, fallback to fp32)
                let encoder_file = if model_dir.join("encoder.int8.onnx").exists() {
                    model_dir.join("encoder.int8.onnx")
                } else if model_dir.join("encoder.onnx").exists() {
                    model_dir.join("encoder.onnx")
                } else {
                    return Err(anyhow::anyhow!(
                        "FireRedAsr encoder file not found in: {:?}. Expected encoder.int8.onnx or encoder.onnx",
                        model_dir
                    ));
                };

                // Try to find decoder file (prefer int8, fallback to fp32)
                let decoder_file = if model_dir.join("decoder.int8.onnx").exists() {
                    model_dir.join("decoder.int8.onnx")
                } else if model_dir.join("decoder.onnx").exists() {
                    model_dir.join("decoder.onnx")
                } else {
                    return Err(anyhow::anyhow!(
                        "FireRedAsr decoder file not found in: {:?}. Expected decoder.int8.onnx or decoder.onnx",
                        model_dir
                    ));
                };

                let tokens_file = model_dir.join("tokens.txt");
                if !tokens_file.exists() {
                    return Err(anyhow::anyhow!(
                        "FireRedAsr tokens file not found: {:?}",
                        tokens_file
                    ));
                }

                // Convert paths to CStrings
                let encoder_path = CString::new(encoder_file.to_string_lossy().as_ref())
                    .map_err(|e| anyhow::anyhow!("Failed to create encoder path CString: {}", e))?;
                let decoder_path = CString::new(decoder_file.to_string_lossy().as_ref())
                    .map_err(|e| anyhow::anyhow!("Failed to create decoder path CString: {}", e))?;
                let tokens_path = CString::new(tokens_file.to_string_lossy().as_ref())
                    .map_err(|e| anyhow::anyhow!("Failed to create tokens path CString: {}", e))?;

                // Build offline FireRedAsr model config
                let mut fire_red_asr_model_cfg =
                    unsafe { std::mem::zeroed::<SherpaOnnxOfflineFireRedAsrModelConfig>() };
                fire_red_asr_model_cfg.encoder = encoder_path.as_ptr();
                fire_red_asr_model_cfg.decoder = decoder_path.as_ptr();

                // Build offline model config
                // Initialize with zeroed memory first, then set fields
                let mut model_cfg = unsafe { std::mem::zeroed::<SherpaOnnxOfflineModelConfig>() };
                model_cfg.fire_red_asr = fire_red_asr_model_cfg;
                model_cfg.tokens = tokens_path.as_ptr();
                model_cfg.num_threads = 1;
                model_cfg.debug = 0;
                
                // Create provider CString (use "cpu" as default)
                let provider_cstr = CString::new("cpu").map_err(|e| {
                    anyhow::anyhow!("Failed to create provider CString: {}", e)
                })?;
                model_cfg.provider = provider_cstr.as_ptr();
                
                // Create model_type CString
                let model_type_cstr = CString::new("fire_red_asr").map_err(|e| {
                    anyhow::anyhow!("Failed to create model_type CString: {}", e)
                })?;
                model_cfg.model_type = model_type_cstr.as_ptr();
                
                // Explicitly set unused pointer fields to NULL for safety
                model_cfg.modeling_unit = std::ptr::null();
                model_cfg.bpe_vocab = std::ptr::null();
                model_cfg.telespeech_ctc = std::ptr::null();
                // Other model types (transducer, paraformer, whisper, etc.) are already zeroed by mem::zeroed()

                // Build feature config (16kHz sample rate, 80 feature dim for FireRedAsr)
                let feat_cfg = SherpaOnnxFeatureConfig {
                    sample_rate: 16000,
                    feature_dim: 80,
                };

                // Build recognizer config
                // FireRedAsr uses greedy_search decoding method
                let decoding_method = CString::new("greedy_search").map_err(|e| {
                    anyhow::anyhow!("Failed to create decoding method CString: {}", e)
                })?;

                // Initialize recognizer config with zeroed memory first, then set fields
                let mut recognizer_cfg =
                    unsafe { std::mem::zeroed::<SherpaOnnxOfflineRecognizerConfig>() };
                recognizer_cfg.model_config = model_cfg;
                recognizer_cfg.feat_config = feat_cfg;
                recognizer_cfg.decoding_method = decoding_method.as_ptr();
                recognizer_cfg.max_active_paths = 4;
                recognizer_cfg.hotwords_file = std::ptr::null();
                recognizer_cfg.hotwords_score = 0.0;
                recognizer_cfg.blank_penalty = 0.0;
                // Explicitly set pointer fields to NULL for safety
                recognizer_cfg.rule_fsts = std::ptr::null();
                recognizer_cfg.rule_fars = std::ptr::null();
                // Explicitly initialize lm_config struct fields
                recognizer_cfg.lm_config.model = std::ptr::null();
                recognizer_cfg.lm_config.scale = 0.0;
                // Explicitly initialize hr (HomophoneReplacer) struct fields
                recognizer_cfg.hr.dict_dir = std::ptr::null();
                recognizer_cfg.hr.lexicon = std::ptr::null();
                recognizer_cfg.hr.rule_fsts = std::ptr::null();

                use std::io::Write;
                let _ = std::io::stderr().flush();
                std::thread::sleep(std::time::Duration::from_millis(100));

                // Create recognizer
                let _ = std::io::stderr().flush();
                std::thread::sleep(std::time::Duration::from_millis(200));
                let recognizer = unsafe { 
                    let _ = std::io::stderr().flush();
                    let config_ptr = &recognizer_cfg as *const _;
                    let _ = std::io::stderr().flush();
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    
                    let result = SherpaOnnxCreateOfflineRecognizer(&recognizer_cfg);
                    
                    let _ = std::io::stderr().flush();
                    result
                };
                if recognizer.is_null() {
                    return Err(anyhow::anyhow!("Failed to create FireRedAsr recognizer"));
                }

                // Keep CStrings alive by storing them in the engine
                LoadedEngine::FireRedAsr(FireRedAsrEngine {
                    recognizer,
                    _encoder_path: encoder_path,
                    _decoder_path: decoder_path,
                    _tokens_path: tokens_path,
                    _decoding_method: decoding_method,
                    _provider: provider_cstr,
                    _model_type: model_type_cstr,
                })
            }
        };

        // Update the current engine and model ID
        {
            let mut engine = self.engine.lock().unwrap();
            *engine = Some(loaded_engine);
        }
        {
            let mut current_model = self.current_model_id.lock().unwrap();
            *current_model = Some(model_id.to_string());
        }

        // Emit loading completed event
        let _ = self.app_handle.emit(
            "model-state-changed",
            ModelStateEvent {
                event_type: "loading_completed".to_string(),
                model_id: Some(model_id.to_string()),
                model_name: Some(model_info.name.clone()),
                error: None,
            },
        );

        let load_duration = load_start.elapsed();
        debug!(
            "Successfully loaded transcription model: {} (took {}ms)",
            model_id,
            load_duration.as_millis()
        );
        Ok(())
    }

    /// Kicks off the model loading in a background thread if it's not already loaded
    pub fn initiate_model_load(&self) {
        let mut is_loading = self.is_loading.lock().unwrap();
        if *is_loading || self.is_model_loaded() {
            return;
        }

        *is_loading = true;
        let self_clone = self.clone();
        thread::spawn(move || {
            let settings = get_settings(&self_clone.app_handle);
            if let Err(e) = self_clone.load_model(&settings.selected_model) {
                error!("Failed to load model: {}", e);
            }
            let mut is_loading = self_clone.is_loading.lock().unwrap();
            *is_loading = false;
            self_clone.loading_condvar.notify_all();
        });
    }

    pub fn get_current_model(&self) -> Option<String> {
        let current_model = self.current_model_id.lock().unwrap();
        current_model.clone()
    }

    pub fn model_manager(&self) -> &Arc<ModelManager> {
        &self.model_manager
    }


    pub fn transcribe(&self, audio: Vec<f32>) -> Result<TranscriptionResult> {
        // Update last activity timestamp
        self.last_activity.store(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            Ordering::Relaxed,
        );

        let st = std::time::Instant::now();

        debug!("Audio vector length: {}", audio.len());

        if audio.is_empty() {
            debug!("Empty audio vector");
            self.maybe_unload_immediately("empty audio");
            return Ok(TranscriptionResult {
                original_text: String::new(),
                itn_text: None,
                final_text: String::new(),
                model_name: None,
                is_api: false,
            });
        }

        // Get current settings for configuration
        let settings = get_settings(&self.app_handle);

        use std::fs::OpenOptions;
        use std::io::Write;
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
        {
            let _ = writeln!(
                file,
                r#"{{"sessionId":"debug-session","runId":"run1","hypothesisId":"A","location":"transcription.rs:1348","message":"Checking use_transcription_api setting","data":{{"use_transcription_api":{},"selected_config_id":"{:?}","audio_len":{}}},"timestamp":{}}}"#,
                settings.use_transcription_api,
                settings.selected_transcription_api_config_id,
                audio.len(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            );
        }

        // Check if transcription was cancelled before starting API transcription
        if self.is_cancelled() {
            self.reset_cancellation_flag();
            return Err(anyhow::anyhow!("Transcription cancelled by user"));
        }

        // Check if API transcription is enabled
        if settings.use_transcription_api {
            debug!("Using API transcription");
            return self.transcribe_with_api(audio, &settings);
        }

        // Otherwise use local model
        // Check if model is loaded, if not try to load it
        {
            // If the model is loading, wait for it to complete.
            let mut is_loading = self.is_loading.lock().unwrap();
            while *is_loading {
                is_loading = self.loading_condvar.wait(is_loading).unwrap();
            }

            let engine_guard = self.engine.lock().unwrap();
            if engine_guard.is_none() {
                return Err(anyhow::anyhow!("Model is not loaded for transcription."));
            }
        }

        // Get current settings for configuration
        let settings = get_settings(&self.app_handle);

        // Perform transcription with the appropriate engine
        let result = {
            let mut engine_guard = self.engine.lock().unwrap();
            let engine = engine_guard.as_mut().ok_or_else(|| {
                anyhow::anyhow!(
                    "Model failed to load after auto-load attempt. Please check your model settings."
                )
            })?;

            match engine {
                LoadedEngine::Whisper(whisper_engine) => {
                    // Normalize language code for Whisper
                    // Convert zh-Hans and zh-Hant to zh since Whisper uses ISO 639-1 codes
                    let whisper_language = if settings.selected_language == "auto" {
                        None
                    } else {
                        let normalized = if settings.selected_language == "zh-Hans"
                            || settings.selected_language == "zh-Hant"
                        {
                            "zh".to_string()
                        } else {
                            settings.selected_language.clone()
                        };
                        Some(normalized)
                    };

                    let params = WhisperInferenceParams {
                        language: whisper_language,
                        translate: settings.translate_to_english,
                        ..Default::default()
                    };

                    whisper_engine
                        .transcribe_samples(audio, Some(params))
                        .map_err(|e| anyhow::anyhow!("Whisper transcription failed: {}", e))?
                }
                LoadedEngine::Paraformer(paraformer_engine) => {
                    // Paraformer automatically detects language (Chinese/Cantonese/English)
                    // Paraformer expects 16kHz mono audio as f32 samples
                    // Our audio is already in f32 format at 16kHz

                    // Transcribe using Paraformer
                    let result = paraformer_engine.transcribe(16000, &audio);

                    // Return in same format as Whisper
                    transcribe_rs::TranscriptionResult {
                        text: result.text,
                        segments: None,
                    }
                }
                LoadedEngine::FireRedAsr(ref mut fire_red_asr_engine) => {
                    // FireRedAsr uses offline recognizer, similar to Transducer
                    // Audio is already in f32 format (16kHz)
                    // No conversion needed - sherpa-onnx expects f32 samples
                    unsafe {
                        let stream = SherpaOnnxCreateOfflineStream(fire_red_asr_engine.recognizer);
                        if stream.is_null() {
                            return Err(anyhow::anyhow!(
                                "Failed to create FireRedAsr offline stream"
                            ));
                        }

                        // Accept waveform (expects f32 samples at 16kHz)
                        SherpaOnnxAcceptWaveformOffline(
                            stream,
                            16000,          // sample_rate
                            audio.as_ptr(), // f32 samples
                            audio.len() as i32,
                        );

                        SherpaOnnxDecodeOfflineStream(fire_red_asr_engine.recognizer, stream);

                        let result_ptr = SherpaOnnxGetOfflineStreamResult(stream);
                        if result_ptr.is_null() {
                            unsafe { SherpaOnnxDestroyOfflineStream(stream) };
                            return Err(anyhow::anyhow!("Failed to get FireRedAsr result"));
                        }

                        let raw = *result_ptr;
                        let text = if !raw.text.is_null() {
                            std::ffi::CStr::from_ptr(raw.text)
                                .to_string_lossy()
                                .into_owned()
                        } else {
                            String::new()
                        };

                        SherpaOnnxDestroyOfflineRecognizerResult(result_ptr);
                        SherpaOnnxDestroyOfflineStream(stream);

                        transcribe_rs::TranscriptionResult {
                            text,
                            segments: None,
                        }
                    }
                }
                LoadedEngine::Transducer(ref mut transducer_engine) => {
                    // Audio is already in f32 format (16kHz)
                    // Use sherpa-rs high-level API for transcription
                    let text = transducer_engine.recognizer.transcribe(16000, &audio);

                    transcribe_rs::TranscriptionResult {
                        text,
                        segments: None,
                    }
                }
                // SeACo Paraformer now uses ParaformerRecognizer (same as Paraformer)
                // Both Paraformer and SeACo Paraformer are handled by the Paraformer case above
            }
        };

        // Apply word correction if custom words are configured
        let corrected_result = if !settings.custom_words.is_empty() {
            apply_custom_words(
                &result.text,
                &settings.custom_words,
                settings.word_correction_threshold,
            )
        } else {
            result.text
        };

        // Filter out filler words and hallucinations
        let filtered_result = filter_transcription_output(&corrected_result);

        let filtered_for_log = filtered_result.clone();
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
        {
            let _ = std::io::Write::write_all(&mut file, format!(r#"{{"id":"log_{}","timestamp":{},"location":"transcription.rs:1503","message":"Before ITN","data":{{"filteredResult":"{}","itnEnabled":{}}},"sessionId":"debug-session","runId":"itn-debug","hypothesisId":"B"}}"#, 
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                filtered_for_log.replace('"', r#"\""#),
                settings.itn_enabled
            ).as_bytes());
        }

        // Don't apply ITN here - it will be applied later in actions.rs
        // after checking for selected text and role matching
        // ITN will only be applied if there's no selected text and no role matched
        let filtered_clone_for_log = filtered_result.clone();

        // Apply hot rules if file exists (on original text, before ITN)
        let app_data_dir = self.app_handle.path().app_data_dir().ok();
        let hot_rules_result = if let Some(app_data_dir) = app_data_dir {
            let hot_rule_path = app_data_dir.join("hot-rule.txt");
            let rules = load_hot_rules(&hot_rule_path);
            if !rules.is_empty() {
                apply_hot_rules(&filtered_result, &rules)
            } else {
                filtered_result
            }
        } else {
            filtered_result
        };

        let et = std::time::Instant::now();
        let translation_note = if settings.translate_to_english {
            " (translated)"
        } else {
            ""
        };
        info!(
            "Transcription completed in {}ms{}",
            (et - st).as_millis(),
            translation_note
        );

        let final_result = hot_rules_result;

        if !final_result.is_empty() {
            info!("Transcription result: {}", final_result);
        }

        if final_result.is_empty() {
            info!("Transcription result is empty");
        }

        self.maybe_unload_immediately("transcription");

        // Return structured result
        // original_text: text before ITN (used for role matching)
        // itn_text: will be computed later in actions.rs if needed
        // final_text: text after hot rules (before ITN)
        // Get current model ID for local models
        let current_model_id = self.current_model_id.lock().unwrap().clone();
        Ok(TranscriptionResult {
            original_text: filtered_clone_for_log.clone(),
            itn_text: None, // Will be computed later in actions.rs if needed
            final_text: final_result,
            model_name: current_model_id,
            is_api: false,
        })
    }

    /// Transcribe audio using cloud API
    fn transcribe_with_api(
        &self,
        audio: Vec<f32>,
        settings: &crate::settings::AppSettings,
    ) -> Result<TranscriptionResult> {
        {
            use std::fs::OpenOptions;
            use std::io::Write;
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
            {
                let _ = writeln!(
                    file,
                    r#"{{"sessionId":"debug-session","runId":"run1","hypothesisId":"B","location":"transcription.rs:1626","message":"transcribe_with_api entry","data":{{"selected_config_id":"{:?}","configs_count":{},"audio_len":{}}},"timestamp":{}}}"#,
                    settings.selected_transcription_api_config_id,
                    settings.transcription_api_configs.len(),
                    audio.len(),
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                );
            }
        }

        // Get API config
        let api_config_id = settings
            .selected_transcription_api_config_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No API config selected"))?;

        let api_config = settings
            .transcription_api_configs
            .iter()
            .find(|c| c.id == *api_config_id)
            .ok_or_else(|| anyhow::anyhow!("API config not found: {}", api_config_id))?;

        let provider = settings
            .transcription_api_providers
            .iter()
            .find(|p| p.id == api_config.provider_id)
            .ok_or_else(|| anyhow::anyhow!("API provider not found: {}", api_config.provider_id))?;

        {
            use std::fs::OpenOptions;
            use std::io::Write;
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
            {
                let _ = writeln!(
                    file,
                    r#"{{"sessionId":"debug-session","runId":"run1","hypothesisId":"B","location":"transcription.rs:1635","message":"API config found","data":{{"provider_id":"{}","model":"{}","has_api_key":{}}},"timestamp":{}}}"#,
                    api_config.provider_id,
                    api_config.model,
                    !api_config.api_key.is_empty(),
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                );
            }
        }

        // Convert f32 audio samples to bytes (16-bit PCM)
        let audio_len = audio.len();
        let mut audio_bytes = Vec::with_capacity(audio_len * 2);
        for sample in audio {
            let sample_i16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
            audio_bytes.extend_from_slice(&sample_i16.to_le_bytes());
        }

        {
            use std::fs::OpenOptions;
            use std::io::Write;
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
            {
                let _ = writeln!(
                    file,
                    r#"{{"sessionId":"debug-session","runId":"run1","hypothesisId":"D","location":"transcription.rs:1642","message":"Audio converted to PCM","data":{{"audio_samples":{},"pcm_bytes":{}}},"timestamp":{}}}"#,
                    audio_len,
                    audio_bytes.len(),
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                );
            }
        }

        // Use tokio runtime handle - try to use current handle if we're in an async context
        // Otherwise create a new runtime
        let language = api_config.language.as_deref();
        let custom_api_url = api_config.api_url.clone();
        let provider_clone = provider.clone();
        let api_key = api_config.api_key.clone();
        let model = api_config.model.clone();

        {
            use std::fs::OpenOptions;
            use std::io::Write;
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
            {
                let _ = writeln!(
                    file,
                    r#"{{"sessionId":"debug-session","runId":"run1","hypothesisId":"E","location":"transcription.rs:1654","message":"Calling transcribe_audio","data":{{"provider":"{}","model":"{}","audio_bytes":{}}},"timestamp":{}}}"#,
                    provider.id,
                    model,
                    audio_bytes.len(),
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                );
            }
        }

        // Check if we're in a tokio runtime
        // If we are, we cannot create a new runtime - use a separate thread instead
        // If not, create a new runtime directly
        let text = if tokio::runtime::Handle::try_current().is_ok() {
            // We're already in a tokio runtime, spawn a new thread to avoid nested runtime
            // Create HTTP client in the new thread to avoid builder errors in tokio runtime context
            let provider_clone_thread = provider.clone();
            let api_key_thread = api_key.clone();
            let model_thread = model.clone();
            let audio_bytes_thread = audio_bytes.clone();
            let language_thread = language.map(|s| s.to_string());
            let custom_api_url_thread = custom_api_url.clone();

            // Use channel to receive result from thread
            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                // Create HTTP client in the new thread (outside tokio runtime)
                let client_result = transcription_api_client::create_client_for_provider(
                    &provider_clone_thread,
                    &api_key_thread,
                );

                // Create a new runtime in the separate thread (this is safe)
                let rt_result = tokio::runtime::Runtime::new();

                match (client_result, rt_result) {
                    (Ok(client), Ok(rt)) => {
                        log::info!("[Transcription API] Client and runtime created successfully");
                        let result = rt.block_on(async {
                            transcription_api_client::transcribe_audio_with_client(
                                &client,
                                &provider_clone_thread,
                                &model_thread,
                                audio_bytes_thread,
                                language_thread.as_deref(),
                                custom_api_url_thread,
                            )
                            .await
                        });
                        let _ = tx.send(result);
                    }
                    (Err(e), _) => {
                        let error_msg = format!("Failed to create HTTP client: {}", e);
                        log::error!("[Transcription API] {}", error_msg);
                        let _ = tx.send(Err(error_msg));
                    }
                    (_, Err(e)) => {
                        let error_msg = format!("Failed to create tokio runtime: {}", e);
                        log::error!("[Transcription API] {}", error_msg);
                        let _ = tx.send(Err(error_msg));
                    }
                }
            });

            // Wait for result from thread (blocking wait)
            match rx.recv() {
                Ok(Ok(text)) => {
                    log::info!("[Transcription API] Transcription completed successfully");
                    Ok(text)
                }
                Ok(Err(e)) => {
                    log::error!("[Transcription API] Transcription failed: {}", e);
                    Err(anyhow::anyhow!("API transcription failed: {}", e))
                }
                Err(e) => {
                    log::error!("[Transcription API] Channel receive error: {:?}", e);
                    Err(anyhow::anyhow!(
                        "Failed to receive transcription result: {:?}",
                        e
                    ))
                }
            }?
        } else {
            // Not in a tokio runtime, safe to create a new one
            let runtime = tokio::runtime::Runtime::new()
                .map_err(|e| anyhow::anyhow!("Failed to create tokio runtime: {}", e))?;
            runtime
                .block_on(async {
                    transcription_api_client::transcribe_audio(
                        &provider_clone,
                        api_key,
                        &model,
                        audio_bytes,
                        language,
                        custom_api_url,
                    )
                    .await
                })
                .map_err(|e| anyhow::anyhow!("API transcription failed: {}", e))?
        };

        {
            use std::fs::OpenOptions;
            use std::io::Write;
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
            {
                let _ = writeln!(
                    file,
                    r#"{{"sessionId":"debug-session","runId":"run1","hypothesisId":"C","location":"transcription.rs:1665","message":"API transcription completed","data":{{"text_len":{},"text_preview":"{}"}},"timestamp":{}}}"#,
                    text.len(),
                    if text.chars().count() > 50 { 
                        text.chars().take(50).collect::<String>()
                    } else { 
                        text.clone()
                    },
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                );
            }
        }

        info!("API transcription completed: {}", text);

        // Apply post-processing (custom words, filtering, hot rules)
        let corrected_result = if !settings.custom_words.is_empty() {
            apply_custom_words(
                &text,
                &settings.custom_words,
                settings.word_correction_threshold,
            )
        } else {
            text
        };

        let filtered_result = filter_transcription_output(&corrected_result);
        let filtered_result_clone = filtered_result.clone();

        // Apply hot rules if file exists
        let app_data_dir = self.app_handle.path().app_data_dir().ok();
        let hot_rules_result = if let Some(app_data_dir) = app_data_dir {
            let hot_rule_path = app_data_dir.join("hot-rule.txt");
            let rules = load_hot_rules(&hot_rule_path);
            if !rules.is_empty() {
                apply_hot_rules(&filtered_result, &rules)
            } else {
                filtered_result
            }
        } else {
            filtered_result
        };

        // For API transcription, use the actual model name (e.g., "qwen3-asr-flash")
        // not the config name
        Ok(TranscriptionResult {
            original_text: filtered_result_clone,
            itn_text: None,
            final_text: hot_rules_result,
            model_name: Some(api_config.model.clone()),
            is_api: true,
        })
    }
}

impl Drop for TranscriptionManager {
    fn drop(&mut self) {
        debug!("Shutting down TranscriptionManager");

        // Signal the watcher thread to shutdown
        self.shutdown_signal.store(true, Ordering::Relaxed);

        // Wait for the thread to finish gracefully
        if let Some(handle) = self.watcher_handle.lock().unwrap().take() {
            if let Err(e) = handle.join() {
                warn!("Failed to join idle watcher thread: {:?}", e);
            } else {
                debug!("Idle watcher thread joined successfully");
            }
        }
    }
}
