use crate::settings;
use crate::tray_i18n::get_tray_translations;
use tauri::image::Image;
use tauri::menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Manager};

/// Create and set the macOS application menu
#[cfg(target_os = "macos")]
pub fn setup_app_menu(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let settings = settings::get_settings(app);
    let locale = &settings.app_language;
    let strings = get_tray_translations(Some(locale.to_string()));

    // Create About menu item using PredefinedMenuItem::about
    // This will show the native macOS About dialog with the custom logo
    let about_text = if locale.starts_with("zh") {
        "关于 KeVoiceInput"
    } else {
        "About KeVoiceInput"
    };

    // Load the custom logo for the About dialog
    let icon_path = app
        .path()
        .resolve(
            "resources/voice_input.png",
            tauri::path::BaseDirectory::Resource,
        )
        .ok();

    let about_icon = icon_path.and_then(|p| Image::from_path(p).ok());

    // Create AboutMetadata with custom icon
    let about_metadata = AboutMetadata {
        name: Some("KeVoiceInput".to_string()),
        version: Some("0.1.0".to_string()),
        short_version: None,
        copyright: None,
        authors: None,
        comments: None,
        credits: None,
        license: None,
        website: None,
        website_label: None,
        icon: about_icon,
    };

    let about_item = PredefinedMenuItem::about(app, Some(about_text), Some(about_metadata))
        .map_err(|e| format!("Failed to create about menu item: {}", e))?;

    // Create Preferences menu item
    let preferences_item = MenuItem::with_id(
        app,
        "preferences",
        &strings.settings,
        true,
        Some("CmdOrCtrl+,"),
    )
    .map_err(|e| format!("Failed to create preferences menu item: {}", e))?;

    // Create separator helper
    let separator = || PredefinedMenuItem::separator(app).expect("failed to create separator");

    // Create Hide menu item
    let hide_text = if locale.starts_with("zh") {
        "隐藏 KeVoiceInput"
    } else {
        "Hide KeVoiceInput"
    };
    let hide_item = MenuItem::with_id(app, "hide", hide_text, true, Some("CmdOrCtrl+H"))
        .expect("failed to create hide item");

    // Create Hide Others menu item
    let hide_others_text = if locale.starts_with("zh") {
        "隐藏其他"
    } else {
        "Hide Others"
    };
    let hide_others_item = MenuItem::with_id(
        app,
        "hide_others",
        hide_others_text,
        true,
        Some("Alt+CmdOrCtrl+H"),
    )
    .expect("failed to create hide_others item");

    // Create Show All menu item
    let show_all_text = if locale.starts_with("zh") {
        "显示全部"
    } else {
        "Show All"
    };
    let show_all_item = MenuItem::with_id(app, "show_all", show_all_text, true, None::<&str>)
        .expect("failed to create show_all item");

    // Create Quit menu item
    let quit_item = MenuItem::with_id(app, "quit", &strings.quit, true, Some("CmdOrCtrl+Q"))
        .expect("failed to create quit item");

    // Create App submenu (first submenu becomes the App menu on macOS)
    let app_submenu = Submenu::with_items(
        app,
        "KeVoiceInput",
        true,
        &[
            &about_item,
            &separator(),
            &preferences_item,
            &separator(),
            &hide_item,
            &hide_others_item,
            &show_all_item,
            &separator(),
            &quit_item,
        ],
    )
    .expect("failed to create app submenu");

    // Create the main menu
    let menu = Menu::with_items(app, &[&app_submenu])
        .map_err(|e| format!("Failed to create menu: {}", e))?;

    // Set as app menu
    app.set_menu(menu)
        .map_err(|e| format!("Failed to set menu: {}", e))?;

    Ok(())
}

/// Update the app menu when language changes
#[cfg(target_os = "macos")]
pub fn update_app_menu(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    setup_app_menu(app)
}

#[cfg(not(target_os = "macos"))]
pub fn setup_app_menu(_app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn update_app_menu(_app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
