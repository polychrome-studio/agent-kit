---
date: 2026-05-29
session: m0-m2-foundation
status: shipped
related: [[architecture]], [[vault-retrieval]], [[build-status]]
---

# M0–M2 foundation — scaffold, model wiring, vault grounding

*(Backfilled 2026-05-31 from the build sessions of 2026-05-29.)*

## Context
First dev sessions in the repo. The folder was pre-seeded (by a FOUNDRY session) with `docs/` + config only — no code. Goal: execute the milestone ladder from `docs/build-plan.md`, each milestone de-risking the next.

## What changed

**M0 — Scaffold + repo.**
- Scaffolded Tauri 2 + Vite + React-TS into a temp folder and merged around the existing docs (`rsync --ignore-existing`) so `CLAUDE.md` / `.gitignore` / `docs/` survived. The scaffolder needs a TTY — used the `-y --identifier --tauri-version 2` non-interactive flags.
- Renamed `amber-scaffold` → `amber` / `amber_lib` across `package.json`, `Cargo.toml`, `tauri.conf.json`, `main.rs`; productName + window title → `Amber`. Kept our richer `.gitignore` (superset of Tauri's).
- `git init`, gitleaks hook on (`core.hooksPath .githooks`), repo created at `github.com/inkxel/amber` (private), branch renamed `master` → `main`.

**M1 — Talk to a model.**
- `chat()` streams OpenRouter SSE through `reqwest` into a Tauri `Channel<StreamEvent>`. HTTP call in Rust → key never enters the webview, no CSP/CORS.
- Key storage: first the macOS keychain (`keyring`), then **switched to a `0600` app-config file** after the keychain re-prompted on every unsigned-dev-build relaunch. See [[decisions/2026-05-29-dev-key-storage-file-not-keychain]].
- React: ember-glow chat panel + settings/paste-key screen. Verified live by Tucker (real OpenRouter answer streamed in).

**M2 — Read the vault.**
- `vault.rs`: read an index file + keyword-score every `.md`, boost filename + `knowledge/wiki/` paths, return top 3 under a size budget. Inject an index+notes system message; instruct the model to cite notes.
- `Sources` stream event → amber source chips under each answer. `get_vault_path` / `set_vault_path` (validated dir, `AMBER_VAULT` override), persisted in app config.
- Smoke test against real FOUNDRY ranks `knowledge/wiki/praxis-platform.md` #1 for a praxis query (3,091 files, ~0.7s).

- Paths touched: `src-tauri/src/lib.rs`, `src-tauri/src/vault.rs`, `src-tauri/Cargo.toml`, `src/App.tsx`, `src/App.css`, `src-tauri/tauri.conf.json`.
- Subsystems: [[architecture]], [[vault-retrieval]].

## Decisions made
- **A2 / OpenRouter architecture** — [[decisions/2026-05-29-openrouter-a2-architecture]]
- **API key only, never OAuth** — [[decisions/2026-05-29-api-key-only-never-oauth]]
- **Dev key in a file, not the keychain** — [[decisions/2026-05-29-dev-key-storage-file-not-keychain]]
- **Index + grep retrieval, no vector DB** — [[decisions/2026-05-29-index-grep-retrieval-no-vector-db]]

## What was tried and abandoned
- **macOS keychain for the dev key** — worked once, then re-prompted for the login-keychain password on every relaunch because the unsigned dev binary changes identity each rebuild. Dropped for a file; keychain returns at release.
- **Direct `npm create tauri-app` into the live folder** — scaffolder wants an empty dir and a TTY; solved with a temp folder + merge and non-interactive flags.

## Open threads
- [ ] **M3 — command bar** is next: global hotkey + second borderless window. Will force componentizing `App.tsx`. See [[build-status]].
- [ ] Vault-connection state had no UI signal — caused a false "it's not reading my vault" report next session (fixed 2026-05-31). See [[journal/2026-05-31-knowledge-layer-created]].

## Related
- Touched articles: [[architecture]], [[vault-retrieval]], [[build-status]]
- `docs/build-plan.md` — the milestone ladder; `KICKOFF.md` — the M0 runbook
