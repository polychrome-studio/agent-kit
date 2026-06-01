// Amber — Rust backend (Tauri commands).
// M1: talk to a model via OpenRouter with streaming.
// AUTH RULE: API key only (OpenRouter). NEVER a consumer subscription OAuth token.
//
// Key storage (dev): a 0600 file in the app config dir, or the OPENROUTER_API_KEY
// env var. We avoided the macOS keychain here because an unsigned `tauri dev`
// binary changes identity every rebuild, so the keychain re-prompts forever.
// Release builds (code-signed, stable identity) can move this back to the keychain.

mod agent;
mod router;
mod vault;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::ipc::Channel;
use tauri::{AppHandle, Emitter, Manager};

pub(crate) const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const KEY_FILE: &str = "openrouter.key";
const VAULT_FILE: &str = "vault.path";

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct ChatMessage {
    role: String,
    content: String,
}

// Streamed back to the frontend over a Tauri Channel.
// Serializes as { "type": "token", "data": "..." } etc.
#[derive(Clone, Serialize)]
#[serde(rename_all = "lowercase", tag = "type", content = "data")]
pub(crate) enum StreamEvent {
    /// Which mode + model handled this turn (sent first, for the UI's trust/cost label).
    Meta { mode: String, model: String },
    /// The agent invoked a tool this step (search_vault / read_note / web_search) — drives
    /// the "Amber is working" activity trail in the UI.
    Tool { name: String, arg: String },
    /// Vault notes that grounded this answer — only sent in modes that show sources.
    Sources(Vec<String>),
    Token(String),
    Done,
    Error(String),
}

fn config_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn key_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(config_dir(app)?.join(KEY_FILE))
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

/// Resolve the vault folder: AMBER_VAULT env var wins, else the stored path.
/// Returns None if unset or the path is no longer a directory.
pub(crate) fn vault_path(app: &AppHandle) -> Option<PathBuf> {
    let raw = if let Ok(v) = std::env::var("AMBER_VAULT") {
        v
    } else {
        fs::read_to_string(config_dir(app).ok()?.join(VAULT_FILE)).ok()?
    };
    let pb = PathBuf::from(raw.trim());
    pb.is_dir().then_some(pb)
}

/// The currently configured vault folder, if any.
#[tauri::command]
fn get_vault_path(app: AppHandle) -> Option<String> {
    vault_path(&app).map(|p| p.to_string_lossy().into_owned())
}

/// Point Amber at a vault folder (must exist). Empty string clears it.
#[tauri::command]
fn set_vault_path(app: AppHandle, path: String) -> Result<(), String> {
    let path = path.trim();
    let file = config_dir(&app)?.join(VAULT_FILE);
    if path.is_empty() {
        if file.exists() {
            fs::remove_file(&file).map_err(|e| e.to_string())?;
        }
        return Ok(());
    }
    if !PathBuf::from(path).is_dir() {
        return Err(format!("Not a folder: {path}"));
    }
    fs::write(&file, path).map_err(|e| e.to_string())
}

/// Handle one user turn. Classifies the mode (M4), then runs the agent tool-use loop
/// (`agent::run`) which streams the answer and tool activity over `on_event`.
#[tauri::command]
async fn chat(
    app: AppHandle,
    messages: Vec<ChatMessage>,
    on_event: Channel<StreamEvent>,
) -> Result<(), String> {
    let api_key = read_key(&app).ok_or("No API key set. Add it in Settings.")?;

    let query = messages
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .map(|m| m.content.as_str())
        .unwrap_or("");

    // Route the turn: one classification picks the model, the persona, the toolset, and
    // whether sources are shown (M4 — "mode is the primitive"). Sent to the UI up front.
    let mode = router::classify(query, &api_key).await;
    let _ = on_event.send(StreamEvent::Meta {
        mode: mode.label().to_string(),
        model: mode.model().to_string(),
    });

    agent::run(&app, mode, &messages, &api_key, &on_event).await
}

/// Show/hide the command-bar window. The global shortcut and the menubar both
/// route here. Showing also re-focuses and tells the palette UI to reset (one-shot
/// grammar — every summon is a fresh query, see M3 in docs/build-plan.md).
fn toggle_palette(app: &AppHandle) {
    let Some(win) = app.get_webview_window("palette") else {
        return;
    };
    if win.is_visible().unwrap_or(false) {
        let _ = win.hide();
    } else {
        let _ = win.show();
        let _ = win.set_focus();
        let _ = win.emit("palette:show", ());
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri_plugin_global_shortcut::ShortcutState;

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    // Toggle on key-down only; the plugin also fires on release.
                    if event.state == ShortcutState::Pressed {
                        toggle_palette(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
            // Option+Space summons the command bar (Tucker's pick; double-tap
            // right-Shift isn't reachable via the global-shortcut plugin — see
            // knowledge/wiki/roadmap.md). A registration conflict is non-fatal:
            // log and keep running so the app still launches.
            let hotkey = Shortcut::new(Some(Modifiers::ALT), Code::Space);
            if let Err(e) = app.global_shortcut().register(hotkey) {
                eprintln!("Amber: could not register Option+Space hotkey: {e}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_api_key,
            has_api_key,
            clear_api_key,
            get_vault_path,
            set_vault_path,
            chat
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
