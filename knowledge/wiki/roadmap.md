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
- **M4 — task routing** (next): classify the task, pick model tier; surface which model answered (small label) for trust + cost awareness.
- **M5 — rituals**: port FOUNDRY rituals — morning briefing (`/today`-style), journaling, capture-to-vault — with write-back ("two outputs" rule).

## Deferred capabilities
- **Vault write-back** — opt-in, explicit, matching the vault's own conventions (frontmatter, wikilinks). Never silent.
- **Vector search** — only if index+grep recall visibly fails as the vault grows.
- **FAL media generation** — images/video; cosi-platform already wires FAL.
- **Keychain key storage** — at release, once the app is code-signed.
- **Windows build** — Tauri is cross-platform; free-ish later.

## UX / polish parking lot
- **Custom hotkey: double-tap right Shift** — Tucker's actual ask for summoning the command bar. Not reachable via `tauri-plugin-global-shortcut` (combinations only; no modifier-only, no left/right-side, no double-tap). Would need a macOS `CGEventTap`/`rdev`-style global key monitor + an Accessibility-permission prompt. Currently Option+Space. Revisit alongside a user-configurable-hotkey setting.
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
