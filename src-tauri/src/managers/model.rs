use crate::settings::{get_settings, write_settings};
use anyhow::Result;
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use futures_util::StreamExt;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tar::Archive;
use tauri::{AppHandle, Emitter, Manager};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum EngineType {
    Whisper,
    Paraformer,  // Includes standard Paraformer and SeACo (model_eb.onnx) at load time
    Transducer,  // Transducer models support hotwords (e.g., zipformer)
    FireRedAsr,  // FireRedAsr models (Chinese + English, supports dialects)
    Api,         // Cloud API-based transcription
}

/// How the model entered the list: from config (download) or from folder import/scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "snake_case")]
pub enum ModelSource {
    /// From models.toml / default_models.toml (has url for download).
    #[default]
    Download,
    /// From import_local_model_folder or scan_for_custom_models.
    LocalImport,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub filename: String,
    pub url: Option<String>,
    pub size_mb: u64,
    pub is_downloaded: bool,
    pub is_downloading: bool,
    pub partial_size: u64,
    pub is_directory: bool,
    pub engine_type: EngineType,
    pub accuracy_score: f32, // 0.0 to 1.0, higher is more accurate
    pub speed_score: f32,   // 0.0 to 1.0, higher is faster
    #[serde(default)]
    pub source: ModelSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DownloadProgress {
    pub model_id: String,
    pub downloaded: u64,
    pub total: u64,
    pub percentage: f64,
}

/// One entry in models.toml [[local_models]] (for deserialization only).
#[derive(Debug, Clone, Deserialize)]
pub struct ModelEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "engine_type")]
    pub engine_type_str: String,
    pub filename: String,
    pub url: Option<String>,
    pub size_mb: u64,
    pub is_directory: bool,
    pub accuracy_score: f32,
    pub speed_score: f32,
}

/// Root of models.toml / default_models.toml.
#[derive(Debug, Deserialize)]
pub struct ModelsConfig {
    pub local_models: Vec<ModelEntry>,
}

fn parse_engine_type(s: &str) -> EngineType {
    match s.to_lowercase().as_str() {
        "whisper" => EngineType::Whisper,
        "paraformer" | "seacoparaformer" => EngineType::Paraformer,
        "transducer" => EngineType::Transducer,
        "fireredasr" => EngineType::FireRedAsr,
        "api" => EngineType::Api,
        _ => EngineType::Paraformer,
    }
}

impl ModelEntry {
    fn to_model_info(&self) -> ModelInfo {
        ModelInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            filename: self.filename.clone(),
            url: self.url.clone(),
            size_mb: self.size_mb,
            is_downloaded: false,
            is_downloading: false,
            partial_size: 0,
            is_directory: self.is_directory,
            engine_type: parse_engine_type(&self.engine_type_str),
            accuracy_score: self.accuracy_score,
            speed_score: self.speed_score,
            source: ModelSource::Download,
        }
    }
}

/// Load model list from user models.toml or bundled default_models.toml. No in-code default list.
fn load_models_config(app_handle: &AppHandle) -> Result<HashMap<String, ModelInfo>> {
    let app_data = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| anyhow::anyhow!("Failed to get app data dir: {}", e))?;
    let user_config_path = app_data.join("models.toml");

    let config: ModelsConfig = if user_config_path.exists() {
        match (|| {
            let s = fs::read_to_string(&user_config_path)
                .map_err(|e| anyhow::anyhow!("Failed to read models.toml: {}", e))?;
            toml::from_str::<ModelsConfig>(&s).map_err(|e| anyhow::anyhow!("Failed to parse models.toml: {}", e))
        })() {
            Ok(c) => c,
            Err(e) => {
                warn!("models.toml missing or invalid ({}), using default_models.toml", e);
                let default_path = app_handle.path().resolve(
                    "resources/default_models.toml",
                    tauri::path::BaseDirectory::Resource,
                ).map_err(|e| anyhow::anyhow!("Failed to get resource path: {}", e))?;
                if !default_path.exists() {
                    return Ok(HashMap::new());
                }
                let s = fs::read_to_string(&default_path)
                    .map_err(|e| anyhow::anyhow!("Failed to read default_models.toml: {}", e))?;
                toml::from_str(&s).map_err(|e| anyhow::anyhow!("Failed to parse default_models.toml: {}", e))?
            }
        }
    } else {
        let default_path = app_handle.path().resolve(
            "resources/default_models.toml",
            tauri::path::BaseDirectory::Resource,
        ).map_err(|e| anyhow::anyhow!("Failed to get resource path: {}", e))?;
        if !default_path.exists() {
            return Ok(HashMap::new());
        }
        let s = fs::read_to_string(&default_path)
            .map_err(|e| anyhow::anyhow!("Failed to read default_models.toml: {}", e))?;
        toml::from_str(&s).map_err(|e| anyhow::anyhow!("Failed to parse default_models.toml: {}", e))?
    };

    let mut map = HashMap::new();
    for entry in config.local_models {
        let info = entry.to_model_info();
        map.insert(info.id.clone(), info);
    }
    Ok(map)
}

pub struct ModelManager {
    app_handle: AppHandle,
    models_dir: PathBuf,
    available_models: Mutex<HashMap<String, ModelInfo>>,
    download_cancellations: Arc<Mutex<HashMap<String, CancellationToken>>>,
}

impl ModelManager {
    /// Helper function to find transducer model files in a directory
    /// Returns (encoder_path, decoder_path, joiner_path) if found
    fn find_transducer_files(model_dir: &PathBuf) -> Option<(PathBuf, PathBuf, PathBuf)> {
        // Try to find encoder, decoder, and joiner files with various naming patterns
        let entries = match fs::read_dir(model_dir) {
            Ok(entries) => entries,
            Err(_) => return None,
        };

        let mut encoder_file: Option<PathBuf> = None;
        let mut decoder_file: Option<PathBuf> = None;
        let mut joiner_file: Option<PathBuf> = None;

        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(filename) = path.file_name() {
                let filename_str = filename.to_string_lossy();

                // Match encoder files (prefer non-int8 versions)
                if filename_str.starts_with("encoder") && filename_str.ends_with(".onnx") {
                    if !filename_str.contains(".int8.") {
                        encoder_file = Some(path.clone());
                    } else if encoder_file.is_none() {
                        encoder_file = Some(path.clone());
                    }
                }

                // Match decoder files
                if filename_str.starts_with("decoder") && filename_str.ends_with(".onnx") {
                    decoder_file = Some(path.clone());
                }

                // Match joiner files (prefer non-int8 versions)
                if filename_str.starts_with("joiner") && filename_str.ends_with(".onnx") {
                    if !filename_str.contains(".int8.") {
                        joiner_file = Some(path.clone());
                    } else if joiner_file.is_none() {
                        joiner_file = Some(path.clone());
                    }
                }
            }
        }

        // Return only if all three files are found
        match (encoder_file, decoder_file, joiner_file) {
            (Some(e), Some(d), Some(j)) => Some((e, d, j)),
            _ => None,
        }
    }

    pub fn new(app_handle: &AppHandle) -> Result<Self> {
        // Create models directory in app data
        let models_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| anyhow::anyhow!("Failed to get app data dir: {}", e))?
            .join("models");

        if !models_dir.exists() {
            fs::create_dir_all(&models_dir)?;
        }

        let available_models = load_models_config(app_handle)?;

        let manager = Self {
            app_handle: app_handle.clone(),
            models_dir,
            available_models: Mutex::new(available_models),
            download_cancellations: Arc::new(Mutex::new(HashMap::new())),
        };

        // Remove any unsupported models from available_models (parakeet, moonshine, etc.)
        {
            let mut models = manager.available_models.lock().unwrap();
            let keys_to_remove: Vec<String> = models
                .iter()
                .filter(|(_, model)| {
                    !matches!(
                        model.engine_type,
                        EngineType::Whisper
                            | EngineType::Paraformer
                            | EngineType::Transducer
                            | EngineType::FireRedAsr
                    )
                })
                .map(|(id, _)| id.clone())
                .collect();
            for key in keys_to_remove {
                models.remove(&key);
                info!("Removed unsupported model: {}", key);
            }
        }

        // Migrate any bundled models to user directory
        manager.migrate_bundled_models()?;

        // Scan for custom models in the models directory
        manager.scan_for_custom_models()?;

        // Check which models are already downloaded
        manager.update_download_status()?;

        // Auto-select a model if none is currently selected
        manager.auto_select_model_if_needed()?;

        Ok(manager)
    }

    /// Reload model list from models.toml (or default_models.toml). Keeps custom-* and scan results, replaces config-sourced entries.
    pub fn reload_models_config(&self) -> Result<()> {
        let config_models = load_models_config(&self.app_handle)?;
        {
            let mut models = self.available_models.lock().unwrap();
            let keys_to_remove: Vec<String> = models
                .keys()
                .filter(|id| !id.starts_with("custom-"))
                .cloned()
                .collect();
            for id in keys_to_remove {
                models.remove(&id);
            }
            for (id, info) in config_models {
                if matches!(
                    info.engine_type,
                    EngineType::Whisper
                        | EngineType::Paraformer
                        | EngineType::Transducer
                        | EngineType::FireRedAsr
                ) {
                    models.insert(id, info);
                }
            }
        }
        self.scan_for_custom_models()?;
        self.update_download_status()?;
        Ok(())
    }

    pub fn get_available_models(&self) -> Vec<ModelInfo> {
        let models = self.available_models.lock().unwrap();
        // Filter out any unsupported models (parakeet, moonshine, etc.)
        let filtered: Vec<ModelInfo> = models
            .values()
            .filter(|model| {
                matches!(
                    model.engine_type,
                    EngineType::Whisper
                        | EngineType::Paraformer
                        | EngineType::Transducer
                        | EngineType::FireRedAsr
                )
            })
            .cloned()
            .collect();

        // Debug: log all available models (only at debug level to avoid spam)
        debug!("Available models count: {}", filtered.len());
        for model in &filtered {
            debug!(
                "Model: {} (id: {}, engine_type: {:?}, is_downloaded: {})",
                model.name, model.id, model.engine_type, model.is_downloaded
            );
        }

        filtered
    }

    pub fn get_model_info(&self, model_id: &str) -> Option<ModelInfo> {
        let models = self.available_models.lock().unwrap();
        models.get(model_id).and_then(|model| {
            // Only return supported models (Whisper, Paraformer, Transducer, FireRedAsr)
            if matches!(
                model.engine_type,
                EngineType::Whisper
                    | EngineType::Paraformer
                    | EngineType::Transducer
                    | EngineType::FireRedAsr
            ) {
                Some(model.clone())
            } else {
                None
            }
        })
    }

    fn migrate_bundled_models(&self) -> Result<()> {
        // Check for bundled models and copy them to user directory
        let bundled_models = ["ggml-small.bin"]; // Add other bundled models here if any

        for filename in &bundled_models {
            let bundled_path = self.app_handle.path().resolve(
                &format!("resources/models/{}", filename),
                tauri::path::BaseDirectory::Resource,
            );

            if let Ok(bundled_path) = bundled_path {
                if bundled_path.exists() {
                    let user_path = self.models_dir.join(filename);

                    // Only copy if user doesn't already have the model
                    if !user_path.exists() {
                        info!("Migrating bundled model {} to user directory", filename);
                        fs::copy(&bundled_path, &user_path)?;
                        info!("Successfully migrated {}", filename);
                    }
                }
            }
        }

        Ok(())
    }

    fn scan_for_custom_models(&self) -> Result<()> {
        // Scan models directory for any unrecognized model folders
        let entries = match fs::read_dir(&self.models_dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(()), // Directory doesn't exist yet
        };

        let mut models = self.available_models.lock().unwrap();

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let folder_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name,
                None => continue,
            };

            // Skip if already registered
            let model_id = format!("custom-{}", folder_name.replace(" ", "-").to_lowercase());
            if models.contains_key(&model_id) {
                continue;
            }

            // Skip unstable models that are known to crash
            // Note: seaco_paraformer.20250904.for_general.sherpa_onnx is now supported via import_local_model_folder
            // Only block the old problematic models
            let blocked_models = [
                "seaco-paraformer贝壳",
                "paraformer-流式双语",
            ];
            let folder_name_lower = folder_name.to_lowercase();
            if blocked_models.iter().any(|blocked| {
                folder_name_lower.contains(&blocked.to_lowercase())
                    || model_id.contains(&blocked.to_lowercase())
            }) {
                info!("Skipping blocked model: {} (known to be unstable)", folder_name);
                continue;
            }

            // Detect model type
            let has_model_eb = path.join("model_eb.onnx").exists();
            let has_model_onnx = path.join("model.onnx").exists();
            let has_tokens = path.join("tokens.txt").exists() || path.join("tokens.json").exists();

            let engine_type = if has_model_onnx && has_tokens {
                // Paraformer (with optional model_eb.onnx for hotwords)
                Some(EngineType::Paraformer)
            } else {
                None
            };

            if let Some(engine_type) = engine_type {
                // Calculate size
                let size_mb = Self::calculate_dir_size(&path).unwrap_or(0) / (1024 * 1024);

                let model_name = if has_model_eb {
                    "KeSeACoParaformer".to_string()
                } else {
                    folder_name.to_string()
                };
                let description = if has_model_eb {
                    "自定义 SeACo Paraformer 模型（已自动识别）".to_string()
                } else {
                    "自定义 Paraformer 模型（已自动识别）".to_string()
                };

                let model_info = ModelInfo {
                    id: model_id.clone(),
                    name: model_name,
                    description,
                    filename: folder_name.to_string(),
                    url: None,
                    size_mb,
                    is_downloaded: true,
                    is_downloading: false,
                    partial_size: 0,
                    is_directory: true,
                    engine_type,
                    accuracy_score: 0.85,
                    speed_score: 0.75,
                    source: ModelSource::LocalImport,
                };

                info!("Auto-discovered custom model: {} at {:?}", model_id, path);
                models.insert(model_id, model_info);
            }
        }

        Ok(())
    }

    fn update_download_status(&self) -> Result<()> {
        let mut models = self.available_models.lock().unwrap();

        for model in models.values_mut() {
            if model.is_directory {
                // For directory-based models, check if the directory exists
                let model_path = self.models_dir.join(&model.filename);
                let partial_path = self.models_dir.join(format!("{}.partial", &model.filename));
                let extracting_path = self
                    .models_dir
                    .join(format!("{}.extracting", &model.filename));

                // Clean up any leftover .extracting directories from interrupted extractions
                if extracting_path.exists() {
                    warn!("Cleaning up interrupted extraction for model: {}", model.id);
                    let _ = fs::remove_dir_all(&extracting_path);
                }

                // For directory-based models, check if directory exists
                // Paraformer models from tar.bz2 archives should have model.onnx and tokens.txt
                model.is_downloaded = model_path.exists() && model_path.is_dir();

                // Additional check for Paraformer (includes SeACo): model.onnx or model.int8.onnx, tokens.txt or tokens.json
                if matches!(model.engine_type, EngineType::Paraformer) {
                    let model_ok = model_path.join("model.onnx").exists()
                        || model_path.join("model.int8.onnx").exists();
                    let tokens_ok = model_path.join("tokens.txt").exists()
                        || model_path.join("tokens.json").exists();
                    model.is_downloaded = model.is_downloaded && model_ok && tokens_ok;
                }

                // Additional check for FireRedAsr: verify encoder, decoder, and tokens.txt exist
                if matches!(model.engine_type, EngineType::FireRedAsr) {
                    let encoder_file = model_path.join("encoder.int8.onnx");
                    let decoder_file = model_path.join("decoder.int8.onnx");
                    let tokens_file = model_path.join("tokens.txt");

                    model.is_downloaded = model.is_downloaded
                        && (encoder_file.exists() || model_path.join("encoder.onnx").exists())
                        && (decoder_file.exists() || model_path.join("decoder.onnx").exists())
                        && tokens_file.exists();
                }

                // Additional check for Transducer: verify encoder, decoder, joiner, and tokens.txt exist
                if matches!(model.engine_type, EngineType::Transducer) {
                    let tokens_file = model_path.join("tokens.txt");
                    let bbpe_model_file = model_path.join("bbpe.model");

                    // Use helper function to find transducer files (supports various naming patterns)
                    let files_found = Self::find_transducer_files(&model_path).is_some();

                    // conformer-zh-stateless2 uses cjkchar modeling unit and doesn't need bbpe.model
                    let needs_bbpe = model.id != "conformer-zh-stateless2";

                    model.is_downloaded = model.is_downloaded
                        && files_found
                        && tokens_file.exists()
                        && (!needs_bbpe || bbpe_model_file.exists());
                }
                model.is_downloading = false;

                // Get partial file size if it exists (for the .tar.gz being downloaded)
                if partial_path.exists() {
                    model.partial_size = partial_path.metadata().map(|m| m.len()).unwrap_or(0);
                } else {
                    model.partial_size = 0;
                }
            } else {
                // For file-based models (existing logic)
                let model_path = self.models_dir.join(&model.filename);
                let partial_path = self.models_dir.join(format!("{}.partial", &model.filename));

                model.is_downloaded = model_path.exists();
                model.is_downloading = false;

                // Get partial file size if it exists
                if partial_path.exists() {
                    model.partial_size = partial_path.metadata().map(|m| m.len()).unwrap_or(0);
                } else {
                    model.partial_size = 0;
                }
            }
        }

        Ok(())
    }

    fn auto_select_model_if_needed(&self) -> Result<()> {
        // Check if we have a selected model in settings
        let settings = get_settings(&self.app_handle);

        // Validate that the selected model is still valid (Whisper, Paraformer, Transducer, FireRedAsr)
        let models = self.available_models.lock().unwrap();
        let selected_model_valid = if settings.selected_model.is_empty() {
            false
        } else {
            models
                .get(&settings.selected_model)
                .map(|model| {
                    matches!(
                        model.engine_type,
                        EngineType::Whisper
                            | EngineType::Paraformer
                            | EngineType::Transducer
                            | EngineType::FireRedAsr
                    )
                })
                .unwrap_or(false)
        };

        // If no model is selected, selected model is empty, or selected model is invalid
        if !selected_model_valid {
            // Find the first available (downloaded) supported model
            if let Some(available_model) = models.values().find(|model| {
                model.is_downloaded
                    && matches!(
                        model.engine_type,
                        EngineType::Whisper
                            | EngineType::Paraformer
                            | EngineType::Transducer
                            | EngineType::FireRedAsr
                    )
            }) {
                info!(
                    "Auto-selecting model: {} ({})",
                    available_model.id, available_model.name
                );

                // Update settings with the selected model
                let mut updated_settings = get_settings(&self.app_handle);
                updated_settings.selected_model = available_model.id.clone();
                write_settings(&self.app_handle, updated_settings);

                info!("Successfully auto-selected model: {}", available_model.id);
            }
        }

        Ok(())
    }

    pub async fn download_model(&self, model_id: &str) -> Result<()> {
        // Create cancellation token for this download
        let cancellation_token = CancellationToken::new();
        {
            let mut cancellations = self.download_cancellations.lock().unwrap();
            cancellations.insert(model_id.to_string(), cancellation_token.clone());
        }

        let model_info = {
            let models = self.available_models.lock().unwrap();
            models.get(model_id).cloned()
        };

        let model_info =
            model_info.ok_or_else(|| anyhow::anyhow!("Model not found: {}", model_id))?;

        // Clone necessary fields for error handling
        let app_handle = self.app_handle.clone();
        let download_cancellations = self.download_cancellations.clone();

        // Clone URL before using it, as we need it later for format detection
        let url = model_info
            .url
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No download URL for model"))?
            .clone();
        let model_path = self.models_dir.join(&model_info.filename);
        let partial_path = self
            .models_dir
            .join(format!("{}.partial", &model_info.filename));

        // Don't download if complete version already exists
        if model_path.exists() {
            // Clean up any partial file that might exist
            if partial_path.exists() {
                let _ = fs::remove_file(&partial_path);
            }
            self.update_download_status()?;
            return Ok(());
        }

        // Check if we have a partial download to resume
        let mut resume_from = if partial_path.exists() {
            let size = partial_path.metadata()?.len();
            info!("Resuming download of model {} from byte {}", model_id, size);
            size
        } else {
            info!("Starting fresh download of model {} from {}", model_id, url);
            0
        };

        // Mark as downloading
        {
            let mut models = self.available_models.lock().unwrap();
            if let Some(model) = models.get_mut(model_id) {
                model.is_downloading = true;
            }
        }

        // Create HTTP client with range request for resuming
        let client = reqwest::Client::new();
        let mut request = client.get(&url);

        if resume_from > 0 {
            request = request.header("Range", format!("bytes={}-", resume_from));
        }

        let mut response = request.send().await?;

        // If we tried to resume but server returned 200 (not 206 Partial Content),
        // the server doesn't support range requests. Delete partial file and restart
        // fresh to avoid file corruption (appending full file to partial).
        if resume_from > 0 && response.status() == reqwest::StatusCode::OK {
            warn!(
                "Server doesn't support range requests for model {}, restarting download",
                model_id
            );
            drop(response);
            let _ = fs::remove_file(&partial_path);

            // Reset resume_from since we're starting fresh
            resume_from = 0;

            // Restart download without range header
            response = client.get(&url).send().await?;
        }

        // Check for success or partial content status
        if !response.status().is_success()
            && response.status() != reqwest::StatusCode::PARTIAL_CONTENT
        {
            // Mark as not downloading on error
            {
                let mut models = self.available_models.lock().unwrap();
                if let Some(model) = models.get_mut(model_id) {
                    model.is_downloading = false;
                }
            }
            // Remove cancellation token
            {
                let mut cancellations = download_cancellations.lock().unwrap();
                cancellations.remove(model_id);
            }
            let error_msg = format!("Failed to download model: HTTP {}", response.status());
            // Emit failure event
            let _ = app_handle.emit(
                "model-download-failed",
                serde_json::json!({
                    "model_id": model_id,
                    "error": error_msg,
                }),
            );
            return Err(anyhow::anyhow!("{}", error_msg));
        }

        let total_size = if resume_from > 0 {
            // For resumed downloads, add the resume point to content length
            resume_from + response.content_length().unwrap_or(0)
        } else {
            response.content_length().unwrap_or(0)
        };

        let mut downloaded = resume_from;
        let mut stream = response.bytes_stream();

        // Open file for appending if resuming, or create new if starting fresh
        let mut file = if resume_from > 0 {
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&partial_path)?
        } else {
            std::fs::File::create(&partial_path)?
        };

        // Emit initial progress
        let initial_progress = DownloadProgress {
            model_id: model_id.to_string(),
            downloaded,
            total: total_size,
            percentage: if total_size > 0 {
                (downloaded as f64 / total_size as f64) * 100.0
            } else {
                0.0
            },
        };
        let _ = self
            .app_handle
            .emit("model-download-progress", &initial_progress);

        // Download with progress
        while let Some(chunk_result) = stream.next().await {
            // Check if download was cancelled
            if cancellation_token.is_cancelled() {
                // Mark as not downloading
                {
                    let mut models = self.available_models.lock().unwrap();
                    if let Some(model) = models.get_mut(model_id) {
                        model.is_downloading = false;
                    }
                }
                // Remove cancellation token
                {
                    let mut cancellations = self.download_cancellations.lock().unwrap();
                    cancellations.remove(model_id);
                }
                // Emit cancellation event
                let _ = self.app_handle.emit("model-download-cancelled", model_id);
                return Err(anyhow::anyhow!("Download cancelled by user"));
            }

            let chunk = chunk_result.map_err(|e| {
                // Mark as not downloading on error
                {
                    let mut models = self.available_models.lock().unwrap();
                    if let Some(model) = models.get_mut(model_id) {
                        model.is_downloading = false;
                    }
                }
                // Remove cancellation token on error
                {
                    let mut cancellations = download_cancellations.lock().unwrap();
                    cancellations.remove(model_id);
                }
                let error_msg = format!("Network error during download: {}", e);
                // Emit failure event
                let _ = app_handle.emit(
                    "model-download-failed",
                    serde_json::json!({
                        "model_id": model_id,
                        "error": error_msg,
                    }),
                );
                e
            })?;

            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;

            let percentage = if total_size > 0 {
                (downloaded as f64 / total_size as f64) * 100.0
            } else {
                0.0
            };

            // Emit progress event
            let progress = DownloadProgress {
                model_id: model_id.to_string(),
                downloaded,
                total: total_size,
                percentage,
            };

            let _ = self.app_handle.emit("model-download-progress", &progress);
        }

        // Remove cancellation token on successful completion
        {
            let mut cancellations = self.download_cancellations.lock().unwrap();
            cancellations.remove(model_id);
        }

        file.flush()?;
        drop(file); // Ensure file is closed before moving

        // Verify downloaded file size matches expected size
        if total_size > 0 {
            let actual_size = partial_path.metadata()?.len();
            if actual_size != total_size {
                // Download is incomplete/corrupted - delete partial and return error
                let _ = fs::remove_file(&partial_path);
                {
                    let mut models = self.available_models.lock().unwrap();
                    if let Some(model) = models.get_mut(model_id) {
                        model.is_downloading = false;
                    }
                }
                // Remove cancellation token
                {
                    let mut cancellations = download_cancellations.lock().unwrap();
                    cancellations.remove(model_id);
                }
                let error_msg = format!(
                    "Download incomplete: expected {} bytes, got {} bytes",
                    total_size, actual_size
                );
                // Emit failure event
                let _ = app_handle.emit(
                    "model-download-failed",
                    serde_json::json!({
                        "model_id": model_id,
                        "error": error_msg,
                    }),
                );
                return Err(anyhow::anyhow!("{}", error_msg));
            }
        }

        // Handle directory-based models (extract tar.gz or tar.bz2) vs file-based models
        if model_info.is_directory {
            // Emit extraction started event
            let _ = self.app_handle.emit("model-extraction-started", model_id);
            info!("Extracting archive for directory-based model: {}", model_id);

            // Use a temporary extraction directory to ensure atomic operations
            let temp_extract_dir = self
                .models_dir
                .join(format!("{}.extracting", &model_info.filename));
            let final_model_dir = self.models_dir.join(&model_info.filename);

            // Clean up any previous incomplete extraction
            if temp_extract_dir.exists() {
                let _ = fs::remove_dir_all(&temp_extract_dir);
            }

            // Create temporary extraction directory
            fs::create_dir_all(&temp_extract_dir)?;

            // Determine archive format based on URL extension
            let is_bz2 = url.ends_with(".tar.bz2") || url.ends_with(".tbz2");

            // Extract based on archive format
            if is_bz2 {
                // Handle tar.bz2 files
                let archive_file = File::open(&partial_path)?;
                let bz2_decoder = BzDecoder::new(BufReader::new(archive_file));
                let mut archive = Archive::new(bz2_decoder);

                archive.unpack(&temp_extract_dir).map_err(|e| {
                    let error_msg = format!("Failed to extract tar.bz2 archive: {}", e);
                    // Clean up failed extraction
                    let _ = fs::remove_dir_all(&temp_extract_dir);
                    let _ = self.app_handle.emit(
                        "model-extraction-failed",
                        &serde_json::json!({
                            "model_id": model_id,
                            "error": error_msg
                        }),
                    );
                    anyhow::anyhow!(error_msg)
                })?;
            } else {
                // Handle tar.gz files (default)
                let archive_file = File::open(&partial_path)?;
                let gz_decoder = GzDecoder::new(archive_file);
                let mut archive = Archive::new(gz_decoder);

                archive.unpack(&temp_extract_dir).map_err(|e| {
                    let error_msg = format!("Failed to extract tar.gz archive: {}", e);
                    // Clean up failed extraction
                    let _ = fs::remove_dir_all(&temp_extract_dir);
                    let _ = self.app_handle.emit(
                        "model-extraction-failed",
                        &serde_json::json!({
                            "model_id": model_id,
                            "error": error_msg
                        }),
                    );
                    anyhow::anyhow!(error_msg)
                })?;
            }

            // Find the actual extracted directory (archive might have a nested structure)
            let extracted_dirs: Vec<_> = fs::read_dir(&temp_extract_dir)?
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
                .collect();

            if extracted_dirs.len() == 1 {
                // Single directory extracted, move it to the final location
                let source_dir = extracted_dirs[0].path();
                if final_model_dir.exists() {
                    fs::remove_dir_all(&final_model_dir)?;
                }
                fs::rename(&source_dir, &final_model_dir)?;
                // Clean up temp directory
                let _ = fs::remove_dir_all(&temp_extract_dir);
            } else {
                // Multiple items or no directories, rename the temp directory itself
                if final_model_dir.exists() {
                    fs::remove_dir_all(&final_model_dir)?;
                }
                fs::rename(&temp_extract_dir, &final_model_dir)?;
            }

            info!("Successfully extracted archive for model: {}", model_id);
            // Emit extraction completed event
            let _ = self.app_handle.emit("model-extraction-completed", model_id);

            // Remove the downloaded archive file
            let _ = fs::remove_file(&partial_path);
        } else {
            // Move partial file to final location for file-based models
            fs::rename(&partial_path, &model_path)?;
        }

        // Update download status
        {
            let mut models = self.available_models.lock().unwrap();
            if let Some(model) = models.get_mut(model_id) {
                model.is_downloading = false;
                model.is_downloaded = true;
                model.partial_size = 0;
            }
        }

        // Remove cancellation token on completion
        {
            let mut cancellations = self.download_cancellations.lock().unwrap();
            cancellations.remove(model_id);
        }

        // Remove cancellation token on completion
        {
            let mut cancellations = self.download_cancellations.lock().unwrap();
            cancellations.remove(model_id);
        }

        // Emit completion event
        let _ = self.app_handle.emit("model-download-complete", model_id);

        info!(
            "Successfully downloaded model {} to {:?}",
            model_id, model_path
        );

        Ok(())
    }

    /// Import a local model folder
    /// Copies the folder to models directory and registers it as a custom model
    pub fn import_local_model_folder(&self, source_path: PathBuf) -> Result<String> {
        use std::path::Path;

        // Validate source path exists and is a directory
        if !source_path.exists() {
            return Err(anyhow::anyhow!(
                "Source path does not exist: {:?}",
                source_path
            ));
        }
        if !source_path.is_dir() {
            return Err(anyhow::anyhow!(
                "Source path is not a directory: {:?}",
                source_path
            ));
        }

        // Get folder name as model name
        let folder_name = source_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid folder name"))?;

        // Generate model ID from folder name (sanitize for use as ID)
        let model_id = format!("custom-{}", folder_name.replace(" ", "-").to_lowercase());

        // Check if model already exists - if so, just register it
        let target_path = self.models_dir.join(folder_name);
        let needs_copy = !target_path.exists();

        // Detect model type by checking for model_eb.onnx (SeACo Paraformer)
        let has_model_eb = source_path.join("model_eb.onnx").exists();
        let has_model_onnx = source_path.join("model.onnx").exists();
        let has_tokens =
            source_path.join("tokens.txt").exists() || source_path.join("tokens.json").exists();

        let engine_type = if has_model_onnx && has_tokens {
            EngineType::Paraformer
        } else {
            return Err(anyhow::anyhow!(
                "Unsupported model format. Required files: model.onnx and tokens.txt (or tokens.json). For SeACo Paraformer, also requires model_eb.onnx"
            ));
        };

        // Copy folder to models directory if needed
        if needs_copy {
            info!(
                "Importing model from {:?} to {:?}",
                source_path, target_path
            );
            Self::copy_dir_all(&source_path, &target_path)?;
        } else {
            info!(
                "Model folder already exists at {:?}, registering it",
                target_path
            );
        }

        // Register model in available_models
        let mut models = self.available_models.lock().unwrap();

        // Calculate approximate size
        let size_mb = Self::calculate_dir_size(&target_path)? / (1024 * 1024);

        let model_name = if has_model_eb {
            "KeSeACoParaformer".to_string()
        } else {
            folder_name.to_string()
        };
        let description = if has_model_eb {
            "自定义 SeACo Paraformer 模型，支持热词功能。".to_string()
        } else {
            "自定义 Paraformer 模型。".to_string()
        };

        let model_info = ModelInfo {
            id: model_id.clone(),
            name: model_name,
            description,
            filename: folder_name.to_string(),
            url: None,
            size_mb,
            is_downloaded: true,
            is_downloading: false,
            partial_size: 0,
            is_directory: true,
            engine_type,
            accuracy_score: 0.85, // Default scores
            speed_score: 0.75,
            source: ModelSource::LocalImport,
        };

        models.insert(model_id.clone(), model_info);
        info!("Successfully imported model: {}", model_id);

        // Update download status
        drop(models);
        self.update_download_status()?;

        // Emit event to notify frontend that a model was imported
        let _ = self.app_handle.emit("model-imported", model_id.clone());

        Ok(model_id)
    }

    /// Helper function to copy directory recursively
    fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();
            let dst_path = dst.join(&file_name);

            if path.is_dir() {
                Self::copy_dir_all(&path, &dst_path)?;
            } else {
                fs::copy(&path, &dst_path)?;
            }
        }
        Ok(())
    }

    /// Calculate total size of directory recursively
    fn calculate_dir_size(path: &PathBuf) -> Result<u64> {
        let mut total_size = 0u64;
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    total_size += Self::calculate_dir_size(&entry_path)?;
                } else {
                    total_size += entry_path.metadata()?.len();
                }
            }
        } else {
            total_size = path.metadata()?.len();
        }
        Ok(total_size)
    }

    pub fn delete_model(&self, model_id: &str) -> Result<()> {
        debug!("ModelManager: delete_model called for: {}", model_id);

        let model_info = {
            let models = self.available_models.lock().unwrap();
            models.get(model_id).cloned()
        };

        let model_info =
            model_info.ok_or_else(|| anyhow::anyhow!("Model not found: {}", model_id))?;

        debug!("ModelManager: Found model info: {:?}", model_info);

        let model_path = self.models_dir.join(&model_info.filename);
        let partial_path = self
            .models_dir
            .join(format!("{}.partial", &model_info.filename));
        debug!("ModelManager: Model path: {:?}", model_path);
        debug!("ModelManager: Partial path: {:?}", partial_path);

        let mut deleted_something = false;

        if model_info.is_directory {
            // Delete complete model directory if it exists
            if model_path.exists() && model_path.is_dir() {
                info!("Deleting model directory at: {:?}", model_path);
                fs::remove_dir_all(&model_path)?;
                info!("Model directory deleted successfully");
                deleted_something = true;
            }
        } else {
            // Delete complete model file if it exists
            if model_path.exists() {
                info!("Deleting model file at: {:?}", model_path);
                fs::remove_file(&model_path)?;
                info!("Model file deleted successfully");
                deleted_something = true;
            }
        }

        // Delete partial file if it exists (same for both types)
        if partial_path.exists() {
            info!("Deleting partial file at: {:?}", partial_path);
            fs::remove_file(&partial_path)?;
            info!("Partial file deleted successfully");
            deleted_something = true;
        }

        if !deleted_something {
            return Err(anyhow::anyhow!("No model files found to delete"));
        }

        // 文件夹导入的本地模型（custom-*）删除后从列表移除，卡片消失；内置模型只更新下载状态
        if model_id.starts_with("custom-") {
            let mut models = self.available_models.lock().unwrap();
            models.remove(model_id);
            info!("Custom model {} removed from list, card will disappear", model_id);
        } else {
            info!("Model {} files deleted, updating download status", model_id);
            self.update_download_status()?;
            debug!("ModelManager: download status updated");
        }

        Ok(())
    }

    pub fn get_model_path(&self, model_id: &str) -> Result<PathBuf> {
        let model_info = self
            .get_model_info(model_id)
            .ok_or_else(|| anyhow::anyhow!("Model not found: {}", model_id))?;

        if !model_info.is_downloaded {
            return Err(anyhow::anyhow!("Model not available: {}", model_id));
        }

        // Ensure we don't return partial files/directories
        if model_info.is_downloading {
            return Err(anyhow::anyhow!(
                "Model is currently downloading: {}",
                model_id
            ));
        }

        let model_path = self.models_dir.join(&model_info.filename);
        let partial_path = self
            .models_dir
            .join(format!("{}.partial", &model_info.filename));

        if model_info.is_directory {
            // For directory-based models, ensure the directory exists and is complete
            if model_path.exists() && model_path.is_dir() && !partial_path.exists() {
                Ok(model_path)
            } else {
                Err(anyhow::anyhow!(
                    "Complete model directory not found: {}",
                    model_id
                ))
            }
        } else {
            // For file-based models (existing logic)
            if model_path.exists() && !partial_path.exists() {
                Ok(model_path)
            } else {
                Err(anyhow::anyhow!(
                    "Complete model file not found: {}",
                    model_id
                ))
            }
        }
    }

    pub fn cancel_download(&self, model_id: &str) -> Result<()> {
        debug!("ModelManager: cancel_download called for: {}", model_id);

        // Cancel the download by triggering the cancellation token
        let cancellation_token = {
            let mut cancellations = self.download_cancellations.lock().unwrap();
            cancellations.remove(model_id)
        };

        if let Some(token) = cancellation_token {
            token.cancel();
            info!("Download cancellation token triggered for: {}", model_id);
        } else {
            warn!("No active download found to cancel for: {}", model_id);
        }

        // Mark as not downloading
        {
            let mut models = self.available_models.lock().unwrap();
            if let Some(model) = models.get_mut(model_id) {
                model.is_downloading = false;
            }
        }

        // Update download status to reflect current state
        self.update_download_status()?;

        info!("Download cancelled for: {}", model_id);
        Ok(())
    }
}
