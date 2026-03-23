use crate::settings::TranscriptionApiProvider;
use log::{debug, error, warn};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct OpenAITranscriptionResponse {
    text: String,
}

#[derive(Debug, Serialize)]
struct DashScopeMessage {
    role: String,
    content: Vec<DashScopeContent>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum DashScopeContent {
    Text { text: String },
    Audio { audio: String },
}

// DashScope native API request structures
#[derive(Debug, Serialize)]
struct DashScopeNativeRequest {
    model: String,
    input: DashScopeNativeInput,
    parameters: DashScopeNativeParameters,
}

#[derive(Debug, Serialize)]
struct DashScopeNativeInput {
    messages: Vec<DashScopeMessage>,
}

#[derive(Debug, Serialize)]
struct DashScopeNativeParameters {
    #[serde(rename = "result_format")]
    result_format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    asr_options: Option<DashScopeAsrOptions>,
}

#[derive(Debug, Serialize)]
struct DashScopeAsrOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_lid: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_itn: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum DashScopeResponse {
    Success { output: DashScopeOutput },
    Error { code: String, message: String },
}

#[derive(Debug, Deserialize)]
struct DashScopeOutput {
    choices: Vec<DashScopeChoice>,
}

#[derive(Debug, Deserialize)]
struct DashScopeChoice {
    message: DashScopeResponseMessage,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DashScopeResponseMessage {
    content: Vec<DashScopeResponseContent>,
}

#[derive(Debug, Deserialize)]
struct DashScopeResponseContent {
    text: String,
}

/// Build headers for API requests based on provider type
fn build_headers(provider: &TranscriptionApiProvider, api_key: &str) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();

    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    // Provider-specific auth headers
    if !api_key.is_empty() {
        if provider.id == "dashscope" {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", api_key))
                    .map_err(|e| format!("Invalid API key header value: {}", e))?,
            );
        } else if provider.id == "azure" {
            headers.insert(
                "Ocp-Apim-Subscription-Key",
                HeaderValue::from_str(api_key)
                    .map_err(|e| format!("Invalid API key header value: {}", e))?,
            );
        } else {
            // OpenAI and custom providers
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", api_key))
                    .map_err(|e| format!("Invalid authorization header value: {}", e))?,
            );
        }
    }

    Ok(headers)
}

/// Create an HTTP client with provider-specific headers (public for use in separate threads)
pub fn create_client_for_provider(
    provider: &TranscriptionApiProvider,
    api_key: &str,
) -> Result<reqwest::Client, String> {
    create_client(provider, api_key)
}

/// Create an HTTP client with provider-specific headers
fn create_client(
    provider: &TranscriptionApiProvider,
    api_key: &str,
) -> Result<reqwest::Client, String> {
    let headers = build_headers(provider, api_key).map_err(|e| e)?;

    // Try to get system proxy settings from environment
    let http_proxy = std::env::var("http_proxy")
        .ok()
        .or_else(|| std::env::var("HTTP_PROXY").ok());
    let https_proxy = std::env::var("https_proxy")
        .ok()
        .or_else(|| std::env::var("HTTPS_PROXY").ok());

    let mut builder = reqwest::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(120))
        .connect_timeout(std::time::Duration::from_secs(30));

    // Set proxy if available
    if let Some(proxy_url) = https_proxy.or(http_proxy) {
        if let Ok(proxy) =
            reqwest::Proxy::https(&proxy_url).or_else(|_| reqwest::Proxy::http(&proxy_url))
        {
            builder = builder.proxy(proxy);
        }
    }

    builder.build().map_err(|e| {
        let error_msg = format!("Failed to build HTTP client: {}", e);
        error!("[HTTP Client] Builder error details: {}", error_msg);
        error_msg
    })
}

/// Send a transcription request to an API provider
/// Returns the transcribed text or an error message
pub async fn transcribe_audio(
    provider: &TranscriptionApiProvider,
    api_key: String,
    model: &str,
    audio_data: Vec<u8>,
    language: Option<&str>,
    custom_api_url: Option<String>,
) -> Result<String, String> {
    let url = if let Some(custom_url) = custom_api_url {
        custom_url.trim_end_matches('/').to_string()
    } else {
        provider.base_url.trim_end_matches('/').to_string()
    };

    debug!("Sending transcription request to: {}", url);
    debug!(
        "Provider: {}, Model: {}, Audio size: {} bytes",
        provider.id,
        model,
        audio_data.len()
    );

    let client = create_client(provider, &api_key).map_err(|e| e)?;

    match provider.id.as_str() {
        "dashscope" => transcribe_dashscope(&client, &url, model, audio_data, language).await,
        "openai" | "custom" => transcribe_openai(&client, &url, model, audio_data, language).await,
        "azure" => transcribe_azure(&client, &url, model, audio_data, language).await,
        _ => Err(format!("Unsupported provider: {}", provider.id)),
    }
}

/// Send a transcription request using a pre-created HTTP client
/// This is useful when the client needs to be created in a different thread context
pub async fn transcribe_audio_with_client(
    client: &reqwest::Client,
    provider: &TranscriptionApiProvider,
    model: &str,
    audio_data: Vec<u8>,
    language: Option<&str>,
    custom_api_url: Option<String>,
) -> Result<String, String> {
    let base_url = if let Some(ref custom_url) = custom_api_url {
        let trimmed = custom_url.trim_end_matches('/').to_string();
        if trimmed.is_empty() {
            warn!("[Transcription API] Custom API URL is empty, falling back to provider base URL");
            provider.base_url.trim_end_matches('/').to_string()
        } else {
            trimmed
        }
    } else {
        provider.base_url.trim_end_matches('/').to_string()
    };

    if base_url.is_empty() {
        let error_msg = format!(
            "Base URL is empty for provider: {} (provider.base_url: {:?}, custom_api_url: {:?})",
            provider.id, provider.base_url, custom_api_url
        );
        error!("[Transcription API] {}", error_msg);
        return Err(error_msg);
    }

    debug!("Sending transcription request to base URL: {}", base_url);
    debug!(
        "Provider: {}, Model: {}, Audio size: {} bytes",
        provider.id,
        model,
        audio_data.len()
    );

    // Validate URL format
    if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
        let error_msg = format!(
            "Invalid base URL format (must start with http:// or https://): {} (provider: {})",
            base_url, provider.id
        );
        error!("[Transcription API] {}", error_msg);
        return Err(error_msg);
    }

    match provider.id.as_str() {
        "dashscope" => transcribe_dashscope(client, &base_url, model, audio_data, language).await,
        "openai" | "custom" => {
            transcribe_openai(client, &base_url, model, audio_data, language).await
        }
        "azure" => transcribe_azure(client, &base_url, model, audio_data, language).await,
        _ => Err(format!("Unsupported provider: {}", provider.id)),
    }
}

/// Transcribe audio using DashScope (Alibaba Cloud) native API
/// Uses MultiModalConversation API endpoint
async fn transcribe_dashscope(
    client: &reqwest::Client,
    base_url: &str,
    model: &str,
    audio_data: Vec<u8>,
    language: Option<&str>,
) -> Result<String, String> {
    // DashScope MultiModalConversation API endpoint
    // Ensure base_url is a full URL (not relative)
    let base_url_trimmed = base_url.trim_end_matches('/');
    if base_url_trimmed.is_empty() {
        return Err(format!("Base URL is empty in transcribe_dashscope"));
    }
    if !base_url_trimmed.starts_with("http://") && !base_url_trimmed.starts_with("https://") {
        return Err(format!(
            "Invalid base URL (must start with http:// or https://): {}",
            base_url_trimmed
        ));
    }
    let url = format!(
        "{}/services/aigc/multimodal-generation/generation",
        base_url_trimmed
    );

    debug!(
        "DashScope transcribe full URL: {} (base_url: {})",
        url, base_url_trimmed
    );

    // Convert PCM to WAV format (DashScope expects WAV format with headers)
    let wav_data = convert_pcm_to_wav(audio_data)?;

    // Convert WAV to base64
    let audio_base64 =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &wav_data);

    // Create data URL for audio (DashScope supports data URLs)
    let audio_url = format!("data:audio/wav;base64,{}", audio_base64);

    // According to qwen3_asr.py, system_prompt defaults to empty string ""
    let system_prompt = "".to_string();
    let messages = vec![
        DashScopeMessage {
            role: "system".to_string(),
            content: vec![DashScopeContent::Text {
                text: system_prompt,
            }],
        },
        DashScopeMessage {
            role: "user".to_string(),
            content: vec![DashScopeContent::Audio { audio: audio_url }],
        },
    ];

    let asr_options = Some(DashScopeAsrOptions {
        language: language.map(|l| l.to_string()),
        enable_lid: Some(true), // Enable language identification as per Python script
        enable_itn: Some(false),
    });

    // DashScope native API format: { model, input: { messages }, parameters: { result_format, asr_options } }
    #[derive(Debug, Serialize)]
    struct DashScopeNativeRequest {
        model: String,
        input: DashScopeNativeInput,
        parameters: DashScopeNativeParameters,
    }

    #[derive(Debug, Serialize)]
    struct DashScopeNativeInput {
        messages: Vec<DashScopeMessage>,
    }

    #[derive(Debug, Serialize)]
    struct DashScopeNativeParameters {
        #[serde(rename = "result_format")]
        result_format: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        asr_options: Option<DashScopeAsrOptions>,
    }

    let request_body = DashScopeNativeRequest {
        model: model.to_string(),
        input: DashScopeNativeInput { messages },
        parameters: DashScopeNativeParameters {
            result_format: "message".to_string(),
            asr_options,
        },
    };

    // Log request for debugging
    let request_json = serde_json::to_string_pretty(&request_body)
        .unwrap_or_else(|_| "Failed to serialize request".to_string());
    debug!("DashScope transcribe request URL: {}", url);
    debug!("DashScope transcribe request body: {}", request_json);

    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            let error_msg = format!(
                "HTTP request failed: {} (URL: {}, error type: {:?})",
                e, url, e
            );
            error!("[DashScope Transcribe] {}", error_msg);
            error_msg
        })?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error response".to_string());
        return Err(format!(
            "API request failed with status {}: {}",
            status, error_text
        ));
    }

    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    debug!("DashScope transcribe response: {}", response_text);

    // Parse DashScope native API response
    // DashScope may return different response formats, so we need to handle both success and error cases
    #[derive(Debug, Deserialize)]
    struct DashScopeNativeResponse {
        #[serde(default)]
        output: Option<DashScopeOutput>,
        #[serde(default)]
        code: Option<String>,
        #[serde(default)]
        message: Option<String>,
        #[serde(default)]
        request_id: Option<String>,
    }

    let response_data: DashScopeNativeResponse =
        serde_json::from_str(&response_text).map_err(|e| {
            let error_msg = format!(
                "Failed to parse API response: {}\n\nResponse text: {}",
                e, response_text
            );
            error!("[DashScope Transcribe] Parse error: {}", error_msg);
            error_msg
        })?;

    // Check for errors first
    if let Some(code) = &response_data.code {
        if !code.is_empty() {
            let error_msg = format!(
                "DashScope API error [{}]: {}\nRequest ID: {:?}",
                code,
                response_data.message.as_deref().unwrap_or("Unknown error"),
                response_data.request_id
            );
            error!("[DashScope Transcribe] API error: {}", error_msg);
            return Err(error_msg);
        }
    }

    // Check if output exists
    if let Some(output) = response_data.output {
        if let Some(choice) = output.choices.first() {
            // Check all content items, not just first
            // Sometimes content might be in a different position
            for content_item in &choice.message.content {
                if !content_item.text.is_empty() {
                    return Ok(content_item.text.clone());
                }
            }

            // Empty content array - API processed successfully but no text detected
            // Return empty string instead of error
            Ok(String::new())
        } else {
            Err(format!(
                "No choices in response. Response: {}",
                response_text
            ))
        }
    } else {
        Err(format!(
            "No output in response. Response: {}",
            response_text
        ))
    }
}

/// Transcribe audio using OpenAI Whisper API (or compatible)
async fn transcribe_openai(
    client: &reqwest::Client,
    base_url: &str,
    model: &str,
    audio_data: Vec<u8>,
    language: Option<&str>,
) -> Result<String, String> {
    // Convert PCM to WAV format
    let wav_data = convert_pcm_to_wav(audio_data)?;

    let mut form = multipart::Form::new()
        .text("model", model.to_string())
        .part(
            "file",
            multipart::Part::bytes(wav_data)
                .file_name("audio.wav")
                .mime_str("audio/wav")
                .map_err(|e| format!("Failed to create audio part: {}", e))?,
        );

    // Add language if specified
    if let Some(lang) = language {
        form = form.text("language", lang.to_string());
    }

    let response = client
        .post(base_url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error response".to_string());
        return Err(format!(
            "API request failed with status {}: {}",
            status, error_text
        ));
    }

    let transcription: OpenAITranscriptionResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse API response: {}", e))?;

    Ok(transcription.text)
}

/// Transcribe audio using Azure Speech API
async fn transcribe_azure(
    client: &reqwest::Client,
    base_url: &str,
    _model: &str,
    audio_data: Vec<u8>,
    language: Option<&str>,
) -> Result<String, String> {
    // Azure Speech uses a different format - needs WAV with specific headers
    let wav_data = convert_pcm_to_wav(audio_data)?;

    // Azure requires language in the URL parameter
    let url = if let Some(lang) = language {
        format!("{}?language={}", base_url, lang)
    } else {
        base_url.to_string()
    };

    let response = client
        .post(&url)
        .header(CONTENT_TYPE, "audio/wav")
        .body(wav_data)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error response".to_string());
        return Err(format!(
            "API request failed with status {}: {}",
            status, error_text
        ));
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse API response: {}", e))?;

    // Azure response format: { "RecognitionStatus": "Success", "DisplayText": "..." }
    if let Some(text) = response_json.get("DisplayText").and_then(|v| v.as_str()) {
        Ok(text.to_string())
    } else {
        Err("No DisplayText in Azure response".to_string())
    }
}

/// Convert raw PCM audio data to WAV format
/// Input: 16kHz, mono, 16-bit PCM
/// Output: WAV file bytes
fn convert_pcm_to_wav(pcm_data: Vec<u8>) -> Result<Vec<u8>, String> {
    let sample_rate = 16000u32;
    let channels = 1u16;
    let bits_per_sample = 16u16;

    let data_size = pcm_data.len() as u32;
    let file_size = 36 + data_size;

    let mut wav_data = Vec::with_capacity((file_size + 8) as usize);

    // RIFF header
    wav_data.extend_from_slice(b"RIFF");
    wav_data.extend_from_slice(&file_size.to_le_bytes());
    wav_data.extend_from_slice(b"WAVE");

    // fmt chunk
    wav_data.extend_from_slice(b"fmt ");
    wav_data.extend_from_slice(&16u32.to_le_bytes()); // fmt chunk size
    wav_data.extend_from_slice(&1u16.to_le_bytes()); // audio format (PCM)
    wav_data.extend_from_slice(&channels.to_le_bytes());
    wav_data.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate * channels as u32 * bits_per_sample as u32 / 8;
    wav_data.extend_from_slice(&byte_rate.to_le_bytes());
    let block_align = channels * bits_per_sample / 8;
    wav_data.extend_from_slice(&block_align.to_le_bytes());
    wav_data.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data chunk
    wav_data.extend_from_slice(b"data");
    wav_data.extend_from_slice(&data_size.to_le_bytes());
    wav_data.extend_from_slice(&pcm_data);

    Ok(wav_data)
}

/// Test transcription API configuration using a local audio file
pub async fn test_transcription_api_with_file(
    provider: &TranscriptionApiProvider,
    api_key: String,
    model: &str,
    audio_file_path: PathBuf,
    custom_api_url: Option<String>,
) -> Result<String, String> {
    // Read the audio file
    let audio_data =
        std::fs::read(&audio_file_path).map_err(|e| format!("Failed to read audio file: {}", e))?;

    // For DashScope, convert to base64 data URL (DashScope doesn't support file:// URLs)
    if provider.id == "dashscope" {
        return test_dashscope_with_base64(
            provider,
            api_key,
            model,
            audio_data,
            custom_api_url,
            Some(&audio_file_path),
        )
        .await;
    }

    // For other providers, send as binary
    transcribe_audio(provider, api_key, model, audio_data, None, custom_api_url).await
}

/// Decode audio file (MP3, WAV, etc.) to PCM format using rodio
/// Returns PCM bytes (16kHz, mono, 16-bit)
fn decode_audio_file(audio_data: Vec<u8>, file_path: &std::path::Path) -> Result<Vec<u8>, String> {
    use rodio::{Decoder, Source};
    use rubato::{FftFixedIn, Resampler};
    use std::io::Cursor;

    let extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    warn!("[Audio Decode] File extension: {}", extension);
    warn!("[Audio Decode] Audio data size: {} bytes", audio_data.len());

    // Create a cursor for rodio to read from
    let cursor = Cursor::new(audio_data);

    // Try to decode the audio file
    let decoder = Decoder::new(cursor).map_err(|e| format!("Failed to create decoder: {}", e))?;

    // Get sample rate and channels
    let sample_rate = decoder.sample_rate();
    let channels = decoder.channels();

    warn!(
        "[Audio Decode] Sample rate: {}, Channels: {}",
        sample_rate, channels
    );

    // Collect all samples first (decoder implements Source trait)
    let samples: Vec<f32> = decoder.collect();
    warn!("[Audio Decode] Collected {} samples", samples.len());

    // Convert to mono if stereo
    let mono_samples = if channels > 1 {
        let mut mono = Vec::with_capacity(samples.len() / channels as usize);
        for chunk in samples.chunks(channels as usize) {
            let avg = chunk.iter().sum::<f32>() / chunk.len() as f32;
            mono.push(avg);
        }
        warn!(
            "[Audio Decode] Converted stereo to mono: {} samples",
            mono.len()
        );
        mono
    } else {
        samples
    };

    // Resample to 16kHz if needed
    let final_samples = if sample_rate != 16000 {
        warn!(
            "[Audio Decode] Resampling from {} Hz to 16000 Hz",
            sample_rate
        );
        warn!("[Audio Decode] Input samples: {}", mono_samples.len());

        // Use FftFixedIn for better handling of large audio files
        use rubato::{FftFixedIn, Resampler};

        let ratio = 16000.0 / sample_rate as f64;
        let chunk_size = 1024;

        // Create resampler
        let mut resampler = FftFixedIn::<f32>::new(
            sample_rate as usize,
            16000,
            chunk_size,
            1, // channels (mono)
            1, // channel count
        )
        .map_err(|e| format!("Failed to create resampler: {}", e))?;

        // Process audio in chunks
        let mut resampled_samples = Vec::new();
        let mut remaining_samples = mono_samples.as_slice();

        while !remaining_samples.is_empty() {
            let chunk_size_actual = chunk_size.min(remaining_samples.len());
            let chunk = &remaining_samples[..chunk_size_actual];

            // Prepare input buffer
            let input = vec![chunk];

            // Process chunk
            match resampler.process(&input, None) {
                Ok(output) => {
                    if !output.is_empty() && !output[0].is_empty() {
                        resampled_samples.extend_from_slice(&output[0]);
                    }
                }
                Err(e) => {
                    warn!("[Audio Decode] Resampler error on chunk: {}", e);
                    // Continue with next chunk
                }
            }

            remaining_samples = &remaining_samples[chunk_size_actual..];
        }

        warn!(
            "[Audio Decode] Resampled to {} samples (expected ~{})",
            resampled_samples.len(),
            ((mono_samples.len() as f64) * ratio).ceil() as usize
        );

        if resampled_samples.is_empty() {
            return Err("Resampling produced no output samples".to_string());
        }

        resampled_samples
    } else {
        mono_samples
    };

    // Convert f32 samples to i16 PCM bytes
    let mut pcm_bytes = Vec::with_capacity(final_samples.len() * 2);
    for sample in final_samples {
        let sample_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
        pcm_bytes.extend_from_slice(&sample_i16.to_le_bytes());
    }

    warn!(
        "[Audio Decode] Decoded PCM size: {} bytes ({} samples)",
        pcm_bytes.len(),
        pcm_bytes.len() / 2
    );

    Ok(pcm_bytes)
}

/// Test DashScope API using base64 data URL (DashScope doesn't support file:// URLs)
async fn test_dashscope_with_base64(
    provider: &TranscriptionApiProvider,
    api_key: String,
    model: &str,
    audio_data: Vec<u8>,
    custom_api_url: Option<String>,
    audio_file_path: Option<&std::path::Path>,
) -> Result<String, String> {
    warn!("[DashScope Test] ===== Starting test =====");
    warn!("[DashScope Test] Model: {}", model);
    warn!(
        "[DashScope Test] Audio data size: {} bytes",
        audio_data.len()
    );

    // Decode audio file if path is provided
    let pcm_data = if let Some(path) = audio_file_path {
        match decode_audio_file(audio_data.clone(), path) {
            Ok(pcm) => {
                warn!(
                    "[DashScope Test] Successfully decoded audio file to PCM: {} bytes",
                    pcm.len()
                );
                if pcm.is_empty() {
                    return Err("Decoded PCM data is empty".to_string());
                }
                pcm
            }
            Err(e) => {
                warn!("[DashScope Test] Failed to decode audio file: {}", e);
                return Err(format!(
                    "Failed to decode audio file: {}. Cannot use raw MP3 data as PCM.",
                    e
                ));
            }
        }
    } else {
        audio_data // No path provided, assume it's already PCM
    };

    let base_url = if let Some(custom_url) = custom_api_url {
        custom_url.trim_end_matches('/').to_string()
    } else {
        provider.base_url.trim_end_matches('/').to_string()
    };

    // Use DashScope MultiModalConversation endpoint
    let url = format!(
        "{}/services/aigc/multimodal-generation/generation",
        base_url.trim_end_matches('/')
    );

    warn!("[DashScope Test] Base URL: {}", base_url);
    warn!("[DashScope Test] Full URL: {}", url);

    let client = create_client(provider, &api_key).map_err(|e| {
        error!("[DashScope Test] Failed to create HTTP client: {}", e);
        e
    })?;

    // Convert PCM to WAV format (DashScope expects WAV format with headers)
    let wav_data = convert_pcm_to_wav(pcm_data)?;

    // Convert WAV to base64 data URL (like transcribe_dashscope does)
    let audio_base64 =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &wav_data);
    let audio_url = format!("data:audio/wav;base64,{}", audio_base64);

    warn!(
        "[DashScope Test] Using base64 data URL (WAV format, length: {} chars)",
        audio_url.len()
    );
    warn!("[DashScope Test] WAV data size: {} bytes", wav_data.len());
    warn!(
        "[DashScope Test] Base64 length: {} chars",
        audio_base64.len()
    );

    // According to qwen3_asr.py, system_prompt defaults to empty string ""
    let system_prompt = "".to_string();
    let messages = vec![
        DashScopeMessage {
            role: "system".to_string(),
            content: vec![DashScopeContent::Text {
                text: system_prompt,
            }],
        },
        DashScopeMessage {
            role: "user".to_string(),
            content: vec![DashScopeContent::Audio { audio: audio_url }],
        },
    ];

    let asr_options = Some(DashScopeAsrOptions {
        language: None,
        enable_lid: Some(true), // Enable language identification as per Python script
        enable_itn: Some(false),
    });

    // DashScope native API format
    #[derive(Debug, Serialize)]
    struct DashScopeNativeRequest {
        model: String,
        input: DashScopeNativeInput,
        parameters: DashScopeNativeParameters,
    }

    #[derive(Debug, Serialize)]
    struct DashScopeNativeInput {
        messages: Vec<DashScopeMessage>,
    }

    #[derive(Debug, Serialize)]
    struct DashScopeNativeParameters {
        #[serde(rename = "result_format")]
        result_format: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        asr_options: Option<DashScopeAsrOptions>,
    }

    let request_body = DashScopeNativeRequest {
        model: model.to_string(),
        input: DashScopeNativeInput { messages },
        parameters: DashScopeNativeParameters {
            result_format: "message".to_string(),
            asr_options,
        },
    };

    // Log request for debugging
    let request_json = serde_json::to_string_pretty(&request_body)
        .unwrap_or_else(|_| "Failed to serialize request".to_string());
    warn!("[DashScope Test] URL: {}", url);
    warn!("[DashScope Test] Request: {}", request_json);

    warn!("[DashScope Test] Sending HTTP request...");
    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            let error_msg = format!("HTTP request failed: {}", e);
            error!("[DashScope Test] {}", error_msg);
            error_msg
        })?;

    let status = response.status();
    warn!("[DashScope Test] Response status: {}", status);
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error response".to_string());
        let error_msg = format!("API request failed with status {}: {}", status, error_text);
        error!("[DashScope Test] {}", error_msg);
        return Err(error_msg);
    }

    let response_text = response.text().await.map_err(|e| {
        let error_msg = format!("Failed to read response: {}", e);
        error!("[DashScope Test] {}", error_msg);
        error_msg
    })?;

    warn!("[DashScope Test] Response: {}", response_text);

    // Parse DashScope native API response
    // DashScope may return different response formats, so we need to handle both success and error cases
    #[derive(Debug, Deserialize)]
    struct DashScopeNativeResponse {
        #[serde(default)]
        output: Option<DashScopeOutput>,
        #[serde(default)]
        code: Option<String>,
        #[serde(default)]
        message: Option<String>,
        #[serde(default)]
        request_id: Option<String>,
    }

    let response_data: DashScopeNativeResponse =
        serde_json::from_str(&response_text).map_err(|e| {
            let error_msg = format!(
                "Failed to parse API response: {}\n\nResponse text: {}",
                e, response_text
            );
            error!("[DashScope Test] Parse error: {}", error_msg);
            error_msg
        })?;

    // Check for errors first
    if let Some(code) = &response_data.code {
        if !code.is_empty() {
            let error_msg = format!(
                "DashScope API error [{}]: {}\nRequest ID: {:?}",
                code,
                response_data.message.as_deref().unwrap_or("Unknown error"),
                response_data.request_id
            );
            error!("[DashScope Test] API error: {}", error_msg);
            return Err(error_msg);
        }
    }

    // Check if output exists
    if let Some(output) = response_data.output {
        if let Some(choice) = output.choices.first() {
            // Log detailed information about the response
            warn!(
                "[DashScope Test] Choice found, content array length: {}",
                choice.message.content.len()
            );
            warn!("[DashScope Test] Finish reason: {:?}", choice.finish_reason);

            // Check all content items, not just first
            for (idx, content_item) in choice.message.content.iter().enumerate() {
                warn!(
                    "[DashScope Test] Content[{}]: text length = {}",
                    idx,
                    content_item.text.len()
                );
                if !content_item.text.is_empty() {
                    warn!(
                        "[DashScope Test] Found non-empty text: {}",
                        if content_item.text.chars().count() > 100 {
                            format!("{}...", content_item.text.chars().take(100).collect::<String>())
                        } else {
                            content_item.text.clone()
                        }
                    );
                    return Ok(content_item.text.clone());
                }
            }

            // Empty content array - API processed successfully but no text detected
            // This can happen with silent audio or very short clips
            warn!("[DashScope Test] No content in response (empty array). This may indicate silent audio or very short clip.");
            warn!(
                "[DashScope Test] Full response for debugging: {}",
                response_text
            );
            Ok(String::new())
        } else {
            Err(format!(
                "No choices in response. Response: {}",
                response_text
            ))
        }
    } else {
        Err(format!(
            "No output in response. Response: {}",
            response_text
        ))
    }
}

/// Test transcription API configuration
pub async fn test_transcription_api(
    provider: &TranscriptionApiProvider,
    api_key: String,
    model: &str,
    custom_api_url: Option<String>,
) -> Result<String, String> {
    // Create a short silent audio sample for testing (1 second)
    let sample_duration_ms = 1000;
    let sample_rate = 16000;
    let samples = (sample_rate * sample_duration_ms / 1000) * 2; // 16-bit = 2 bytes per sample
    let test_audio = vec![0u8; samples];

    match transcribe_audio(provider, api_key, model, test_audio, None, custom_api_url).await {
        Ok(_) => Ok("连接成功".to_string()),
        Err(e) => {
            // If the error is about empty audio or similar, consider it a successful connection
            if e.contains("empty") || e.contains("silence") || e.contains("no speech") {
                Ok("连接成功 (API可访问)".to_string())
            } else {
                Err(e)
            }
        }
    }
}
