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
| **M3 — Command bar** | the signature global-hotkey invocation UX | ✅ done (2026-05-31) — verified live |
| **M4 — Task routing** | the cost lever — cheap vs frontier by task | 🟡 implemented (2026-05-31) — pending live confirm |
| **M5 — Rituals** | the FOUNDRY-specific differentiation | ⬜ ongoing |

## Done — detail

- **M0** — Tauri 2 + Vite + React-TS scaffold merged around the pre-seeded docs; identifiers set (`com.inkxel.amber`, productName `Amber`); repo at `github.com/inkxel/amber` (private, `main`); gitleaks hook active. Both sides compile clean.
- **M1** — `chat()` streams OpenRouter SSE through a Tauri `Channel`; key stored locally (file, not keychain — see [[decisions/2026-05-29-dev-key-storage-file-not-keychain]]); ember-glow chat panel + settings/paste-key screen. Verified live by Tucker.
- **M2** — index + grep vault retrieval (`vault.rs`), grounded answers with source chips, vault-path config + status pill + native folder picker. See [[vault-retrieval]]. Verified live + smoke test.

## M3 (command bar) — implemented, how it's shaped

Option+Space (global shortcut, registered in Rust) toggles a second `palette` window: borderless, transparent, always-on-top, config-defined + hidden at startup so the summon is instant. Type → reuses the M2 `chat` command → answer streams in the bar grounded in the vault; Esc or blur dismisses. Ephemeral one-shot (no shared thread). Both windows load one Vite bundle, routed by `getCurrentWindow().label` in `src/main.tsx`; chat plumbing was extracted to `src/lib/chat.ts` and shared. Full rationale + the two open threads: [[journal/2026-05-31-m3-command-bar]].

**Verified live** by Tucker (2026-05-31): Option+Space → bar → streamed answer → Esc. Persona/model were then tuned (companion voice + Sonnet 4.6 default) — see [[journal/2026-05-31-m3-command-bar]] "Follow-on".

### M3 caveats / deferred
- **Hotkey** is hardcoded Option+Space. Double-tap-right-Shift was requested but isn't reachable via the global-shortcut plugin (needs a `CGEventTap` + Accessibility) → parked on [[roadmap]]. User-configurable hotkey is a later add.
- **Palette** is fixed-height (480px, internal scroll) — auto-grow with the answer is parked.

## M4 (task routing) — implemented, how it's shaped

"Mode is the primitive": one classification per turn drives model + voice + source-chip visibility. `src-tauri/src/router.rs` — `Mode { Quick, Companion, Research }` → `claude-haiku-4.5 / claude-sonnet-4.6 / claude-opus-4.8`. **Hybrid classifier:** instant heuristics for obvious cases (hot path stays snappy), a cheap `claude-3.5-haiku` tie-break only for long unsignalled queries. `chat` emits a `Meta { mode, model }` event → both surfaces show a "✦ sonnet 4.6 · companion" label; `Sources` chips emit only in research mode. Persona moved from `vault.rs` into `Mode::persona()`. Full rationale: [[journal/2026-05-31-m4-task-routing]].

**Pending:** live confirm by Tucker (route a format task / a question / a "what do my notes say…" and watch the label + chips change).

## Hardcoded / deferred (revisit at the named milestone)

- ~~**Model** is hardcoded `anthropic/claude-3.5-haiku` → made dynamic at **M4**.~~ ✅ done — task-routed by mode (M4).
- **Keychain** key storage → returns at **release** (code-signed build).
- **Vault write-back** ("every task produces two outputs") → **M5**, opt-in.
- **Vector search** → only if the vault outgrows index+grep.

## Context log

### 2026-05-31 — Tracker created
M0–M2 done and verified; M3 is next.
