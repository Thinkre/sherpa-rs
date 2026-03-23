use enigo::{Enigo, Key, Keyboard, Mouse, Settings};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

/// Wrapper for Enigo to store in Tauri's managed state.
/// Enigo is wrapped in a Mutex since it requires mutable access.
pub struct EnigoState(pub Mutex<Enigo>);

impl EnigoState {
    pub fn new() -> Result<Self, String> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| format!("Failed to initialize Enigo: {}", e))?;
        Ok(Self(Mutex::new(enigo)))
    }
}

/// Get the current mouse cursor position using the managed Enigo instance.
/// Returns None if the state is not available or if getting the location fails.
pub fn get_cursor_position(app_handle: &AppHandle) -> Option<(i32, i32)> {
    let enigo_state = app_handle.try_state::<EnigoState>()?;
    let enigo = enigo_state.0.lock().ok()?;
    enigo.location().ok()
}

/// Sends a Ctrl+V or Cmd+V paste command using platform-specific virtual key codes.
/// This ensures the paste works regardless of keyboard layout (e.g., Russian, AZERTY, DVORAK).
/// Note: On Wayland, this may not work - callers should check for Wayland and use alternative methods.
pub fn send_paste_ctrl_v(enigo: &mut Enigo) -> Result<(), String> {
    // Platform-specific key definitions
    #[cfg(target_os = "macos")]
    let (modifier_key, v_key_code) = (Key::Meta, Key::Other(9));
    #[cfg(target_os = "windows")]
    let (modifier_key, v_key_code) = (Key::Control, Key::Other(0x56)); // VK_V
    #[cfg(target_os = "linux")]
    let (modifier_key, v_key_code) = (Key::Control, Key::Unicode('v'));

    // Press modifier + V
    enigo
        .key(modifier_key, enigo::Direction::Press)
        .map_err(|e| format!("Failed to press modifier key: {}", e))?;
    enigo
        .key(v_key_code, enigo::Direction::Click)
        .map_err(|e| format!("Failed to click V key: {}", e))?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    enigo
        .key(modifier_key, enigo::Direction::Release)
        .map_err(|e| format!("Failed to release modifier key: {}", e))?;

    Ok(())
}

/// Sends a Ctrl+Shift+V paste command.
/// This is commonly used in terminal applications on Linux to paste without formatting.
/// Note: On Wayland, this may not work - callers should check for Wayland and use alternative methods.
pub fn send_paste_ctrl_shift_v(enigo: &mut Enigo) -> Result<(), String> {
    // Platform-specific key definitions
    #[cfg(target_os = "macos")]
    let (modifier_key, v_key_code) = (Key::Meta, Key::Other(9)); // Cmd+Shift+V on macOS
    #[cfg(target_os = "windows")]
    let (modifier_key, v_key_code) = (Key::Control, Key::Other(0x56)); // VK_V
    #[cfg(target_os = "linux")]
    let (modifier_key, v_key_code) = (Key::Control, Key::Unicode('v'));

    // Press Ctrl/Cmd + Shift + V
    enigo
        .key(modifier_key, enigo::Direction::Press)
        .map_err(|e| format!("Failed to press modifier key: {}", e))?;
    enigo
        .key(Key::Shift, enigo::Direction::Press)
        .map_err(|e| format!("Failed to press Shift key: {}", e))?;
    enigo
        .key(v_key_code, enigo::Direction::Click)
        .map_err(|e| format!("Failed to click V key: {}", e))?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    enigo
        .key(Key::Shift, enigo::Direction::Release)
        .map_err(|e| format!("Failed to release Shift key: {}", e))?;
    enigo
        .key(modifier_key, enigo::Direction::Release)
        .map_err(|e| format!("Failed to release modifier key: {}", e))?;

    Ok(())
}

/// Sends a Shift+Insert paste command (Windows and Linux only).
/// This is more universal for terminal applications and legacy software.
/// Note: On Wayland, this may not work - callers should check for Wayland and use alternative methods.
pub fn send_paste_shift_insert(enigo: &mut Enigo) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let insert_key_code = Key::Other(0x2D); // VK_INSERT
    #[cfg(not(target_os = "windows"))]
    let insert_key_code = Key::Other(0x76); // XK_Insert (keycode 118 / 0x76, also used as fallback)

    // Press Shift + Insert
    enigo
        .key(Key::Shift, enigo::Direction::Press)
        .map_err(|e| format!("Failed to press Shift key: {}", e))?;
    enigo
        .key(insert_key_code, enigo::Direction::Click)
        .map_err(|e| format!("Failed to click Insert key: {}", e))?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    enigo
        .key(Key::Shift, enigo::Direction::Release)
        .map_err(|e| format!("Failed to release Shift key: {}", e))?;

    Ok(())
}

/// Pastes text directly using the enigo text method.
/// This tries to use system input methods if possible, otherwise simulates keystrokes one by one.
pub fn paste_text_direct(enigo: &mut Enigo, text: &str) -> Result<(), String> {
    enigo
        .text(text)
        .map_err(|e| format!("Failed to send text directly: {}", e))?;

    Ok(())
}

/// Sends a Ctrl+C or Cmd+C copy command using platform-specific virtual key codes.
/// This is used to copy selected text to clipboard.
/// Returns Ok(()) if successful, Err with error message otherwise.
pub fn send_copy_ctrl_c(enigo: &mut Enigo) -> Result<(), String> {
    use log::{error, info};

    // Platform-specific key definitions
    #[cfg(target_os = "macos")]
    let (modifier_key, c_key_code) = (Key::Meta, Key::Unicode('c'));
    #[cfg(target_os = "windows")]
    let (modifier_key, c_key_code) = (Key::Control, Key::Unicode('c'));
    #[cfg(target_os = "linux")]
    let (modifier_key, c_key_code) = (Key::Control, Key::Unicode('c'));

    info!("send_copy_ctrl_c: Pressing modifier key");
    // Press modifier + C
    if let Err(e) = enigo.key(modifier_key, enigo::Direction::Press) {
        error!("send_copy_ctrl_c: Failed to press modifier key: {}", e);
        return Err(format!("Failed to press modifier key: {}", e));
    }
    info!("send_copy_ctrl_c: Modifier key pressed successfully");

    // Small delay to ensure modifier is registered
    std::thread::sleep(std::time::Duration::from_millis(10));

    info!("send_copy_ctrl_c: Clicking C key");
    // Use catch_unwind to prevent panic from crashing the app
    let c_key_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        enigo.key(c_key_code, enigo::Direction::Click)
    }));

    match c_key_result {
        Ok(Ok(_)) => {
            info!("send_copy_ctrl_c: C key clicked successfully");
        }
        Ok(Err(e)) => {
            error!("send_copy_ctrl_c: Failed to click C key: {}", e);
            // Try to release modifier even if C failed
            info!("send_copy_ctrl_c: Attempting to release modifier after C key failure");
            let _ = enigo.key(modifier_key, enigo::Direction::Release);
            return Err(format!("Failed to click C key: {}", e));
        }
        Err(panic_info) => {
            error!("send_copy_ctrl_c: C key click panicked!");
            // Log the panic payload if available
            if let Some(s) = panic_info.downcast_ref::<String>() {
                error!("Panic message: {}", s);
            } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                error!("Panic message: {}", s);
            }
            // Try to release modifier even if C panicked
            info!("send_copy_ctrl_c: Attempting to release modifier after C key panic");
            // Use catch_unwind for release as well to prevent double panic
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = enigo.key(modifier_key, enigo::Direction::Release);
            }));
            return Err("C key click panicked".to_string());
        }
    }

    std::thread::sleep(std::time::Duration::from_millis(50));

    info!("send_copy_ctrl_c: Releasing modifier key");
    if let Err(e) = enigo.key(modifier_key, enigo::Direction::Release) {
        error!("send_copy_ctrl_c: Failed to release modifier key: {}", e);
        return Err(format!("Failed to release modifier key: {}", e));
    }
    info!("send_copy_ctrl_c: Modifier key released successfully");

    Ok(())
}
