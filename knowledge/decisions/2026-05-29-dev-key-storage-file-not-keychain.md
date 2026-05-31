---
date: 2026-05-29
status: accepted
deciders: tucker
related: [[architecture]]
---

# Decision: Store the API key in a 0600 app-config file during dev, not the macOS keychain

## Context
M1 first stored the OpenRouter key in the macOS keychain via the `keyring` crate. It worked once, then on every relaunch macOS re-prompted for the **login keychain** password and never stuck. Root cause: an unsigned `npm run tauri dev` binary changes code-signing identity on every rebuild, so the keychain treats each build as a different app and re-evaluates the ACL. "Always Allow" can't persist across rebuilds, and the login-keychain password (which can drift out of sync from the macOS login password) was being rejected. Every `get`/`set` on startup and save re-triggered the prompt — an unwinnable loop for a dev build. The build plan already sanctioned a `.env`/file path "for first light."

## Decision
**Store the key as a `0600` file in the app config dir** (`~/Library/Application Support/com.inkxel.amber/openrouter.key`), with an `OPENROUTER_API_KEY` env-var override. The keychain returns at the **release** stage, once the app is code-signed with a stable identity.

## Consequences
- **Positive:** Zero prompts; the key persists silently across every relaunch and rebuild; `OPENROUTER_API_KEY` lets you skip the UI entirely in dev.
- **Negative:** The key sits in plaintext on disk (owner-only `0600`, outside the repo). Acceptable for a revocable pay-per-token personal dev key; hardened at packaging time.
- **Neutral:** Same `set_api_key` / `has_api_key` / `clear_api_key` command surface — only the storage backend changed. The vault-path setting reuses the same app-config-file pattern.

## Dissent / Alternatives Considered
- **Keep the keychain + ad-hoc sign the dev binary** — fragile; signature/path still change per rebuild, so prompts persist. Not worth it for dev.
- **Try-keychain-then-fall-back-to-file** — over-engineered for the current stage; revisit when the release build reintroduces the keychain.

## Sources
- [[journal/2026-05-29-m0-m2-foundation]]
- [[2026-05-29-api-key-only-never-oauth]] — keys are API keys, stored locally, never committed
