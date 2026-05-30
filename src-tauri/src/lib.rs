// Amber — Rust backend (Tauri commands).
// M1: talk to a model via OpenRouter with streaming, API key in the OS keychain.
// AUTH RULE: API key only (OpenRouter). NEVER a consumer subscription OAuth token.

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;

const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
// Hardcoded for M1 — cheap + always-valid OpenRouter slug. M4 makes this dynamic (task routing).
const MODEL: &str = "anthropic/claude-3.5-haiku";
const KEYRING_SERVICE: &str = "amber";
const KEYRING_ACCOUNT: &str = "openrouter";

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

fn entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT).map_err(|e| e.to_string())
}

/// Store the OpenRouter API key in the OS keychain. Never written to the vault or disk.
#[tauri::command]
fn set_api_key(key: String) -> Result<(), String> {
    let key = key.trim();
    if key.is_empty() {
        return Err("API key is empty.".into());
    }
    entry()?.set_password(key).map_err(|e| e.to_string())
}

/// True if a key is present in the keychain.
#[tauri::command]
fn has_api_key() -> bool {
    entry()
        .and_then(|e| e.get_password().map_err(|x| x.to_string()))
        .is_ok()
}

/// Remove the stored key (Settings → forget key).
#[tauri::command]
fn clear_api_key() -> Result<(), String> {
    let e = entry()?;
    match e.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(err.to_string()),
    }
}

/// Stream a chat completion from OpenRouter. Tokens arrive on `on_event` as they generate.
#[tauri::command]
async fn chat(messages: Vec<ChatMessage>, on_event: Channel<StreamEvent>) -> Result<(), String> {
    let api_key = entry()?
        .get_password()
        .map_err(|_| "No API key set. Add it in Settings.".to_string())?;

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
