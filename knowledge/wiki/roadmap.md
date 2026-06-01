---
name: roadmap
type: roadmap
created: 2026-05-31
last_updated: 2026-05-31
confidence: medium
related: [[build-status]]
---

# Roadmap — the parking lot

"At some point we should…" — ideas surfaced during build sessions that aren't scheduled yet. Not commitments. When one graduates into active work, it moves to [[build-status]] and gets a journal entry. Append freely; don't delete (strike through with a note if dropped).

## Near-term (next few milestones)
- ~~**M3 — command bar**: global hotkey → borderless always-on-top window. Componentize `App.tsx` for multi-window.~~ → implemented 2026-05-31, see [[build-status]] + [[journal/2026-05-31-m3-command-bar]].
- ~~**M4 — task routing**: classify the task, pick model tier; surface which model answered.~~ → implemented 2026-05-31 (hybrid classifier, `Mode{Quick,Companion,Research}` driving model+voice+chips). See [[build-status]] + [[journal/2026-05-31-m4-task-routing]]. The "mode is the primitive" design below is now built; what remains parked is the *user-facing* control (override mode, curated model picker, token budget) under "Personalization & control".
  - **Mode is the primitive (built 2026-05-31).** The classifier outputs a **mode** that drives three knobs at once: model tier, voice/tone register, source-chip visibility. *Research* → straight + pragmatic, sources shown, Opus. *Companion* → the tuned companion voice, sources hidden, Sonnet. *Quick* → terse, no vault, Haiku. Still open: letting the user *override* the inferred mode (see Personalization & control).
- **M5 — rituals**: port FOUNDRY rituals — morning briefing (`/today`-style), journaling, capture-to-vault — with write-back ("two outputs" rule).

## Deferred capabilities
- **Vault write-back** — opt-in, explicit, matching the vault's own conventions (frontmatter, wikilinks). Never silent.
- **Vector search** — only if index+grep recall visibly fails as the vault grows.
- **FAL media generation** — images/video; cosi-platform already wires FAL.
- **Keychain key storage** — at release, once the app is code-signed.
- **Windows build** — Tauri is cross-platform; free-ish later.

## Personalization & control (post-M4, surfaced 2026-05-31)
- **User-definable personality / tone.** Common + expected (cf. ChatGPT Personalization: base style + characteristics + custom instructions). Amber should let the user shape Amber's voice — *but* the lever is per-**mode** (see M4 "mode is the primitive"), not one global slider: research wants pragmatic/straight, a thought-partner wants companion. Inferred from task type, user-overridable. The M2 persona prompt (`vault.rs::system_prompt`) is the current single hardcoded voice — this generalizes it.
- **Curated model selection with guardrails — NOT free choice.** Tucker's key insight: *"if I give people full choice they will always go for the biggest most expensive."* So: the **harness decides the tier** the task needs; the user only picks from a **short curated shortlist within that tier**, framed as intent ("fast" vs "deep"), never as raw model names. Admin holds the ceiling + (eventually) a **per-user token budget / limit** as a control surface. This is the M4 cost lever with a human-friendly face. Directly relevant to cosi-platform (multi-user, cost governance) — Amber is the proving ground.

## UX / polish parking lot
- **Custom hotkey: double-tap right Shift** — Tucker's actual ask for summoning the command bar. Not reachable via `tauri-plugin-global-shortcut` (combinations only; no modifier-only, no left/right-side, no double-tap). Would need a macOS `CGEventTap`/`rdev`-style global key monitor + an Accessibility-permission prompt. Currently Option+Space. Revisit alongside a user-configurable-hotkey setting.
- **Source chips visibility should follow mode** — currently always shown ("GROUNDED IN" + full paths). Tucker: "I don't need to see that it's pulling this from the vault." Hide in companion/thought-partner mode, show in research mode (ties to M4 "mode is the primitive").
- Markdown rendering in the chat panel (currently plain text / pre-wrap).
- Command-bar palette auto-grow with the answer (fixed 480px today); needs window-resize permission.
- Conversation history persistence + multiple threads.
- Design polish pass — the real "much better looking" ember-glow goal.
- Read-order heuristic for retrieval: if the vault has a handoff/journal convention, read the most recent handoff first (per `docs/knowledge-layer.md`).
- Type-aware ingestion when write-back lands (transcript vs article vs decision templates).

## Strategic / open
- Eventual `Collier-Simon/amber` repo transfer if Amber becomes the official CoSi bridge product (same playbook Praxis→cosi-platform followed).
- OpenRouter data governance / SOC 2 once Amber touches client data (personal v1 is fine).
- A guardrail test asserting nothing writes under the vault root (offered to Tucker; not yet built) — would make read-only enforced by construction.

## Context log

### 2026-05-31 — Created
Seeded from the build plan's "Later" section + threads raised through M2.
