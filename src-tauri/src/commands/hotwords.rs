use crate::settings::{get_settings, write_settings};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HotwordFile {
    pub name: String,        // Display name without extension
    pub path: String,        // Full path
    pub selected: bool,      // Whether this file is selected
}

#[tauri::command]
#[specta::specta]
pub fn get_hotword_files(app: AppHandle) -> Result<Vec<HotwordFile>, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let hotwords_dir = app_data_dir.join("hotwords");
    
    // Create directory if it doesn't exist
    if !hotwords_dir.exists() {
        fs::create_dir_all(&hotwords_dir)
            .map_err(|e| format!("Failed to create hotwords directory: {}", e))?;
    }

    let settings = get_settings(&app);
    let selected_files = settings.selected_hotword_files.clone();

    let mut files = Vec::new();
    
    log::info!("[Hotwords] Reading hotwords directory: {:?}", hotwords_dir);
    
    if let Ok(entries) = fs::read_dir(&hotwords_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("txt") {
                    // Normalize path for consistent comparison
                    let normalized_path = path.canonicalize()
                        .unwrap_or(path.clone())
                        .to_string_lossy()
                        .to_string();
                    
                    let file_name = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    
                    // Check if selected using normalized path
                    let selected = selected_files.iter().any(|selected_path| {
                        if let Ok(selected_canonical) = PathBuf::from(selected_path).canonicalize() {
                            selected_canonical.to_string_lossy() == normalized_path
                        } else {
                            selected_path == &normalized_path
                        }
                    });
                    
                    log::info!("[Hotwords] Found file: name={}, path={}, selected={}", file_name, normalized_path, selected);
                    
                    files.push(HotwordFile {
                        name: file_name,
                        path: normalized_path,
                        selected,
                    });
                }
            }
        }
    } else {
        log::warn!("[Hotwords] Failed to read hotwords directory: {:?}", hotwords_dir);
    }
    
    log::info!("[Hotwords] Total files found: {}", files.len());

    // Sort by name
    files.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(files)
}

#[tauri::command]
#[specta::specta]
pub fn upload_hotword_file(
    app: AppHandle,
    file_path: String,
) -> Result<HotwordFile, String> {
    let source_path = PathBuf::from(&file_path);
    
    if !source_path.exists() {
        return Err("Source file does not exist".to_string());
    }

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let hotwords_dir = app_data_dir.join("hotwords");
    
    // Create directory if it doesn't exist
    if !hotwords_dir.exists() {
        fs::create_dir_all(&hotwords_dir)
            .map_err(|e| format!("Failed to create hotwords directory: {}", e))?;
    }

    // Get file name from source
    let file_name = source_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "Invalid file name".to_string())?;

    // Ensure it has .txt extension
    let target_name = if file_name.ends_with(".txt") {
        file_name.to_string()
    } else {
        format!("{}.txt", file_name)
    };

    let target_path = hotwords_dir.join(&target_name);

    // Copy file
    fs::copy(&source_path, &target_path)
        .map_err(|e| format!("Failed to copy file: {}", e))?;

    // Get file stem before canonicalize (which may consume the path)
    let file_stem = target_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Normalize path for consistent comparison
    let normalized_path = target_path.canonicalize()
        .unwrap_or(target_path)
        .to_string_lossy()
        .to_string();

    log::info!("[Hotwords] Uploaded file: name={}, path={}", file_stem, normalized_path);

    Ok(HotwordFile {
        name: file_stem,
        path: normalized_path,
        selected: false,
    })
}

#[tauri::command]
#[specta::specta]
pub fn delete_hotword_file(
    app: AppHandle,
    file_path: String,
) -> Result<(), String> {
    let path = PathBuf::from(&file_path);
    
    if !path.exists() {
        return Err("File does not exist".to_string());
    }

    fs::remove_file(&path)
        .map_err(|e| format!("Failed to delete file: {}", e))?;

    // Remove from selected files if present
    let mut settings = get_settings(&app);
    settings.selected_hotword_files.retain(|p| p != &file_path);
    write_settings(&app, settings);

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn read_hotword_file(file_path: String) -> Result<String, String> {
    fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
#[specta::specta]
pub fn write_hotword_file(
    file_path: String,
    content: String,
) -> Result<(), String> {
    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write file: {}", e))
}

#[tauri::command]
#[specta::specta]
pub fn toggle_hotword_file_selection(
    app: AppHandle,
    file_path: String,
) -> Result<(), String> {
    let mut settings = get_settings(&app);
    
    if settings.selected_hotword_files.contains(&file_path) {
        settings.selected_hotword_files.retain(|p| p != &file_path);
    } else {
        settings.selected_hotword_files.push(file_path);
    }
    
    write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn add_word_to_user_default(
    app: AppHandle,
    word: String,
) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let hotwords_dir = app_data_dir.join("hotwords");
    
    // Create directory if it doesn't exist
    if !hotwords_dir.exists() {
        fs::create_dir_all(&hotwords_dir)
            .map_err(|e| format!("Failed to create hotwords directory: {}", e))?;
    }

    let user_default_path = hotwords_dir.join("UserDefault.txt");

    // Read existing content
    let mut words = if user_default_path.exists() {
        fs::read_to_string(&user_default_path)
            .map_err(|e| format!("Failed to read UserDefault.txt: {}", e))?
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>()
    } else {
        Vec::new()
    };

    // Add new word if not already present
    let trimmed_word = word.trim().to_string();
    if !trimmed_word.is_empty() && !words.contains(&trimmed_word) {
        words.push(trimmed_word);
        
        // Write back to file (one word per line)
        let mut file = fs::File::create(&user_default_path)
            .map_err(|e| format!("Failed to create UserDefault.txt: {}", e))?;
        
        for word in &words {
            writeln!(file, "{}", word)
                .map_err(|e| format!("Failed to write to UserDefault.txt: {}", e))?;
        }
    }

    Ok(())
}
