#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
use crate::apple_intelligence;
use crate::audio_feedback::{play_feedback_sound, play_feedback_sound_blocking, SoundType};
use crate::audio_toolkit::apply_itn;
use crate::input::EnigoState;
use crate::managers::audio::AudioRecordingManager;
use crate::managers::history::HistoryManager;
use crate::managers::punctuation::PunctuationManager;
use crate::managers::transcription::TranscriptionManager;
use crate::settings::{
    get_settings, AppSettings, LLMRole, PostProcessProvider, APPLE_INTELLIGENCE_PROVIDER_ID,
};
use crate::shortcut;
use crate::tray::{change_tray_icon, TrayIconState};
use crate::utils::{self, show_recording_overlay, show_transcribing_overlay};
use crate::ManagedToggleState;
use anyhow;
use ferrous_opencc::{config::BuiltinConfig, OpenCC};
use log::{debug, error, info, warn};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;
use tauri::AppHandle;
use tauri::{Emitter, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;

// Shortcut Action Trait
pub trait ShortcutAction: Send + Sync {
    fn start(&self, app: &AppHandle, binding_id: &str, shortcut_str: &str);
    fn stop(&self, app: &AppHandle, binding_id: &str, shortcut_str: &str);
}

// Transcribe Action
struct TranscribeAction;

/// Match LLM role from transcription text
/// Returns (role, remaining_text) if matched, None otherwise
fn match_llm_role<'a>(text: &'a str, roles: &'a [LLMRole]) -> Option<(&'a LLMRole, String)> {
    // Normalize text: remove leading/trailing whitespace and punctuation
    let normalized = text.trim();

    for role in roles {
        // Check if text starts with role name (case-insensitive, fuzzy match)
        let role_name = role.name.trim();
        let role_name_lower = role_name.to_lowercase();

        // Try exact match first
        if normalized.starts_with(role_name)
            || normalized.to_lowercase().starts_with(&role_name_lower)
        {
            let remaining = normalized[role_name.len()..]
                .trim_start_matches(|c: char| {
                    c.is_whitespace() || c == '，' || c == ',' || c == '。' || c == '.'
                })
                .to_string();
            return Some((role, remaining));
        }

        // Try fuzzy match: remove punctuation and spaces
        let text_clean: String = normalized
            .chars()
            .filter(|c| !c.is_whitespace() && !c.is_ascii_punctuation() && *c != '，' && *c != '。')
            .collect();
        let role_clean: String = role_name
            .chars()
            .filter(|c| !c.is_whitespace() && !c.is_ascii_punctuation() && *c != '，' && *c != '。')
            .collect();

        if text_clean
            .to_lowercase()
            .starts_with(&role_clean.to_lowercase())
        {
            // Find where the role name ends in the original text
            let mut matched_len = 0;
            let mut role_chars = role_clean.chars().peekable();
            for (i, ch) in normalized.char_indices() {
                if !ch.is_whitespace() && !ch.is_ascii_punctuation() && ch != '，' && ch != '。' {
                    if let Some(&role_ch) = role_chars.peek() {
                        if ch.to_lowercase().next() == role_ch.to_lowercase().next() {
                            role_chars.next();
                            if role_chars.peek().is_none() {
                                matched_len = i + ch.len_utf8();
                                break;
                            }
                        }
                    }
                }
            }
            let remaining = normalized[matched_len..]
                .trim_start_matches(|c: char| {
                    c.is_whitespace() || c == '，' || c == ',' || c == '。' || c == '.'
                })
                .to_string();
            return Some((role, remaining));
        }
    }

    None
}

/// Get selected text from system clipboard
/// Uses Cmd+C (macOS) or Ctrl+C (Windows/Linux) to copy selection, then reads clipboard
pub fn get_selected_text(app_handle: &AppHandle) -> Result<String, String> {
    info!("get_selected_text: Starting");

    #[cfg(target_os = "macos")]
    {
        // Use get-selected-text crate on macOS for safer implementation
        // This crate uses Accessibility API first, then falls back to Cmd+C
        use get_selected_text::get_selected_text as get_selected_text_crate;

        info!("get_selected_text: Attempting to use get-selected-text crate");
        match get_selected_text_crate() {
            Ok(text) if !text.trim().is_empty() => {
                info!(
                    "get_selected_text: Got selected text using crate, length: {}",
                    text.len()
                );
                return Ok(text);
            }
            Ok(_) => {
                info!("get_selected_text: Selected text is empty (from crate)");
                return Err("No text selected".to_string());
            }
            Err(e) => {
                warn!("get_selected_text: Failed to get selected text using crate: {:?}, falling back to Enigo method", e);
                // Fall back to Enigo method
            }
        }
    }

    // Fallback to Enigo method (for non-macOS or if get-selected-text crate fails)
    let clipboard = app_handle.clipboard();
    info!("get_selected_text: Got clipboard handle");

    let original_clipboard = match clipboard.read_text() {
        Ok(text) => {
            debug!(
                "get_selected_text: Read original clipboard, length: {}",
                text.len()
            );
            text
        }
        Err(e) => {
            debug!(
                "get_selected_text: Failed to read clipboard: {:?}, using empty",
                e
            );
            String::new()
        }
    };

    info!("get_selected_text: Getting Enigo state");
    // Get Enigo instance
    let enigo_state = match app_handle.try_state::<EnigoState>() {
        Some(state) => {
            info!("get_selected_text: Found Enigo state");
            state
        }
        None => {
            error!("get_selected_text: Enigo state not found");
            return Err("Enigo state not found".to_string());
        }
    };

    info!("get_selected_text: Locking Enigo");
    let mut enigo = match enigo_state.0.lock() {
        Ok(enigo) => {
            info!("get_selected_text: Successfully locked Enigo");
            enigo
        }
        Err(e) => {
            error!("get_selected_text: Failed to lock Enigo: {:?}", e);
            return Err(format!("Failed to lock Enigo: {:?}", e));
        }
    };

    info!("get_selected_text: Sending copy command");
    // Use the safer send_copy_ctrl_c function with panic protection
    let copy_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        crate::input::send_copy_ctrl_c(&mut enigo)
    }));

    match copy_result {
        Ok(Ok(_)) => {
            info!("get_selected_text: Copy command sent successfully");
        }
        Ok(Err(e)) => {
            error!("get_selected_text: Failed to send copy command: {}", e);
            return Err(format!("Failed to send copy command: {}", e));
        }
        Err(panic_info) => {
            error!("get_selected_text: send_copy_ctrl_c panicked!");
            // Log the panic payload if available
            if let Some(s) = panic_info.downcast_ref::<String>() {
                error!("Panic message: {}", s);
            } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                error!("Panic message: {}", s);
            } else {
                error!("Panic info: {:?}", panic_info);
            }
            return Err("send_copy_ctrl_c panicked".to_string());
        }
    }

    info!("get_selected_text: Waiting 100ms for clipboard update");
    // Wait a bit for clipboard to update
    std::thread::sleep(std::time::Duration::from_millis(100));

    info!("get_selected_text: Reading clipboard after copy");
    // Read clipboard
    let selected_text = match clipboard.read_text() {
        Ok(text) => {
            info!(
                "get_selected_text: Read selected text, length: {}",
                text.len()
            );
            text
        }
        Err(e) => {
            warn!(
                "get_selected_text: Failed to read clipboard after copy: {:?}",
                e
            );
            String::new()
        }
    };

    info!("get_selected_text: Restoring original clipboard");
    // Restore original clipboard
    if let Err(e) = clipboard.write_text(&original_clipboard) {
        warn!("get_selected_text: Failed to restore clipboard: {:?}", e);
    }

    info!("get_selected_text: Comparing selected text with original");
    if selected_text == original_clipboard {
        // Selection might be empty or copy failed
        info!("get_selected_text: No new text selected (same as original)");
        return Err("No text selected or copy failed".to_string());
    }

    info!("get_selected_text: Success, returning selected text");
    Ok(selected_text)
}

/// Result of LLM role processing
#[derive(Debug, Clone)]
pub struct LLMRoleResult {
    pub text: String,
    pub model_name: String,
    pub role_name: String,
    pub selected_text: Option<String>,
    pub command_text: Option<String>,
}

/// Process voice command directly without role matching
/// If there's selected text, use transcription as command/instruction
async fn maybe_process_voice_command(
    settings: &AppSettings,
    transcription: &str,
    app_handle: &AppHandle,
) -> Option<LLMRoleResult> {
    info!(
        "maybe_process_voice_command: Starting with transcription='{}'",
        transcription
    );
    // Skip if LLM is disabled
    if !settings.llm_enabled {
        info!("LLM processing is disabled, skipping voice command");
        return None;
    }

    // Skip if transcription is empty
    if transcription.trim().is_empty() {
        info!("Empty transcription, skipping voice command");
        return None;
    }

    // Check if global LLM is configured first - if not, skip early
    if settings.llm_provider_id.is_none()
        || settings.llm_api_key.is_none()
        || settings.llm_model.is_none()
    {
        info!("maybe_process_voice_command: Global LLM not configured, skipping");
        return None;
    }

    info!("maybe_process_voice_command: Attempting to get selected text");
    // Try to get selected text with error handling
    // Note: get_selected_text is synchronous and uses Enigo which may block
    // We call it directly here since we're already in an async context
    // Wrap in catch_unwind to prevent panic from crashing the app
    let selected_text_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        get_selected_text(app_handle)
    }));

    let selected_text = match selected_text_result {
        Ok(Ok(text)) if !text.trim().is_empty() => {
            info!("Got selected text: {} chars", text.len());
            text
        }
        Ok(Ok(_)) => {
            info!("Selected text is empty or same as clipboard");
            return None;
        }
        Ok(Err(e)) => {
            info!(
                "Failed to get selected text: {} (this is normal if no text is selected)",
                e
            );
            return None;
        }
        Err(_) => {
            error!("get_selected_text panicked! This should not happen.");
            return None;
        }
    };

    info!(
        "Processing voice command: instruction='{}', selected_text length={}",
        transcription,
        selected_text.len()
    );

    // Check if global LLM is configured
    info!("Checking LLM configuration");
    let provider_id = match settings.llm_provider_id.as_ref() {
        Some(id) => {
            info!("Found provider_id: {}", id);
            id
        }
        None => {
            info!("No LLM provider configured");
            return None;
        }
    };

    let api_key = match settings.llm_api_key.as_ref() {
        Some(key) if !key.is_empty() => {
            info!("Found API key (length: {})", key.len());
            key.clone()
        }
        _ => {
            info!("No LLM API key configured");
            return None;
        }
    };

    let model = match settings.llm_model.as_ref() {
        Some(m) if !m.is_empty() => {
            info!("Found model: {}", m);
            m.clone()
        }
        _ => {
            info!("No LLM model configured");
            return None;
        }
    };

    info!("Looking for provider with id: {}", provider_id);
    let provider = match settings
        .post_process_providers
        .iter()
        .find(|p| &p.id == provider_id)
    {
        Some(p) => {
            info!("Found provider: {}", p.id);
            p
        }
        None => {
            warn!("LLM provider '{}' not found", provider_id);
            return None;
        }
    };

    // Build prompt: instruction as system prompt, selected text as user input
    info!("Building prompt");
    let prompt = format!(
        "指令: {}\n\n要处理的文本:\n{}",
        transcription, selected_text
    );
    info!("Prompt length: {}", prompt.len());

    info!("Sending voice command to LLM");

    // Call LLM with error handling
    info!("Calling send_chat_completion");
    match crate::llm_client::send_chat_completion(provider, api_key, &model, prompt).await {
        Ok(Some(result)) => {
            info!(
                "Voice command processing succeeded, result length: {}",
                result.len()
            );
            Some(LLMRoleResult {
                text: result,
                model_name: model,
                role_name: "语音指令".to_string(),
                selected_text: Some(selected_text),
                command_text: Some(transcription.to_string()),
            })
        }
        Ok(None) => {
            warn!("LLM returned empty result for voice command");
            None
        }
        Err(e) => {
            error!("Failed to process voice command with LLM: {}", e);
            None
        }
    }
}

/// Process transcription with LLM role if matched
async fn maybe_process_with_role(
    settings: &AppSettings,
    transcription: &str,
    app_handle: &AppHandle,
) -> Option<LLMRoleResult> {
    // Skip if LLM is disabled
    if !settings.llm_enabled {
        debug!("LLM processing is disabled, skipping role matching");
        return None;
    }

    debug!(
        "Checking LLM role match for transcription: '{}'",
        transcription
    );
    
    // Check if a role is selected - if so, use it directly
    let (role, input_text) = if let Some(selected_role_id) = &settings.selected_llm_role_id {
        // Use selected role
        if let Some(role) = settings.llm_roles.iter().find(|r| r.id == *selected_role_id) {
            info!("Using selected LLM role: '{}'", role.name);
            (role, transcription.to_string())
        } else {
            warn!("Selected role ID '{}' not found in roles list", selected_role_id);
            return None;
        }
    } else {
        // Fall back to role name matching
        match match_llm_role(transcription, &settings.llm_roles) {
            Some((role, text)) => {
                debug!(
                    "Matched role '{}' with remaining text: '{}'",
                    role.name, text
                );
                (role, text)
            }
            None => {
                debug!("No LLM role matched");
                return None;
            }
        }
    };

    debug!("Matched LLM role: {} with input: {}", role.name, input_text);

    // Keep a copy of original input text for later use in prompt formatting
    let original_input = input_text.clone();

    // If role enables read_selection and input is empty or contains instruction keywords
    let (actual_input, selected_text_opt) = if role.enable_read_selection {
        let instruction_keywords = [
            "翻译一下",
            "翻译",
            "translate",
            "润色一下",
            "润色",
            "polish",
        ];
        let should_read_selection = input_text.is_empty()
            || instruction_keywords
                .iter()
                .any(|kw| input_text.contains(kw));

        if should_read_selection {
            match get_selected_text(app_handle) {
                Ok(selected) => {
                    debug!("Got selected text: {} (command: {})", selected, input_text);
                    (selected.clone(), Some(selected))
                }
                Err(e) => {
                    warn!("Failed to get selected text: {}, using transcription", e);
                    (input_text, None)
                }
            }
        } else {
            (input_text, None)
        }
    } else {
        (input_text, None)
    };

    if actual_input.is_empty() {
        debug!("No input text for role processing");
        return None;
    }

    // Extract system prompt from role's template (everything before ${input})
    // and user content (the actual input text)
    //
    // Format matches OpenAI-compatible API:
    // - system_prompt: 提示词（角色指令），放在 role="system" 的 content 中
    // - user_content: 转录文本（用户输入），放在 role="user" 的 content 中
    let role_prompt = &role.prompt;
    info!("Role prompt template: '{}'", role_prompt);
    info!("Actual input text: '{}'", actual_input);

    let (system_prompt, user_content) = if role_prompt.contains("${input}") {
        // Split on ${input}, take the part before it as system prompt
        // Example: "请优化以下文本：\n\n${input}"
        // -> system = "请优化以下文本：\n\n"
        // -> user = actual_input (转录文本)
        let parts: Vec<&str> = role_prompt.split("${input}").collect();
        let system = parts[0].trim().to_string();
        // For voice commands with selected text, we need to include the command instruction
        let user = if selected_text_opt.is_some() && !original_input.is_empty() {
            format!(
                "指令: {}\n\n要处理的文本:\n{}",
                original_input, actual_input
            )
        } else {
            actual_input.clone()
        };
        info!(
            "Prompt contains ${{input}}: system='{}', user='{}'",
            system, user
        );
        (Some(system), user)
    } else {
        // If no ${input} placeholder, use entire prompt as system message and input as user message
        let user = if selected_text_opt.is_some() && !original_input.is_empty() {
            format!(
                "指令: {}\n\n要处理的文本:\n{}",
                original_input, actual_input
            )
        } else {
            actual_input.clone()
        };
        info!(
            "Prompt does NOT contain ${{input}}: system='{}', user='{}'",
            role_prompt, user
        );
        (Some(role_prompt.clone()), user)
    };

    info!("Final LLM request format:");
    info!(
        "  System message (role='system', content='{}'): '{}'",
        system_prompt
            .as_ref()
            .map(|s| if s.is_empty() { "(empty)" } else { "提示词" })
            .unwrap_or("(none)"),
        system_prompt.as_ref().unwrap_or(&String::new())
    );
    info!(
        "  User message (role='user', content='{}'): '{}'",
        if user_content.is_empty() {
            "(empty)"
        } else {
            "转录文本"
        },
        user_content
    );

    // Get provider - use role-specific LLM config, then global LLM config, then fall back to post_process_provider_id
    let (provider, api_key, model, custom_api_url) = if let Some(config_id) = &role.llm_config_id {
        // Use role-specific LLM config
        if let Some(config) = settings.llm_configs.iter().find(|c| &c.id == config_id) {
            if config.provider_id == "custom" {
                // Custom provider - use custom API URL if provided
                let custom_provider = PostProcessProvider {
                    id: "custom".to_string(),
                    label: "Custom".to_string(),
                    base_url: config.api_url.clone().unwrap_or_default(),
                    allow_base_url_edit: true,
                    models_endpoint: None,
                };
                (
                    custom_provider,
                    config.api_key.clone(),
                    config.model.clone(),
                    config.api_url.clone(),
                )
            } else if let Some(provider) = settings
                .post_process_providers
                .iter()
                .find(|p| &p.id == &config.provider_id)
            {
                // Use custom API URL if provided, otherwise use provider's base_url
                let api_url = config
                    .api_url
                    .clone()
                    .or_else(|| Some(provider.base_url.clone()));
                (
                    provider.clone(),
                    config.api_key.clone(),
                    config.model.clone(),
                    api_url,
                )
            } else {
                warn!(
                    "LLM config '{}' references unknown provider '{}'",
                    config.name, config.provider_id
                );
                return None;
            }
        } else {
            warn!(
                "LLM config '{}' not found for role '{}'",
                config_id, role.name
            );
            return None;
        }
    } else if let Some(selected_config_id) = &settings.selected_llm_config_id {
        // Use globally selected LLM config
        if let Some(config) = settings
            .llm_configs
            .iter()
            .find(|c| &c.id == selected_config_id)
        {
            if config.provider_id == "custom" {
                // Custom provider - use custom API URL if provided
                let custom_provider = PostProcessProvider {
                    id: "custom".to_string(),
                    label: "Custom".to_string(),
                    base_url: config.api_url.clone().unwrap_or_default(),
                    allow_base_url_edit: true,
                    models_endpoint: None,
                };
                (
                    custom_provider,
                    config.api_key.clone(),
                    config.model.clone(),
                    config.api_url.clone(),
                )
            } else if let Some(provider) = settings
                .post_process_providers
                .iter()
                .find(|p| &p.id == &config.provider_id)
            {
                // Use custom API URL if provided, otherwise use provider's base_url
                let api_url = config
                    .api_url
                    .clone()
                    .or_else(|| Some(provider.base_url.clone()));
                (
                    provider.clone(),
                    config.api_key.clone(),
                    config.model.clone(),
                    api_url,
                )
            } else {
                warn!(
                    "Global LLM config '{}' references unknown provider '{}'",
                    config.name, config.provider_id
                );
                return None;
            }
        } else {
            warn!("Global LLM config '{}' not found", selected_config_id);
            return None;
        }
    } else {
        // Fall back to old provider-based config
        let provider_id = role
            .provider_id
            .as_ref()
            .filter(|id| !id.is_empty())
            .or_else(|| settings.llm_provider_id.as_ref())
            .unwrap_or(&settings.post_process_provider_id);

        let provider = settings
            .post_process_providers
            .iter()
            .find(|p| &p.id == provider_id)?
            .clone();

        let api_key = role
            .api_key
            .as_ref()
            .filter(|k| !k.is_empty())
            .or_else(|| settings.llm_api_key.as_ref())
            .or_else(|| settings.post_process_api_keys.get(provider_id))
            .cloned()
            .unwrap_or_default();

        let model = role
            .model
            .as_ref()
            .filter(|m| !m.is_empty())
            .or_else(|| settings.llm_model.as_ref())
            .or_else(|| settings.post_process_models.get(provider_id))
            .cloned()
            .unwrap_or_default();

        (provider, api_key, model, None)
    };

    if api_key.is_empty() || model.is_empty() {
        warn!(
            "Role {} has no API key or model configured (provider: {})",
            role.name, provider.id
        );
        return None;
    }

    // Call LLM with separate system and user messages
    match crate::llm_client::send_chat_completion_with_messages_and_url(
        &provider,
        api_key,
        &model,
        system_prompt,
        user_content,
        custom_api_url,
    )
    .await
    {
        Ok(Some(result)) => {
            info!(
                "LLM returned result (length: {}): '{}'",
                result.len(),
                result
            );
            debug!("Role processing succeeded: {} -> {}", actual_input, result);

            // Clean up the result: remove any curl commands or HTTP request examples that LLM might have included
            let cleaned_result = result.trim();
            // Check if result contains curl command and extract only the actual content before it
            let final_result = if cleaned_result.contains("curl") {
                // Try to extract content before curl command
                if let Some(curl_pos) = cleaned_result.find("curl") {
                    let before_curl = cleaned_result.char_indices()
                        .take_while(|(idx, _)| *idx < curl_pos)
                        .map(|(_, ch)| ch)
                        .collect::<String>()
                        .trim()
                        .to_string();
                    warn!(
                        "LLM response contained curl command. Extracted content before curl: '{}'",
                        before_curl
                    );
                    before_curl.to_string()
                } else {
                    cleaned_result.to_string()
                }
            } else {
                cleaned_result.to_string()
            };

            info!("Final cleaned result: '{}'", final_result);
            let cmd_text = if selected_text_opt.is_some() {
                Some(original_input.to_string())
            } else {
                None
            };
            debug!(
                "Voice command info: selected_text={:?}, command_text={:?}",
                selected_text_opt, cmd_text
            );
            Some(LLMRoleResult {
                text: final_result,
                model_name: model,
                role_name: role.name.clone(),
                selected_text: selected_text_opt.clone(),
                command_text: cmd_text,
            })
        }
        Ok(None) => {
            warn!("LLM returned empty result for role processing");
            None
        }
        Err(e) => {
            error!("LLM role processing failed: {}", e);
            None
        }
    }
}

async fn maybe_post_process_transcription(
    settings: &AppSettings,
    transcription: &str,
) -> Option<String> {
    if !settings.post_process_enabled {
        return None;
    }

    let provider = match settings.active_post_process_provider().cloned() {
        Some(provider) => provider,
        None => {
            debug!("Post-processing enabled but no provider is selected");
            return None;
        }
    };

    let model = settings
        .post_process_models
        .get(&provider.id)
        .cloned()
        .unwrap_or_default();

    if model.trim().is_empty() {
        debug!(
            "Post-processing skipped because provider '{}' has no model configured",
            provider.id
        );
        return None;
    }

    let selected_prompt_id = match &settings.post_process_selected_prompt_id {
        Some(id) => id.clone(),
        None => {
            debug!("Post-processing skipped because no prompt is selected");
            return None;
        }
    };

    let prompt = match settings
        .post_process_prompts
        .iter()
        .find(|prompt| prompt.id == selected_prompt_id)
    {
        Some(prompt) => prompt.prompt.clone(),
        None => {
            debug!(
                "Post-processing skipped because prompt '{}' was not found",
                selected_prompt_id
            );
            return None;
        }
    };

    if prompt.trim().is_empty() {
        debug!("Post-processing skipped because the selected prompt is empty");
        return None;
    }

    debug!(
        "Starting LLM post-processing with provider '{}' (model: {})",
        provider.id, model
    );

    // Replace ${output} variable in the prompt with the actual text
    let processed_prompt = prompt.replace("${output}", transcription);
    debug!("Processed prompt length: {} chars", processed_prompt.len());

    if provider.id == APPLE_INTELLIGENCE_PROVIDER_ID {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            if !apple_intelligence::check_apple_intelligence_availability() {
                debug!("Apple Intelligence selected but not currently available on this device");
                return None;
            }

            let token_limit = model.trim().parse::<i32>().unwrap_or(0);
            return match apple_intelligence::process_text(&processed_prompt, token_limit) {
                Ok(result) => {
                    if result.trim().is_empty() {
                        debug!("Apple Intelligence returned an empty response");
                        None
                    } else {
                        debug!(
                            "Apple Intelligence post-processing succeeded. Output length: {} chars",
                            result.len()
                        );
                        Some(result)
                    }
                }
                Err(err) => {
                    error!("Apple Intelligence post-processing failed: {}", err);
                    None
                }
            };
        }

        #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
        {
            debug!("Apple Intelligence provider selected on unsupported platform");
            return None;
        }
    }

    let api_key = settings
        .post_process_api_keys
        .get(&provider.id)
        .cloned()
        .unwrap_or_default();

    // Send the chat completion request
    match crate::llm_client::send_chat_completion(&provider, api_key, &model, processed_prompt)
        .await
    {
        Ok(Some(content)) => {
            debug!(
                "LLM post-processing succeeded for provider '{}'. Output length: {} chars",
                provider.id,
                content.len()
            );
            Some(content)
        }
        Ok(None) => {
            error!("LLM API response has no content");
            None
        }
        Err(e) => {
            error!(
                "LLM post-processing failed for provider '{}': {}. Falling back to original transcription.",
                provider.id,
                e
            );
            None
        }
    }
}

async fn maybe_convert_chinese_variant(
    settings: &AppSettings,
    transcription: &str,
) -> Option<String> {
    // Check if language is set to Simplified or Traditional Chinese
    let is_simplified = settings.selected_language == "zh-Hans";
    let is_traditional = settings.selected_language == "zh-Hant";

    if !is_simplified && !is_traditional {
        debug!("selected_language is not Simplified or Traditional Chinese; skipping translation");
        return None;
    }

    debug!(
        "Starting Chinese translation using OpenCC for language: {}",
        settings.selected_language
    );

    // Use OpenCC to convert based on selected language
    let config = if is_simplified {
        // Convert Traditional Chinese to Simplified Chinese
        BuiltinConfig::Tw2sp
    } else {
        // Convert Simplified Chinese to Traditional Chinese
        BuiltinConfig::S2twp
    };

    match OpenCC::from_config(config) {
        Ok(converter) => {
            let converted = converter.convert(transcription);
            debug!(
                "OpenCC translation completed. Input length: {}, Output length: {}",
                transcription.len(),
                converted.len()
            );
            Some(converted)
        }
        Err(e) => {
            error!("Failed to initialize OpenCC converter: {}. Falling back to original transcription.", e);
            None
        }
    }
}

impl ShortcutAction for TranscribeAction {
    fn start(&self, app: &AppHandle, binding_id: &str, _shortcut_str: &str) {
        let start_time = Instant::now();
        debug!("TranscribeAction::start called for binding: {}", binding_id);

        // Load model in the background
        let tm = app.state::<Arc<TranscriptionManager>>();
        tm.initiate_model_load();


        let binding_id = binding_id.to_string();
        change_tray_icon(app, TrayIconState::Recording);
        show_recording_overlay(app);

        let rm = app.state::<Arc<AudioRecordingManager>>();

        // Get the microphone mode to determine audio feedback timing
        let settings = get_settings(app);
        let is_always_on = settings.always_on_microphone;
        debug!("Microphone mode - always_on: {}", is_always_on);

        let mut recording_started = false;
        if is_always_on {
            // Always-on mode: Play audio feedback immediately, then apply mute after sound finishes
            debug!("Always-on mode: Playing audio feedback immediately");
            let rm_clone = Arc::clone(&rm);
            let app_clone = app.clone();
            // The blocking helper exits immediately if audio feedback is disabled,
            // so we can always reuse this thread to ensure mute happens right after playback.
            std::thread::spawn(move || {
                play_feedback_sound_blocking(&app_clone, SoundType::Start);
                rm_clone.apply_mute();
            });

            recording_started = rm.try_start_recording(&binding_id);
            debug!("Recording started: {}", recording_started);
        } else {
            // On-demand mode: Start recording first, then play audio feedback, then apply mute
            // This allows the microphone to be activated before playing the sound
            debug!("On-demand mode: Starting recording first, then audio feedback");
            let recording_start_time = Instant::now();
            if rm.try_start_recording(&binding_id) {
                recording_started = true;
                debug!("Recording started in {:?}", recording_start_time.elapsed());
                // Small delay to ensure microphone stream is active
                let app_clone = app.clone();
                let rm_clone = Arc::clone(&rm);
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    debug!("Handling delayed audio feedback/mute sequence");
                    // Helper handles disabled audio feedback by returning early, so we reuse it
                    // to keep mute sequencing consistent in every mode.
                    play_feedback_sound_blocking(&app_clone, SoundType::Start);
                    rm_clone.apply_mute();
                });
            } else {
                debug!("Failed to start recording");
            }
        }

        if recording_started {
            // Dynamically register the cancel shortcut in a separate task to avoid deadlock
            shortcut::register_cancel_shortcut(app);
        }

        debug!(
            "TranscribeAction::start completed in {:?}",
            start_time.elapsed()
        );
    }

    fn stop(&self, app: &AppHandle, binding_id: &str, _shortcut_str: &str) {
        // Unregister the cancel shortcut when transcription stops
        shortcut::unregister_cancel_shortcut(app);

        let stop_time = Instant::now();
        debug!("TranscribeAction::stop called for binding: {}", binding_id);

        let ah = app.clone();
        let rm = Arc::clone(&app.state::<Arc<AudioRecordingManager>>());
        let tm = Arc::clone(&app.state::<Arc<TranscriptionManager>>());
        let hm = Arc::clone(&app.state::<Arc<HistoryManager>>());

        change_tray_icon(app, TrayIconState::Transcribing);
        show_transcribing_overlay(app);

        // Unmute before playing audio feedback so the stop sound is audible
        rm.remove_mute();

        // Play audio feedback for recording stop
        play_feedback_sound(app, SoundType::Stop);

        let binding_id = binding_id.to_string(); // Clone binding_id for the async task

        tauri::async_runtime::spawn(async move {
            let binding_id = binding_id.clone(); // Clone for the inner async task
            debug!(
                "Starting async transcription task for binding: {}",
                binding_id
            );

            // Reset cancellation flag at the start of transcription
            tm.reset_cancellation_flag();

            let stop_recording_time = Instant::now();

            // Get samples and transcribe normally
            let (transcription_result, samples_for_history) = if let Some(samples) = rm.stop_recording(&binding_id) {
                debug!(
                    "Recording stopped and samples retrieved in {:?}, sample count: {}",
                    stop_recording_time.elapsed(),
                    samples.len()
                );

                let transcription_time = Instant::now();
                let samples_clone = samples.clone(); // Clone for history saving
                match tm.transcribe(samples.clone()) {
                    Ok(transcription_result) => {
                        // Check if transcription was cancelled after completion
                        if tm.is_cancelled() {
                            debug!("Transcription was cancelled, aborting processing");
                            tm.reset_cancellation_flag();
                            return; // Exit early without processing result
                        }
                        debug!(
                            "Transcription completed in {:?}: '{}'",
                            transcription_time.elapsed(),
                            transcription_result.final_text
                        );
                        (Ok(transcription_result), samples_clone)
                    }
                    Err(e) => {
                        // Check if error is due to cancellation
                        if e.to_string().contains("cancelled") {
                            debug!("Transcription was cancelled by user");
                            return; // Exit early without error handling
                        }
                        error!("Transcription failed: {}", e);
                        (Err(e), samples_clone)
                    }
                }
            } else {
                (Err(anyhow::anyhow!("Failed to stop recording")), Vec::new())
            };

            info!("Processing transcription result");
            match transcription_result {
                Ok(transcription_data) => {
                    info!(
                        "Transcription successful, final_text: '{}'",
                        transcription_data.final_text
                    );
                    if !transcription_data.final_text.is_empty() {
                        info!("Final text is not empty, proceeding with processing");
                        let settings = get_settings(&ah);
                        info!("Got settings");
                        // Get original text (before ITN and hot rules)
                        let original_transcription = transcription_data.original_text.clone();
                        let transcription_after_hot_rules = transcription_data.final_text.clone();

                        info!(
                            "Prepared transcription data: original='{}', after_hot_rules='{}'",
                            original_transcription, transcription_after_hot_rules
                        );

                        let mut final_text = String::new();
                        let mut post_processed_text: Option<String> = None;
                        let mut post_process_prompt: Option<String> = None;
                        let mut llm_model_name: Option<String> = None;
                        let mut llm_role_name: Option<String> = None;
                        let mut voice_command_info: Option<(String, String)> = None; // (command_text, selected_text)
                        let mut should_apply_itn = false; // Flag to determine if we should apply ITN

                        // Step 1: Check if there's selected text (voice command)
                        info!("Step 1: Checking for selected text (voice command)");
                        if let Some(role_result) =
                            maybe_process_voice_command(&settings, &original_transcription, &ah)
                                .await
                        {
                            info!("Voice command processed successfully (has selected text)");
                            final_text = role_result.text.clone();
                            post_processed_text = Some(role_result.text);
                            llm_model_name = Some(role_result.model_name);
                            llm_role_name = Some(role_result.role_name);
                            // Voice command always has both selected text and command text
                            if let (Some(cmd), Some(sel)) =
                                (&role_result.command_text, &role_result.selected_text)
                            {
                                voice_command_info = Some((cmd.clone(), sel.clone()));
                                debug!("Voice command processed: cmd='{}', sel='{}'", cmd, sel);
                            }
                            // Voice command: don't apply ITN
                            should_apply_itn = false;
                        }
                        // Step 2: If no selected text, check if role matches
                        else {
                            info!("Step 2: No selected text, checking LLM role matching");
                            if let Some(role_result) =
                                maybe_process_with_role(&settings, &original_transcription, &ah)
                                    .await
                            {
                                info!("LLM role matched successfully");
                                final_text = role_result.text.clone();
                                post_processed_text = Some(role_result.text);
                                llm_model_name = Some(role_result.model_name);
                                llm_role_name = Some(role_result.role_name);
                                // Check if this is a voice command with selected text
                                info!(
                                    "Role result: cmd={:?}, sel={:?}",
                                    role_result.command_text, role_result.selected_text
                                );
                                if let (Some(cmd), Some(sel)) =
                                    (&role_result.command_text, &role_result.selected_text)
                                {
                                    voice_command_info = Some((cmd.clone(), sel.clone()));
                                    info!("Set voice_command_info: cmd='{}', sel='{}'", cmd, sel);
                                } else {
                                    info!("Not a voice command (missing cmd or sel)");
                                }
                                // Role matched: don't apply ITN, use LLM result directly
                                should_apply_itn = false;
                            }
                            // Step 3: If no role matched, apply punctuation (if enabled), then ITN (if enabled) and then other processing
                            else {
                                info!("Step 3: No role matched, checking if punctuation and ITN should be applied");
                                info!("Punctuation enabled: {}, model path: {:?}", 
                                    settings.punctuation_enabled, 
                                    settings.punctuation_model_path);
                                
                                // Step 3a: Apply punctuation model if enabled (before ITN)
                                let text_after_punctuation = if settings.punctuation_enabled {
                                    info!("Punctuation is enabled, attempting to apply");
                                    // Ensure punctuation model is loaded
                                    if let Some(punctuation_manager) = ah.try_state::<Arc<PunctuationManager>>() {
                                        info!("PunctuationManager found in app state");
                                        if let Err(e) = punctuation_manager.ensure_loaded(&settings) {
                                            warn!("Failed to ensure punctuation model is loaded: {}", e);
                                        } else {
                                            info!("Punctuation model ensure_loaded completed successfully");
                                        }
                                        
                                        match punctuation_manager.add_punctuation(&transcription_after_hot_rules) {
                                            Ok(punctuated_text) => {
                                                info!(
                                                    "Punctuation applied: '{}' -> '{}'",
                                                    transcription_after_hot_rules, punctuated_text
                                                );
                                                punctuated_text
                                            }
                                            Err(e) => {
                                                warn!("Failed to add punctuation: {}, using original text", e);
                                                transcription_after_hot_rules.clone()
                                            }
                                        }
                                    } else {
                                        warn!("PunctuationManager not found in app state");
                                        transcription_after_hot_rules.clone()
                                    }
                                } else {
                                    info!("Punctuation is disabled, skipping");
                                    transcription_after_hot_rules.clone()
                                };
                                
                                // Step 3b: Apply ITN if enabled (on text after punctuation)
                                let text_for_processing = if settings.itn_enabled {
                                    let itn_result = apply_itn(&text_after_punctuation);
                                    info!(
                                        "ITN applied: '{}' -> '{}'",
                                        text_after_punctuation, itn_result
                                    );
                                    should_apply_itn = true;
                                    itn_result
                                } else {
                                    text_after_punctuation
                                };

                                final_text = text_for_processing.clone();

                                // Then check if Chinese variant conversion is needed
                                if let Some(converted_text) =
                                    maybe_convert_chinese_variant(&settings, &text_for_processing)
                                        .await
                                {
                                    info!("Chinese variant conversion applied");
                                    final_text = converted_text.clone();
                                    post_processed_text = Some(converted_text);
                                }
                                // Then apply regular post-processing if enabled
                                else {
                                    info!("No Chinese variant conversion, trying regular post-processing");
                                    if let Some(processed_text) = maybe_post_process_transcription(
                                        &settings,
                                        &text_for_processing,
                                    )
                                    .await
                                    {
                                        info!("Regular post-processing applied");
                                        final_text = processed_text.clone();
                                        post_processed_text = Some(processed_text);

                                        // Get the prompt that was used
                                        if let Some(prompt_id) =
                                            &settings.post_process_selected_prompt_id
                                        {
                                            if let Some(prompt) = settings
                                                .post_process_prompts
                                                .iter()
                                                .find(|p| &p.id == prompt_id)
                                            {
                                                post_process_prompt = Some(prompt.prompt.clone());
                                                info!("Post-process prompt set");
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        info!("Post-processing completed. final_text='{}', has_post_processed={}, has_role={}", 
                            final_text, 
                            post_processed_text.is_some(),
                            llm_role_name.is_some());

                        // Emit transcription result event for UI display
                        debug!("Preparing to emit transcription-result event");
                        let final_text_for_event = final_text.clone();
                        let ah_for_event = ah.clone();
                        tauri::async_runtime::spawn(async move {
                            debug!(
                                "Emitting transcription-result event with text length: {}",
                                final_text_for_event.len()
                            );
                            if let Err(e) =
                                ah_for_event.emit("transcription-result", final_text_for_event)
                            {
                                error!("Failed to emit transcription-result event: {}", e);
                            } else {
                                debug!("Successfully emitted transcription-result event");
                            }
                        });

                        // Save to history with post-processed text and prompt
                        // Get the model ID/name that was actually used for this transcription
                        debug!("Preparing to save to history");
                        // Use model_name from TranscriptionResult (which includes API model name or local model_id)
                        let model_id_used = transcription_data.model_name.clone();
                        let is_api_model = transcription_data.is_api;
                        let hm_clone = Arc::clone(&hm);
                        // Save original transcription (before ITN) to history
                        let transcription_for_history = original_transcription.clone();
                        let samples_for_history_clone = samples_for_history.clone();
                        let llm_model_for_history = llm_model_name.clone();
                        let llm_role_for_history = llm_role_name.clone();
                        let should_apply_itn_for_history = should_apply_itn;
                        let voice_command_info_for_history = voice_command_info.clone();
                        let post_processed_text_for_history = post_processed_text.clone();
                        let post_process_prompt_for_history = post_process_prompt.clone();
                        // Calculate ITN text if ITN was applied (before moving into async block)
                        let itn_text_for_history = if should_apply_itn_for_history {
                            Some(apply_itn(&transcription_for_history))
                        } else {
                            None
                        };
                        debug!("Spawning history save task");
                        tauri::async_runtime::spawn(async move {
                            // Check if this is a voice command (has selected text and command text)
                            if let Some((command_text, selected_text)) =
                                voice_command_info_for_history
                            {
                                // Save as voice command
                                let output_text = post_processed_text_for_history
                                    .unwrap_or(transcription_for_history.clone());
                                debug!(
                                    "Saving voice command: cmd='{}', selected='{}', output='{}'",
                                    command_text, selected_text, output_text
                                );
                                if let Err(e) = hm_clone
                                    .save_voice_command(
                                        samples_for_history_clone,
                                        command_text,
                                        selected_text,
                                        output_text,
                                        model_id_used,
                                        llm_model_for_history,
                                    )
                                    .await
                                {
                                    error!("Failed to save voice command to history: {}", e);
                                }
                            } else {
                                debug!("Saving regular transcription (not voice command)");
                                // Save as regular transcription
                                if let Err(e) = hm_clone
                                    .save_transcription(
                                        samples_for_history_clone,
                                        transcription_for_history,
                                        post_processed_text_for_history,
                                        post_process_prompt_for_history,
                                        model_id_used,
                                        llm_model_for_history,
                                        llm_role_for_history,
                                        itn_text_for_history,
                                    )
                                    .await
                                {
                                    error!("Failed to save transcription to history: {}", e);
                                }
                            }
                        });

                        // Paste the final text (either processed or original)
                        debug!("Preparing to paste text, length: {}", final_text.len());
                        let final_text_for_paste = final_text.clone();
                        let ah_clone = ah.clone();
                        let paste_time = Instant::now();
                        debug!("Calling run_on_main_thread for paste");
                        ah.run_on_main_thread(move || {
                            debug!("Inside main thread, calling paste");
                            match utils::paste(final_text_for_paste, ah_clone.clone()) {
                                Ok(()) => {
                                    debug!("Text pasted successfully in {:?}", paste_time.elapsed())
                                }
                                Err(e) => error!("Failed to paste transcription: {}", e),
                            }
                            debug!("Paste complete, hiding overlay");
                            // Hide the overlay after transcription is complete
                            utils::hide_recording_overlay(&ah_clone);
                            change_tray_icon(&ah_clone, TrayIconState::Idle);
                            debug!("Overlay hidden and tray icon updated");
                        })
                        .unwrap_or_else(|e| {
                            error!("Failed to run paste on main thread: {:?}", e);
                            utils::hide_recording_overlay(&ah);
                            change_tray_icon(&ah, TrayIconState::Idle);
                        });
                        debug!("run_on_main_thread call completed");
                    } else {
                        utils::hide_recording_overlay(&ah);
                        change_tray_icon(&ah, TrayIconState::Idle);
                    }
                }
                Err(err) => {
                    debug!("Global Shortcut Transcription error: {}", err);
                    utils::hide_recording_overlay(&ah);
                    change_tray_icon(&ah, TrayIconState::Idle);
                }
            }

            // Clear toggle state now that transcription is complete
            if let Ok(mut states) = ah.state::<ManagedToggleState>().lock() {
                states.active_toggles.insert(binding_id, false);
            }
        });

        debug!(
            "TranscribeAction::stop completed in {:?}",
            stop_time.elapsed()
        );
    }
}

// Cancel Action
struct CancelAction;

impl ShortcutAction for CancelAction {
    fn start(&self, app: &AppHandle, _binding_id: &str, _shortcut_str: &str) {
        utils::cancel_current_operation(app);
    }

    fn stop(&self, _app: &AppHandle, _binding_id: &str, _shortcut_str: &str) {
        // Nothing to do on stop for cancel
    }
}

// Test Action
struct TestAction;

impl ShortcutAction for TestAction {
    fn start(&self, app: &AppHandle, binding_id: &str, shortcut_str: &str) {
        log::info!(
            "Shortcut ID '{}': Started - {} (App: {})", // Changed "Pressed" to "Started" for consistency
            binding_id,
            shortcut_str,
            app.package_info().name
        );
    }

    fn stop(&self, app: &AppHandle, binding_id: &str, shortcut_str: &str) {
        log::info!(
            "Shortcut ID '{}': Stopped - {} (App: {})", // Changed "Released" to "Stopped" for consistency
            binding_id,
            shortcut_str,
            app.package_info().name
        );
    }
}

// Static Action Map
pub static ACTION_MAP: Lazy<HashMap<String, Arc<dyn ShortcutAction>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(
        "transcribe".to_string(),
        Arc::new(TranscribeAction) as Arc<dyn ShortcutAction>,
    );
    map.insert(
        "cancel".to_string(),
        Arc::new(CancelAction) as Arc<dyn ShortcutAction>,
    );
    map.insert(
        "test".to_string(),
        Arc::new(TestAction) as Arc<dyn ShortcutAction>,
    );
    map
});
