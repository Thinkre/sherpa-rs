use crate::settings::AppSettings;
use anyhow::{bail, Result};
use log::{debug, error, info, warn};
use sherpa_rs::punctuate::{Punctuation, PunctuationConfig};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

/// Manager for punctuation model
pub struct PunctuationManager {
    app_handle: AppHandle,
    punctuation: Arc<Mutex<Option<Punctuation>>>,
    current_model_path: Arc<Mutex<Option<String>>>,
}

impl PunctuationManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            punctuation: Arc::new(Mutex::new(None)),
            current_model_path: Arc::new(Mutex::new(None)),
        }
    }

    /// Load punctuation model from path
    pub fn load_model(&self, model_path: &str) -> Result<()> {
        let path = PathBuf::from(model_path);
        
        if !path.exists() {
            bail!("Punctuation model file does not exist: {}", model_path);
        }

        if !path.is_file() {
            bail!("Punctuation model path is not a file: {}", model_path);
        }

        info!("Loading punctuation model from: {}", model_path);

        // Check if we already have this model loaded
        {
            let current_path = self.current_model_path.lock().unwrap();
            if let Some(ref current) = *current_path {
                if current == model_path {
                    debug!("Punctuation model already loaded: {}", model_path);
                    return Ok(());
                }
            }
        }

        // Unload existing model if any
        self.unload_model();

        // Create new punctuation instance
        let config = PunctuationConfig {
            model: model_path.to_string(),
            debug: false,
            num_threads: Some(1),
            provider: None,
        };

        match Punctuation::new(config) {
            Ok(punct) => {
                *self.punctuation.lock().unwrap() = Some(punct);
                *self.current_model_path.lock().unwrap() = Some(model_path.to_string());
                info!("Punctuation model loaded successfully: {}", model_path);
                Ok(())
            }
            Err(e) => {
                error!("Failed to load punctuation model: {}", e);
                Err(anyhow::anyhow!("Failed to create punctuation model from {}: {}", model_path, e))
            }
        }
    }

    /// Unload punctuation model
    pub fn unload_model(&self) {
        let mut punct = self.punctuation.lock().unwrap();
        *punct = None;
        let mut path = self.current_model_path.lock().unwrap();
        *path = None;
        debug!("Punctuation model unloaded");
    }

    /// Add punctuation to text
    pub fn add_punctuation(&self, text: &str) -> Result<String> {
        let mut punct = self.punctuation.lock().unwrap();
        
        if let Some(ref mut punct_model) = *punct {
            info!("Adding punctuation to text: '{}'", text);
            let result = punct_model.add_punctuation(text);
            info!("Punctuation result: '{}'", result);
            Ok(result)
        } else {
            warn!("Punctuation model not loaded, returning original text. Is loaded: {}", self.is_loaded());
            Ok(text.to_string())
        }
    }

    /// Check if punctuation model is loaded
    pub fn is_loaded(&self) -> bool {
        self.punctuation.lock().unwrap().is_some()
    }

    /// Ensure punctuation model is loaded based on settings
    pub fn ensure_loaded(&self, settings: &AppSettings) -> Result<()> {
        if !settings.punctuation_enabled {
            // If disabled, unload model if loaded
            if self.is_loaded() {
                debug!("Punctuation disabled, unloading model");
                self.unload_model();
            }
            return Ok(());
        }

        info!("Punctuation enabled, checking model path: {:?}", settings.punctuation_model_path);

        if let Some(ref model_path) = settings.punctuation_model_path {
            if !model_path.is_empty() {
                // Check if we need to load or reload
                let needs_load = {
                    let current_path = self.current_model_path.lock().unwrap();
                    current_path.as_ref().map(|p| p != model_path).unwrap_or(true)
                };

                if needs_load {
                    info!("Loading punctuation model from: {}", model_path);
                    self.load_model(model_path)?;
                    info!("Punctuation model loaded successfully");
                } else {
                    debug!("Punctuation model already loaded from: {}", model_path);
                }
            } else {
                warn!("Punctuation enabled but model path is empty");
            }
        } else {
            warn!("Punctuation enabled but no model path specified in settings");
        }

        Ok(())
    }
}
