use crate::settings::PostProcessProvider;
use log::{debug, warn};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, REFERER, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::net::ToSocketAddrs;
use std::process::Command;

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Debug, Deserialize)]
struct ChatMessageResponse {
    content: Option<String>,
}

/// Build headers for API requests based on provider type
fn build_headers(provider: &PostProcessProvider, api_key: &str) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();

    // Common headers
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://github.com/cjpais/KeVoiceInput"),
    );
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("KeVoiceInput/1.0 (+https://github.com/cjpais/KeVoiceInput)"),
    );
    headers.insert("X-Title", HeaderValue::from_static("KeVoiceInput"));

    // Provider-specific auth headers
    if !api_key.is_empty() {
        if provider.id == "anthropic" {
            headers.insert(
                "x-api-key",
                HeaderValue::from_str(api_key)
                    .map_err(|e| format!("Invalid API key header value: {}", e))?,
            );
            headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
        } else {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", api_key))
                    .map_err(|e| format!("Invalid authorization header value: {}", e))?,
            );
        }
    }

    Ok(headers)
}

/// Create an HTTP client with provider-specific headers
fn create_client(provider: &PostProcessProvider, api_key: &str) -> Result<reqwest::Client, String> {
    let headers = build_headers(provider, api_key)?;

    // Try to get system proxy settings from environment
    let http_proxy = std::env::var("http_proxy")
        .ok()
        .or_else(|| std::env::var("HTTP_PROXY").ok());
    let https_proxy = std::env::var("https_proxy")
        .ok()
        .or_else(|| std::env::var("HTTPS_PROXY").ok());

    let mut builder = reqwest::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(60))
        .connect_timeout(std::time::Duration::from_secs(30));

    // Set proxy if available
    if let Some(proxy_url) = https_proxy.or(http_proxy) {
        if let Ok(proxy) =
            reqwest::Proxy::https(&proxy_url).or_else(|_| reqwest::Proxy::http(&proxy_url))
        {
            builder = builder.proxy(proxy);
        }
    }

    builder
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))
}

/// Send a chat completion request to an OpenAI-compatible API
/// Returns Ok(Some(content)) on success, Ok(None) if response has no content,
/// or Err on actual errors (HTTP, parsing, etc.)
pub async fn send_chat_completion(
    provider: &PostProcessProvider,
    api_key: String,
    model: &str,
    prompt: String,
) -> Result<Option<String>, String> {
    send_chat_completion_with_messages(provider, api_key, model, None, prompt).await
}

/// Send a chat completion request with separate system and user messages
///
/// This function sends a request in OpenAI-compatible format:
/// ```json
/// {
///   "model": "qwen-plus",
///   "messages": [
///     {
///       "role": "system",
///       "content": "提示词内容（角色指令）"
///     },
///     {
///       "role": "user",
///       "content": "转录文本（用户输入）"
///     }
///   ]
/// }
/// ```
///
/// - `system_prompt`: 提示词模板（角色指令），放在 role="system" 的 content 中
/// - `user_content`: 转录文本（用户输入），放在 role="user" 的 content 中
///
/// Returns Ok(Some(content)) on success, Ok(None) if response has no content,
/// or Err on actual errors (HTTP, parsing, etc.)
pub async fn send_chat_completion_with_messages(
    provider: &PostProcessProvider,
    api_key: String,
    model: &str,
    system_prompt: Option<String>,
    user_content: String,
) -> Result<Option<String>, String> {
    send_chat_completion_with_messages_and_url(
        provider,
        api_key,
        model,
        system_prompt,
        user_content,
        None,
    )
    .await
}

/// Send a chat completion request with optional custom API URL
pub async fn send_chat_completion_with_messages_and_url(
    provider: &PostProcessProvider,
    api_key: String,
    model: &str,
    system_prompt: Option<String>,
    user_content: String,
    custom_api_url: Option<String>,
) -> Result<Option<String>, String> {
    let base_url = if let Some(custom_url) = custom_api_url {
        custom_url.trim_end_matches('/').to_string()
    } else {
        provider.base_url.trim_end_matches('/').to_string()
    };
    let url = format!("{}/chat/completions", base_url);

    debug!("Sending chat completion request to: {}", url);

    let client = create_client(provider, &api_key)?;

    // Build messages array following OpenAI-compatible format:
    // - system message: contains the role prompt/instruction
    // - user message: contains the transcription text/user input
    let mut messages = Vec::new();
    if let Some(system) = system_prompt {
        debug!("Adding system message (role prompt): '{}'", system);
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: system, // 提示词放在 system message 的 content 中
        });
    }
    debug!(
        "Adding user message (transcription text): '{}'",
        user_content
    );
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: user_content, // 转录文本放在 user message 的 content 中
    });

    debug!("Sending {} messages to LLM", messages.len());
    let request_body = ChatCompletionRequest {
        model: model.to_string(),
        messages,
    };

    let response = client
        .post(&url)
        .json(&request_body)
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

    let completion: ChatCompletionResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse API response: {}", e))?;

    // Extract the response content
    let result = completion
        .choices
        .first()
        .and_then(|choice| choice.message.content.clone());

    // Log the raw response for debugging
    if let Some(ref content) = result {
        debug!(
            "LLM raw response content (length: {}): '{}'",
            content.len(),
            content
        );
        // Check if response contains unexpected content
        if content.contains("curl") || content.contains("POST") || content.contains("Authorization")
        {
            warn!("LLM response contains curl command or HTTP request details. This may indicate the LLM misunderstood the request.");
            warn!("Full response: {}", content);
        }
    } else {
        debug!("LLM returned empty response");
    }

    Ok(result)
}

/// Test LLM configuration connectivity
/// Sends a simple test request to verify the API is accessible
pub async fn test_llm_config(
    provider: &PostProcessProvider,
    api_key: String,
    model: &str,
) -> Result<String, String> {
    test_llm_config_with_url(provider, api_key, model, None).await
}

#[derive(Debug, Serialize)]
pub struct TestResult {
    pub success: bool,
    pub message: String,
    pub curl_command: Option<String>,
}

pub async fn test_llm_config_with_url(
    provider: &PostProcessProvider,
    api_key: String,
    model: &str,
    custom_api_url: Option<String>,
) -> Result<String, String> {
    // #region agent log
    let custom_api_url_for_log = custom_api_url.clone();
    let log_msg = format!(
        r#"{{"id":"log_{}","timestamp":{},"location":"llm_client.rs:245","message":"URL construction","data":{{"custom_api_url":"{:?}","provider_base_url":"{}"}},"sessionId":"debug-session","runId":"llm-test","hypothesisId":"A"}}"#,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        custom_api_url_for_log,
        provider.base_url
    );
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
    {
        let _ = writeln!(file, "{}", log_msg);
    }
    // #endregion

    let base_url = if let Some(custom_url) = custom_api_url {
        custom_url.trim_end_matches('/').to_string()
    } else {
        provider.base_url.trim_end_matches('/').to_string()
    };

    let url = format!("{}/chat/completions", base_url);

    // Check if this is an internal network address
    let is_internal = if let Ok(parsed_url) = url.parse::<reqwest::Url>() {
        if let Some(host) = parsed_url.host_str() {
            // Check DNS resolution to see if it's an internal IP
            let host_port = format!("{}:443", host);
            if let Ok(mut addrs) = host_port.to_socket_addrs() {
                if let Some(addr) = addrs.next() {
                    let ip = addr.ip();
                    // Check if it's a private/internal IP
                    match ip {
                        std::net::IpAddr::V4(ipv4) => {
                            ipv4.is_private() || ipv4.is_loopback() || ipv4.is_link_local()
                        }
                        std::net::IpAddr::V6(ipv6) => ipv6.is_loopback() || ipv6.is_unspecified(),
                    }
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    // #region agent log
    let log_msg = format!(
        r#"{{"id":"log_{}","timestamp":{},"location":"llm_client.rs:260","message":"Final URL","data":{{"final_base_url":"{}","final_url":"{}","is_internal":{}}},"sessionId":"debug-session","runId":"llm-test","hypothesisId":"A"}}"#,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        base_url,
        url,
        is_internal
    );
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
    {
        let _ = writeln!(file, "{}", log_msg);
    }
    // #endregion

    debug!(
        "Testing LLM config: {} with model {} (internal: {})",
        url, model, is_internal
    );

    // Generate curl command for debugging
    let curl_command = generate_curl_command(provider, &base_url, &api_key, model);

    // Test with curl first - if curl succeeds, use that result instead of reqwest
    // This works around network/SSL issues in the app environment
    let curl_test_result = test_with_curl(&url, provider, &api_key, model);

    // #region agent log
    let log_msg = format!(
        r#"{{"id":"log_{}","timestamp":{},"location":"llm_client.rs:285","message":"Curl test result","data":{{"success":{},"output":"{}","error":"{}"}},"sessionId":"debug-session","runId":"llm-test","hypothesisId":"CURL"}}"#,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        curl_test_result.success,
        curl_test_result
            .output
            .replace('"', r#"\""#)
            .replace('\n', r#"\n"#)
            .chars()
            .take(1000)
            .collect::<String>(),
        curl_test_result
            .error
            .replace('"', r#"\""#)
            .replace('\n', r#"\n"#)
            .chars()
            .take(500)
            .collect::<String>()
    );
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
    {
        let _ = writeln!(file, "{}", log_msg);
    }
    // #endregion

    // If curl test succeeded, parse the response and return success
    if curl_test_result.success && !curl_test_result.output.is_empty() {
        // Try to parse the JSON response from curl
        // Extract JSON from output (curl -v includes verbose info, so we need to find the JSON part)
        let json_start = curl_test_result.output.find('{');
        let json_text = if let Some(start) = json_start {
            &curl_test_result.output[start..]
        } else {
            &curl_test_result.output
        };

        match serde_json::from_str::<ChatCompletionResponse>(json_text) {
            Ok(completion) => {
                if !completion.choices.is_empty() {
                    // #region agent log
                    let log_msg = format!(
                        r#"{{"id":"log_{}","timestamp":{},"location":"llm_client.rs:300","message":"Curl test succeeded, using curl result","data":{{"choices_count":{}}},"sessionId":"debug-session","runId":"llm-test","hypothesisId":"CURL_SUCCESS"}}"#,
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis(),
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis(),
                        completion.choices.len()
                    );
                    if let Ok(mut file) = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
                    {
                        let _ = writeln!(file, "{}", log_msg);
                    }
                    // #endregion
                    return Ok("连接成功".to_string());
                }
            }
            Err(e) => {
                warn!(
                    "Failed to parse curl response: {}. Output: {}",
                    e,
                    curl_test_result
                        .output
                        .chars()
                        .take(200)
                        .collect::<String>()
                );
            }
        }
    } else if !curl_test_result.success {
        // Curl failed - this might be due to app network restrictions
        // If it's an internal network address, provide specific guidance
        if is_internal {
            warn!("Curl test failed for internal network address: {}. This may be due to app network restrictions.", curl_test_result.error);
            // For internal addresses, skip reqwest and return error immediately with helpful message
            return Err(format!(
                "连接失败：内网地址无法访问\n\n检测到这是内网地址（{}），应用可能无法访问内网服务。\n\n如果终端可以访问，可能是应用网络权限限制。\n\n请在终端测试以下命令：\n{}",
                base_url, curl_command
            ));
        } else {
            warn!(
                "Curl test failed: {}. Will try reqwest as fallback.",
                curl_test_result.error
            );
        }
    }

    // #region agent log
    let log_msg = format!(
        r#"{{"id":"log_{}","timestamp":{},"location":"llm_client.rs:250","message":"Creating HTTP client","data":{{"provider_id":"{}"}},"sessionId":"debug-session","runId":"llm-test","hypothesisId":"A"}}"#,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        provider.id
    );
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
    {
        let _ = writeln!(file, "{}", log_msg);
    }
    // #endregion

    let client = create_client(provider, &api_key).map_err(|e| {
        // #region agent log
        let log_msg = format!(
            r#"{{"id":"log_{}","timestamp":{},"location":"llm_client.rs:260","message":"Failed to create HTTP client","data":{{"error":"{}"}},"sessionId":"debug-session","runId":"llm-test","hypothesisId":"A"}}"#,
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
            e.replace('"', r#"\""#)
        );
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log") {
            let _ = writeln!(file, "{}", log_msg);
        }
        // #endregion
        format!("Failed to create HTTP client: {}\n\nCurl command:\n{}", e, curl_command)
    })?;

    // #region agent log
    let log_msg = format!(
        r#"{{"id":"log_{}","timestamp":{},"location":"llm_client.rs:275","message":"HTTP client created, sending request","data":{{}},"sessionId":"debug-session","runId":"llm-test","hypothesisId":"A"}}"#,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
    {
        let _ = writeln!(file, "{}", log_msg);
    }
    // #endregion

    // Send a simple test message
    let request_body = ChatCompletionRequest {
        model: model.to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "你是谁？".to_string(),
            },
        ],
    };

    // #region agent log
    let log_msg = format!(
        r#"{{"id":"log_{}","timestamp":{},"location":"llm_client.rs:265","message":"Starting LLM test","data":{{"url":"{}","model":"{}"}},"sessionId":"debug-session","runId":"llm-test","hypothesisId":"A"}}"#,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        url,
        model
    );
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
    {
        let _ = writeln!(file, "{}", log_msg);
    }
    // #endregion

    // #region agent log
    let request_body_json =
        serde_json::to_string(&request_body).unwrap_or_else(|_| "{}".to_string());

    // Try to resolve DNS to check if it's a DNS issue
    let url_for_dns = url.clone();
    let dns_test = tokio::task::spawn(async move {
        use std::net::ToSocketAddrs;
        if let Ok(parsed_url) = url_for_dns.parse::<reqwest::Url>() {
            if let Some(host) = parsed_url.host_str() {
                let host_port = format!("{}:443", host);
                match host_port.to_socket_addrs() {
                    Ok(mut addrs) => {
                        if let Some(addr) = addrs.next() {
                            return format!("DNS resolved: {} -> {}", host, addr);
                        }
                    }
                    Err(e) => return format!("DNS resolution failed: {}", e),
                }
            }
        }
        "DNS test skipped".to_string()
    });

    let log_msg = format!(
        r#"{{"id":"log_{}","timestamp":{},"location":"llm_client.rs:320","message":"Sending HTTP POST request","data":{{"url":"{}","request_body":"{}"}},"sessionId":"debug-session","runId":"llm-test","hypothesisId":"B"}}"#,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        url,
        request_body_json.replace('"', r#"\""#)
    );
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
    {
        let _ = writeln!(file, "{}", log_msg);
    }

    // Log DNS resolution result
    if let Ok(dns_result) = dns_test.await {
        let log_msg = format!(
            r#"{{"id":"log_{}","timestamp":{},"location":"llm_client.rs:330","message":"DNS resolution test","data":{{"result":"{}"}},"sessionId":"debug-session","runId":"llm-test","hypothesisId":"B"}}"#,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            dns_result.replace('"', r#"\""#)
        );
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
        {
            let _ = writeln!(file, "{}", log_msg);
        }
    }
    // #endregion

    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            // #region agent log
            let error_detail = format!("{:?}", e);
            let error_source = if e.is_timeout() {
                "timeout"
            } else if e.is_connect() {
                "connection"
            } else if e.is_request() {
                "request"
            } else {
                "unknown"
            };
            let log_msg = format!(
                r#"{{"id":"log_{}","timestamp":{},"location":"llm_client.rs:335","message":"HTTP request failed","data":{{"error":"{}","error_detail":"{}","error_source":"{}","is_timeout":{},"is_connect":{},"is_request":{}}},"sessionId":"debug-session","runId":"llm-test","hypothesisId":"B"}}"#,
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                e.to_string().replace('"', r#"\""#),
                error_detail.replace('"', r#"\""#),
                error_source,
                e.is_timeout(),
                e.is_connect(),
                e.is_request()
            );
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log") {
                let _ = writeln!(file, "{}", log_msg);
            }
            // #endregion
            // If curl also failed, provide a more helpful error message
            if !curl_test_result.success {
                format!(
                    "连接失败：应用内网络测试失败\n\n错误详情：{}\n\n这可能是因为应用网络环境限制。\n\n请在终端测试以下命令：\n{}",
                    e, curl_command
                )
            } else {
                format!("HTTP request failed: {} (source: {})\n\nCurl command:\n{}", e, error_source, curl_command)
            }
        })?;

    let status = response.status();
    debug!("LLM test response status: {}", status);

    // #region agent log
    let log_msg = format!(
        r#"{{"id":"log_{}","timestamp":{},"location":"llm_client.rs:290","message":"Received HTTP response","data":{{"status":{},"status_text":"{}"}},"sessionId":"debug-session","runId":"llm-test","hypothesisId":"B"}}"#,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        status.as_u16(),
        status.as_str()
    );
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
    {
        let _ = writeln!(file, "{}", log_msg);
    }
    // #endregion

    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error response".to_string());
        debug!("LLM test error response: {}", error_text);
        // #region agent log
        let log_msg = format!(
            r#"{{"id":"log_{}","timestamp":{},"location":"llm_client.rs:300","message":"HTTP error status","data":{{"status":{},"error_text":"{}"}},"sessionId":"debug-session","runId":"llm-test","hypothesisId":"B"}}"#,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            status.as_u16(),
            error_text
                .chars()
                .take(500)
                .collect::<String>()
                .replace('"', r#"\""#)
        );
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
        {
            let _ = writeln!(file, "{}", log_msg);
        }
        // #endregion
        // Include curl command in error message
        return Err(format!(
            "API request failed with status {}: {}\n\nCurl command:\n{}",
            status, error_text, curl_command
        ));
    }

    // Read response text first for debugging
    let response_text = response.text().await.map_err(|e| {
        // #region agent log
        let log_msg = format!(
            r#"{{"id":"log_{}","timestamp":{},"location":"llm_client.rs:320","message":"Failed to read response text","data":{{"error":"{}"}},"sessionId":"debug-session","runId":"llm-test","hypothesisId":"C"}}"#,
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
            e.to_string().replace('"', r#"\""#)
        );
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log") {
            let _ = writeln!(file, "{}", log_msg);
        }
        // #endregion
        format!("Failed to read response body: {}\n\nCurl command:\n{}", e, curl_command)
    })?;

    debug!(
        "LLM test raw response (first 500 chars): {}",
        response_text.chars().take(500).collect::<String>()
    );

    // #region agent log
    let response_preview = response_text.chars().take(1000).collect::<String>();
    let log_msg = format!(
        r#"{{"id":"log_{}","timestamp":{},"location":"llm_client.rs:335","message":"Response text received","data":{{"response_preview":"{}","response_length":{}}},"sessionId":"debug-session","runId":"llm-test","hypothesisId":"C"}}"#,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        response_preview
            .replace('"', r#"\""#)
            .replace('\n', r#"\n"#),
        response_text.len()
    );
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
    {
        let _ = writeln!(file, "{}", log_msg);
    }
    // #endregion

    // Try to parse response to verify it's valid
    let completion: ChatCompletionResponse = serde_json::from_str(&response_text)
        .map_err(|e| {
            warn!("Failed to parse LLM response: {}. Response text: {}", e, 
                  response_text.chars().take(200).collect::<String>());
            // #region agent log
            let log_msg = format!(
                r#"{{"id":"log_{}","timestamp":{},"location":"llm_client.rs:350","message":"JSON parse failed","data":{{"error":"{}","response_preview":"{}"}},"sessionId":"debug-session","runId":"llm-test","hypothesisId":"D"}}"#,
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis(),
                e.to_string().replace('"', r#"\""#),
                response_text.chars().take(500).collect::<String>().replace('"', r#"\""#).replace('\n', r#"\n"#)
            );
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log") {
                let _ = writeln!(file, "{}", log_msg);
            }
            // #endregion
            // Include curl command in error message
            format!("Failed to parse API response: {}\n\nCurl command:\n{}", e, curl_command)
        })?;

    debug!(
        "LLM test parsed response: choices count = {}",
        completion.choices.len()
    );

    // #region agent log
    let log_msg = format!(
        r#"{{"id":"log_{}","timestamp":{},"location":"llm_client.rs:365","message":"JSON parsed successfully","data":{{"choices_count":{}}},"sessionId":"debug-session","runId":"llm-test","hypothesisId":"E"}}"#,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        completion.choices.len()
    );
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
    {
        let _ = writeln!(file, "{}", log_msg);
    }
    // #endregion

    // Return success message
    if completion.choices.is_empty() {
        warn!(
            "LLM test returned empty choices array. Full response: {}",
            response_text.chars().take(500).collect::<String>()
        );
        // #region agent log
        let log_msg = format!(
            r#"{{"id":"log_{}","timestamp":{},"location":"llm_client.rs:375","message":"Empty choices array","data":{{"response_preview":"{}"}},"sessionId":"debug-session","runId":"llm-test","hypothesisId":"F"}}"#,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            response_text
                .chars()
                .take(500)
                .collect::<String>()
                .replace('"', r#"\""#)
                .replace('\n', r#"\n"#)
        );
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
        {
            let _ = writeln!(file, "{}", log_msg);
        }
        // #endregion
        Err(format!(
            "API returned empty response\n\nCurl command:\n{}",
            curl_command
        ))
    } else {
        debug!("LLM test succeeded");
        // #region agent log
        let log_msg = format!(
            r#"{{"id":"log_{}","timestamp":{},"location":"llm_client.rs:385","message":"LLM test succeeded","data":{{"choices_count":{}}},"sessionId":"debug-session","runId":"llm-test","hypothesisId":"E"}}"#,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            completion.choices.len()
        );
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("/Users/thinkre/Desktop/projects/KeVoiceInput/.cursor/debug.log")
        {
            let _ = writeln!(file, "{}", log_msg);
        }
        // #endregion
        Ok("连接成功".to_string())
    }
}

/// Test API connectivity using curl command
fn test_with_curl(
    url: &str,
    provider: &PostProcessProvider,
    api_key: &str,
    model: &str,
) -> CurlTestResult {
    let json_body = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": "You are a helpful assistant."
            },
            {
                "role": "user",
                "content": "你是谁？"
            }
        ]
    });

    let json_str = serde_json::to_string(&json_body).unwrap_or_else(|_| "{}".to_string());

    let mut cmd = Command::new("curl");
    cmd.arg("-X").arg("POST");
    cmd.arg(url);
    cmd.arg("-H").arg("Content-Type: application/json");

    if provider.id == "anthropic" {
        cmd.arg("-H").arg(&format!("x-api-key: {}", api_key));
        cmd.arg("-H").arg("anthropic-version: 2023-06-01");
    } else {
        cmd.arg("-H")
            .arg(&format!("Authorization: Bearer {}", api_key));
    }

    cmd.arg("-d").arg(&json_str);

    match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let success = output.status.success();

            // Combine stdout and stderr for better error reporting
            let combined_output = if !stdout.is_empty() {
                stdout.to_string()
            } else {
                stderr.to_string()
            };

            CurlTestResult {
                success,
                output: combined_output,
                error: if success {
                    String::new()
                } else {
                    stderr.to_string()
                },
            }
        }
        Err(e) => CurlTestResult {
            success: false,
            output: String::new(),
            error: format!("Failed to execute curl: {}", e),
        },
    }
}

struct CurlTestResult {
    success: bool,
    output: String,
    error: String,
}

/// Generate a curl command for testing the LLM API
fn generate_curl_command(
    provider: &PostProcessProvider,
    base_url: &str,
    api_key: &str,
    model: &str,
) -> String {
    let url = format!("{}/chat/completions", base_url);
    let json_body = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": "You are a helpful assistant."
            },
            {
                "role": "user",
                "content": "你是谁？"
            }
        ]
    });

    // Format JSON for curl command (single line, escaped)
    let json_str = serde_json::to_string(&json_body).unwrap_or_else(|_| "{}".to_string());
    let escaped_json = json_str.replace('\'', "'\\''");

    if provider.id == "anthropic" {
        format!(
            "curl -X POST {} \\\n  -H \"Content-Type: application/json\" \\\n  -H \"x-api-key: {}\" \\\n  -H \"anthropic-version: 2023-06-01\" \\\n  -d '{}'",
            url, api_key, escaped_json
        )
    } else {
        format!(
            "curl -X POST {} \\\n  -H \"Authorization: Bearer {}\" \\\n  -H \"Content-Type: application/json\" \\\n  -d '{}'",
            url, api_key, escaped_json
        )
    }
}

/// Fetch available models from an OpenAI-compatible API
/// Returns a list of model IDs
pub async fn fetch_models(
    provider: &PostProcessProvider,
    api_key: String,
) -> Result<Vec<String>, String> {
    let base_url = provider.base_url.trim_end_matches('/');
    let url = format!("{}/models", base_url);

    debug!("Fetching models from: {}", url);

    let client = create_client(provider, &api_key)?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch models: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!(
            "Model list request failed ({}): {}",
            status, error_text
        ));
    }

    let parsed: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let mut models = Vec::new();

    // Handle OpenAI format: { data: [ { id: "..." }, ... ] }
    if let Some(data) = parsed.get("data").and_then(|d| d.as_array()) {
        for entry in data {
            if let Some(id) = entry.get("id").and_then(|i| i.as_str()) {
                models.push(id.to_string());
            } else if let Some(name) = entry.get("name").and_then(|n| n.as_str()) {
                models.push(name.to_string());
            }
        }
    }
    // Handle array format: [ "model1", "model2", ... ]
    else if let Some(array) = parsed.as_array() {
        for entry in array {
            if let Some(model) = entry.as_str() {
                models.push(model.to_string());
            }
        }
    }

    Ok(models)
}
