---
name: build-status
type: topic
created: 2026-05-31
last_updated: 2026-05-31
confidence: high
related: [[architecture]], [[vault-retrieval]], [[roadmap]]
---

# Build status — M0→M5 milestone tracker

The milestones (from `docs/build-plan.md`) are ordered so each de-risks the next. Current state of the build:

| Milestone | What it proves | Status |
|---|---|---|
| **M0 — Scaffold + repo** | the toolchain works end to end | ✅ done (2026-05-29) |
| **M1 — Talk to a model** | model wiring + streaming + key handling | ✅ done (2026-05-29) |
| **M2 — Read the vault** | the knowledge layer — answers from *your* brain | ✅ done (2026-05-29 → 31) |
| **M3 — Command bar** | the signature global-hotkey invocation UX | ⬜ next |
| **M4 — Task routing** | the cost lever — cheap vs frontier by task | ⬜ |
| **M5 — Rituals** | the FOUNDRY-specific differentiation | ⬜ ongoing |

## Done — detail

- **M0** — Tauri 2 + Vite + React-TS scaffold merged around the pre-seeded docs; identifiers set (`com.inkxel.amber`, productName `Amber`); repo at `github.com/inkxel/amber` (private, `main`); gitleaks hook active. Both sides compile clean.
- **M1** — `chat()` streams OpenRouter SSE through a Tauri `Channel`; key stored locally (file, not keychain — see [[decisions/2026-05-29-dev-key-storage-file-not-keychain]]); ember-glow chat panel + settings/paste-key screen. Verified live by Tucker.
- **M2** — index + grep vault retrieval (`vault.rs`), grounded answers with source chips, vault-path config + status pill + native folder picker. See [[vault-retrieval]]. Verified live + smoke test.

## Next — M3 (command bar)

Global hotkey (Tauri global-shortcut plugin) → a second borderless, always-on-top window = the Raycast-style command bar. Type → routes to the model → answer streams in the bar; Esc dismisses. This is the signature invocation UX and the first multi-window work (architecture will need to componentize the single `App.tsx`).

## Hardcoded / deferred (revisit at the named milestone)

- **Model** is hardcoded `anthropic/claude-3.5-haiku` → made dynamic at **M4**.
- **Keychain** key storage → returns at **release** (code-signed build).
- **Vault write-back** ("every task produces two outputs") → **M5**, opt-in.
- **Vector search** → only if the vault outgrows index+grep.

## Context log

### 2026-05-31 — Tracker created
M0–M2 done and verified; M3 is next.
