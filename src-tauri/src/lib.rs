// Amber — Rust backend (Tauri commands).
// M1: talk to a model via OpenRouter with streaming.
// AUTH RULE: API key only (OpenRouter). NEVER a consumer subscription OAuth token.
//
// Key storage (dev): a 0600 file in the app config dir, or the OPENROUTER_API_KEY
// env var. We avoided the macOS keychain here because an unsigned `tauri dev`
// binary changes identity every rebuild, so the keychain re-prompts forever.
// Release builds (code-signed, stable identity) can move this back to the keychain.

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager};

const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
// Hardcoded for M1 — cheap + always-valid OpenRouter slug. M4 makes this dynamic (task routing).
const MODEL: &str = "anthropic/claude-3.5-haiku";
const KEY_FILE: &str = "openrouter.key";

#[derive(Serialize, Deserialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

// Streamed back to the frontend over a Tauri Channel.
// Serializes as { "type": "token", "data": "..." } etc.
#[derive(Clone, Serialize)]
#[serde(rename_all = "lowercase", tag = "type", content = "data")]
enum StreamEvent {
    Token(String),
    Done,
    Error(String),
}

fn key_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join(KEY_FILE))
}

/// Resolve the API key: OPENROUTER_API_KEY env var wins, else the stored file.
fn read_key(app: &AppHandle) -> Option<String> {
    if let Ok(k) = std::env::var("OPENROUTER_API_KEY") {
        let k = k.trim().to_string();
        if !k.is_empty() {
            return Some(k);
        }
    }
    let k = fs::read_to_string(key_path(app).ok()?).ok()?;
    let k = k.trim().to_string();
    (!k.is_empty()).then_some(k)
}

/// Store the OpenRouter API key as a 0600 file in the app config dir. Never in the vault.
#[tauri::command]
fn set_api_key(app: AppHandle, key: String) -> Result<(), String> {
    let key = key.trim();
    if key.is_empty() {
        return Err("API key is empty.".into());
    }
    let path = key_path(&app)?;
    fs::write(&path, key).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path).map_err(|e| e.to_string())?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&path, perms).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// True if a key is available (env var or stored file).
#[tauri::command]
fn has_api_key(app: AppHandle) -> bool {
    read_key(&app).is_some()
}

/// Remove the stored key file (Settings → forget key). Does not touch the env var.
#[tauri::command]
fn clear_api_key(app: AppHandle) -> Result<(), String> {
    let path = key_path(&app)?;
    if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Stream a chat completion from OpenRouter. Tokens arrive on `on_event` as they generate.
#[tauri::command]
async fn chat(
    app: AppHandle,
    messages: Vec<ChatMessage>,
    on_event: Channel<StreamEvent>,
) -> Result<(), String> {
    let api_key = read_key(&app).ok_or("No API key set. Add it in Settings.")?;

    let body = serde_json::json!({
        "model": MODEL,
        "messages": messages,
        "stream": true,
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(OPENROUTER_URL)
        .header("Authorization", format!("Bearer {api_key}"))
        // OpenRouter attribution headers (optional but recommended).
        .header("HTTP-Referer", "https://github.com/inkxel/amber")
        .header("X-Title", "Amber")
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        let msg = format!("OpenRouter {status}: {text}");
        let _ = on_event.send(StreamEvent::Error(msg.clone()));
        return Err(msg);
    }

    // Parse the SSE stream: lines like `data: {json}`, terminated by `data: [DONE]`.
    let mut stream = resp.bytes_stream();
    let mut buf = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        buf.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(nl) = buf.find('\n') {
            let line: String = buf.drain(..=nl).collect();
            let line = line.trim();

            let Some(data) = line.strip_prefix("data:") else {
                continue; // SSE comments (": ...") and blank lines
            };
            let data = data.trim();

            if data == "[DONE]" {
                let _ = on_event.send(StreamEvent::Done);
                return Ok(());
            }

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                if let Some(tok) = json["choices"][0]["delta"]["content"].as_str() {
                    if !tok.is_empty() {
                        let _ = on_event.send(StreamEvent::Token(tok.to_string()));
                    }
                }
            }
        }
    }

    let _ = on_event.send(StreamEvent::Done);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            set_api_key,
            has_api_key,
            clear_api_key,
            chat
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
