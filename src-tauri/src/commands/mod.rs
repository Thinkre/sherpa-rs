pub mod audio;
pub mod history;
pub mod hotwords;
pub mod models;
pub mod transcription;

use crate::managers::punctuation::PunctuationManager;
use crate::settings::{get_settings, write_settings, AppSettings, LogLevel};
use crate::transcription_api_client;
use crate::utils::cancel_current_operation;
use anyhow::Result;
use bzip2::read::BzDecoder;
use futures_util::StreamExt;
use log::{info, warn};
use std::fs;
use std::fs::File;
use std::io::{BufReader, Write};
use std::sync::Arc;
use tar::Archive;
use std::process::Command;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
#[specta::specta]
pub fn cancel_operation(app: AppHandle) {
    cancel_current_operation(&app);
}

#[tauri::command]
#[specta::specta]
pub fn get_app_dir_path(app: AppHandle) -> Result<String, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    Ok(app_data_dir.to_string_lossy().to_string())
}

#[tauri::command]
#[specta::specta]
pub fn get_app_settings(app: AppHandle) -> Result<AppSettings, String> {
    Ok(get_settings(&app))
}

#[tauri::command]
#[specta::specta]
pub fn get_default_settings() -> Result<AppSettings, String> {
    Ok(crate::settings::get_default_settings())
}

#[tauri::command]
#[specta::specta]
pub fn get_log_dir_path(app: AppHandle) -> Result<String, String> {
    let log_dir = app
        .path()
        .app_log_dir()
        .map_err(|e| format!("Failed to get log directory: {}", e))?;

    Ok(log_dir.to_string_lossy().to_string())
}

#[specta::specta]
#[tauri::command]
pub fn set_log_level(app: AppHandle, level: LogLevel) -> Result<(), String> {
    let tauri_log_level: tauri_plugin_log::LogLevel = level.into();
    let log_level: log::Level = tauri_log_level.into();
    // Update the file log level atomic so the filter picks up the new level
    crate::FILE_LOG_LEVEL.store(
        log_level.to_level_filter() as u8,
        std::sync::atomic::Ordering::Relaxed,
    );

    let mut settings = get_settings(&app);
    settings.log_level = level;
    write_settings(&app, settings);

    Ok(())
}

#[specta::specta]
#[tauri::command]
pub fn open_recordings_folder(app: AppHandle) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let recordings_dir = app_data_dir.join("recordings");

    let path = recordings_dir.to_string_lossy().as_ref().to_string();
    app.opener()
        .open_path(path, None::<String>)
        .map_err(|e| format!("Failed to open recordings folder: {}", e))?;

    Ok(())
}

#[specta::specta]
#[tauri::command]
pub fn open_log_dir(app: AppHandle) -> Result<(), String> {
    let log_dir = app
        .path()
        .app_log_dir()
        .map_err(|e| format!("Failed to get log directory: {}", e))?;

    let path = log_dir.to_string_lossy().as_ref().to_string();
    app.opener()
        .open_path(path, None::<String>)
        .map_err(|e| format!("Failed to open log directory: {}", e))?;

    Ok(())
}

#[specta::specta]
#[tauri::command]
pub fn open_app_data_dir(app: AppHandle) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let _ = std::fs::create_dir_all(&app_data_dir);

    let config_path = app_data_dir.join("models.toml");
    if !config_path.exists() {
        let default_path = app
            .path()
            .resolve("resources/default_models.toml", tauri::path::BaseDirectory::Resource)
            .map_err(|e| format!("Failed to get default config path: {}", e))?;
        if default_path.exists() {
            fs::copy(&default_path, &config_path)
                .map_err(|e| format!("Failed to create models.toml: {}", e))?;
        } else {
            return Err("Default config not found. Please create models.toml manually.".into());
        }
    }

    let path = config_path.to_string_lossy().into_owned();

    #[cfg(target_os = "macos")]
    Command::new("open").arg(&path).spawn()
        .map_err(|e| format!("Failed to open config file: {}", e))?;

    #[cfg(target_os = "windows")]
    Command::new("cmd").args(["/C", "start", "", &path]).spawn()
        .map_err(|e| format!("Failed to open config file: {}", e))?;

    #[cfg(target_os = "linux")]
    Command::new("xdg-open").arg(&path).spawn()
        .map_err(|e| format!("Failed to open config file: {}", e))?;

    Ok(())
}

/// Check if Apple Intelligence is available on this device.
/// Called by the frontend when the user selects Apple Intelligence provider.
#[specta::specta]
#[tauri::command]
pub fn check_apple_intelligence_available() -> bool {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        crate::apple_intelligence::check_apple_intelligence_availability()
    }
    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    {
        false
    }
}

/// Clear LLM memory (rectify records)
#[specta::specta]
#[tauri::command]
pub fn clear_llm_memory(app: AppHandle) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let rectify_file = app_data_dir.join("hot-rectify.txt");

    if rectify_file.exists() {
        fs::remove_file(&rectify_file)
            .map_err(|e| format!("Failed to remove rectify file: {}", e))?;
    }

    Ok(())
}

/// Try to initialize Enigo (keyboard/mouse simulation).
/// On macOS, this will return an error if accessibility permissions are not granted.
#[specta::specta]
#[tauri::command]
pub fn initialize_enigo(app: AppHandle) -> Result<(), String> {
    use crate::input::EnigoState;

    // Check if already initialized
    if app.try_state::<EnigoState>().is_some() {
        log::debug!("Enigo already initialized");
        return Ok(());
    }

    // Try to initialize
    match EnigoState::new() {
        Ok(enigo_state) => {
            app.manage(enigo_state);
            log::info!("Enigo initialized successfully after permission grant");
            Ok(())
        }
        Err(e) => {
            if cfg!(target_os = "macos") {
                log::warn!(
                    "Failed to initialize Enigo: {} (accessibility permissions may not be granted)",
                    e
                );
            } else {
                log::warn!("Failed to initialize Enigo: {}", e);
            }
            Err(format!("Failed to initialize input system: {}", e))
        }
    }
}

/// Test transcription API configuration with silent audio
#[specta::specta]
#[tauri::command]
pub async fn test_transcription_api_config(
    provider_id: String,
    api_key: String,
    model: String,
    custom_api_url: Option<String>,
    app: AppHandle,
) -> Result<String, String> {
    let settings = get_settings(&app);

    let provider = settings
        .transcription_api_providers
        .iter()
        .find(|p| p.id == provider_id)
        .ok_or_else(|| format!("Provider not found: {}", provider_id))?;

    transcription_api_client::test_transcription_api(provider, api_key, &model, custom_api_url)
        .await
}

/// Test transcription API configuration with local audio file
#[specta::specta]
#[tauri::command]
pub async fn test_transcription_api_with_file(
    provider_id: String,
    api_key: String,
    model: String,
    custom_api_url: Option<String>,
    app: AppHandle,
) -> Result<String, String> {
    use log::warn;
    use std::path::PathBuf;

    warn!(
        "[Transcription API Test] Command called with provider_id: {}, model: {}",
        provider_id, model
    );

    let settings = get_settings(&app);

    let provider = settings
        .transcription_api_providers
        .iter()
        .find(|p| p.id == provider_id)
        .ok_or_else(|| {
            let error = format!("Provider not found: {}", provider_id);
            warn!("[Transcription API Test] {}", error);
            error
        })?;

    let audio_path =
        PathBuf::from("/Users/thinkre/Desktop/projects/KeVoiceInput/scripts/welcome.mp3");
    warn!("[Transcription API Test] Audio file path: {:?}", audio_path);

    transcription_api_client::test_transcription_api_with_file(
        provider,
        api_key,
        &model,
        audio_path,
        custom_api_url,
    )
    .await
}

/// Ensure punctuation model is downloaded and loaded
#[specta::specta]
#[tauri::command]
pub async fn ensure_punctuation_model(
    app: AppHandle,
    punctuation_manager: State<'_, Arc<PunctuationManager>>,
) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    // Model directory name
    let model_dir_name = "sherpa-onnx-punct-ct-transformer-zh-en-vocab272727-2024-04-12-int8";
    let model_dir = app_data_dir.join("models").join(model_dir_name);
    let model_file = model_dir.join("model.int8.onnx");

    // Check if model already exists
    if model_file.exists() {
        info!("Punctuation model already exists at: {:?}", model_file);
        // Update settings with model path
        let mut settings = get_settings(&app);
        settings.punctuation_model_path = Some(
            model_file
                .to_string_lossy()
                .to_string(),
        );
        write_settings(&app, settings);

        // Load the model
        punctuation_manager
            .load_model(
                model_file
                    .to_string_lossy()
                    .as_ref(),
            )
            .map_err(|e| format!("Failed to load punctuation model: {}", e))?;

        return Ok(());
    }

    // Model doesn't exist, download it
    info!("Punctuation model not found, downloading...");
    let download_url = "https://github.com/k2-fsa/sherpa-onnx/releases/download/punctuation-models/sherpa-onnx-punct-ct-transformer-zh-en-vocab272727-2024-04-12-int8.tar.bz2";
    let archive_name = "sherpa-onnx-punct-ct-transformer-zh-en-vocab272727-2024-04-12-int8.tar.bz2";
    let archive_path = app_data_dir.join("models").join(archive_name);

    // Create models directory if it doesn't exist
    let models_dir = app_data_dir.join("models");
    if !models_dir.exists() {
        fs::create_dir_all(&models_dir)
            .map_err(|e| format!("Failed to create models directory: {}", e))?;
    }

    // Download the archive
    info!("Downloading punctuation model from: {}", download_url);
    let response = reqwest::get(download_url)
        .await
        .map_err(|e| format!("Failed to download punctuation model: {}", e))?;

    let total_size = response
        .content_length()
        .ok_or_else(|| "Failed to get content length".to_string())?;

    let mut file = File::create(&archive_path)
        .map_err(|e| format!("Failed to create archive file: {}", e))?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| format!("Download error: {}", e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("Failed to write to file: {}", e))?;
        downloaded += chunk.len() as u64;
        if downloaded % (1024 * 1024) == 0 {
            info!(
                "Downloaded {} / {} MB",
                downloaded / (1024 * 1024),
                total_size / (1024 * 1024)
            );
        }
    }

    info!("Download completed, extracting...");

    // Extract the archive
    let tar_bz2 = File::open(&archive_path)
        .map_err(|e| format!("Failed to open archive: {}", e))?;
    let tar = BzDecoder::new(BufReader::new(tar_bz2));
    let mut archive = Archive::new(tar);

    archive
        .unpack(&models_dir)
        .map_err(|e| format!("Failed to extract archive: {}", e))?;

    // Clean up archive file
    if let Err(e) = fs::remove_file(&archive_path) {
        warn!("Failed to remove archive file: {}", e);
    }

    // Verify model file exists
    if !model_file.exists() {
        return Err(format!(
            "Model file not found after extraction: {:?}",
            model_file
        ));
    }

    info!("Punctuation model extracted successfully");

    // Update settings with model path
    let mut settings = get_settings(&app);
    settings.punctuation_model_path = Some(
        model_file
            .to_string_lossy()
            .to_string(),
    );
    write_settings(&app, settings);

    // Load the model
    punctuation_manager
        .load_model(
            model_file
                .to_string_lossy()
                .as_ref(),
        )
        .map_err(|e| format!("Failed to load punctuation model: {}", e))?;

    info!("Punctuation model loaded successfully");
    Ok(())
}
