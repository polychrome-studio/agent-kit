---
name: architecture
type: subsystem
created: 2026-05-31
last_updated: 2026-05-31
confidence: high
related: [[vault-retrieval]], [[knowledge-layer]], [[build-status]]
---

# Architecture — how Amber is built

Amber is a **Tauri 2** desktop app: a Rust core driving the system webview, with a Vite + React + TypeScript frontend. Small (~10MB), native, fast. The model layer routes through **OpenRouter** (architecture A2 — locked, see [[decisions/2026-05-29-openrouter-a2-architecture]]).

## The layers

- **Frontend (`src/`)** — React. `App.tsx` is the whole UI today: a chat panel + a settings view. Talks to Rust only through Tauri `invoke()` and a streaming `Channel`. Ember-glow aesthetic (warm near-black, amber accents) in `App.css`.
- **Rust backend (`src-tauri/src/`)** — the Tauri commands. `lib.rs` holds the command surface; `vault.rs` holds retrieval. Commands: `set_api_key` / `has_api_key` / `clear_api_key`, `get_vault_path` / `set_vault_path`, `chat`.
- **Model layer** — `chat()` calls OpenRouter's `/chat/completions` with `reqwest`, streaming. The **HTTP call lives in Rust**, so the API key never enters the webview and there's no CSP/CORS surface. Model is hardcoded for now (`anthropic/claude-3.5-haiku`); M4 makes it task-routed.
- **Streaming** — `chat()` parses the SSE stream and pushes `StreamEvent`s back over a Tauri `Channel<StreamEvent>`: `Sources` (vault notes used, sent once before tokens), `Token`, `Done`, `Error`. Serialized as `{type, data}` (serde lowercase tag/content).

## State & secrets

App state lives in the OS app-config dir (`~/Library/Application Support/com.inkxel.amber/`), **never in the vault, never committed**:
- `openrouter.key` — the API key, `0600` (see [[decisions/2026-05-29-dev-key-storage-file-not-keychain]]). `OPENROUTER_API_KEY` env var overrides.
- `vault.path` — the configured vault folder. `AMBER_VAULT` env var overrides.

A gitleaks pre-commit hook (`.githooks/pre-commit`, enabled via `git config core.hooksPath .githooks`) blocks secret leaks — scans staged changes + hard-blocks Anthropic/OpenRouter/AWS key patterns.

## The auth hard line

🔒 API key only — **never** a consumer subscription OAuth token. Ban risk. See [[decisions/2026-05-29-api-key-only-never-oauth]].

## Why Tauri (not Electron)

Tucker already knows Tauri from the Jellybean project — same toolchain. cosi-platform's desktop thesis is still weighing Electron vs Tauri; Amber commits to Tauri for the bridge-product build. Cross-platform (Windows) comes free later.

## Context log

### 2026-05-31 — Article created
Documents the M0–M2 state: Tauri scaffold, OpenRouter streaming chat, file-based key + vault storage, vault retrieval. Frontend is still a single `App.tsx`; will componentize as M3 (command bar) adds a second window. See [[journal/2026-05-29-m0-m2-foundation]].
