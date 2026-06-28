---
name: roadmap
type: roadmap
created: 2026-05-31
last_updated: 2026-06-01
confidence: medium
related: [[build-status]]
---

# Roadmap — the parking lot

"At some point we should…" — ideas surfaced during build sessions that aren't scheduled yet. Not commitments. When one graduates into active work, it moves to [[build-status]] and gets a journal entry. Append freely; don't delete (strike through with a note if dropped).

> **Organizing principle (2026-06-01): the agentic-runtime arc.** Per [[north-star]], the post-M4 goal is to turn Amber from chat+RAG into a genuinely agentic backend (Hermes/OpenClaw/Pi-class) behind a beautiful surface. The **spine** is a model-driven tool-use loop; web-search, vault write-back, rituals, and proactivity are all *tools* that hang off it. So **model-decided web search (below) is brick #1**, not just a deferred nicety — it's where the agent loop starts. Bend new work toward this arc rather than adding more one-shot chat features.

## Near-term (next few milestones)
- ~~**M3 — command bar**: global hotkey → borderless always-on-top window. Componentize `App.tsx` for multi-window.~~ → implemented 2026-05-31, see [[build-status]] + [[journal/2026-05-31-m3-command-bar]].
- ~~**M4 — task routing**: classify the task, pick model tier; surface which model answered.~~ → implemented 2026-05-31 (hybrid classifier, `Mode{Quick,Companion,Research}` driving model+voice+chips). See [[build-status]] + [[journal/2026-05-31-m4-task-routing]]. The "mode is the primitive" design below is now built; what remains parked is the *user-facing* control (override mode, curated model picker, token budget) under "Personalization & control".
  - **Mode is the primitive (built 2026-05-31).** The classifier outputs a **mode** that drives three knobs at once: model tier, voice/tone register, source-chip visibility. *Research* → straight + pragmatic, sources shown, Opus. *Companion* → the tuned companion voice, sources hidden, Sonnet. *Quick* → terse, no vault, Haiku. Still open: letting the user *override* the inferred mode (see Personalization & control).
- **M5 — rituals**: port FOUNDRY rituals — morning briefing (`/today`-style), journaling, capture-to-vault — with write-back ("two outputs" rule).

## Deferred capabilities
- ~~**Model-decided web search (tool-use)**~~ → **BUILT 2026-06-01.** The agent tool-use loop ([[journal/2026-06-01-agent-loop]]) replaced forced `:online` with a `web_search` tool the model calls only when it judges it needs to — plus `search_vault`/`read_note`. The spine is in; remaining web polish: surface web citations as chips, and consider a dedicated search API vs the OpenRouter web-plugin sub-call.
- **Vault write-back** — opt-in, explicit, matching the vault's own conventions (frontmatter, wikilinks). Never silent. Becomes a tool on the agent loop ([[journal/2026-06-01-agent-loop]]). **Testing: Tucker will duplicate his FOUNDRY vault** to a safe sandbox to point Amber at before any write goes near the live vault (stated 2026-06-02). Needs a design huddle (what writes, where, format) before building.
- **Vector search** — only if index+grep recall visibly fails as the vault grows. (If/when this lands, embeddings are the first thing to run on the local tier — see below.)
- **Local compute, bundled** — embed a local inference engine *into* the app (candle/llama.cpp on Metal, Option B) so the cheap always-on tier (embeddings, summaries, rituals, classification) runs locally at near-zero cost while cloud Claude handles heavy reasoning. Net-new ground — Amber is cloud-only today. Decision-in-waiting, reasoned out ahead of the work in [[decisions/2026-06-06-local-compute-bundled]]; the one thing to keep in mind now is that the local-vs-cloud choice belongs in the `router.rs` seam. Completes the "local/sovereign" half of [[north-star]].
- **FAL media generation** — images/video; cosi-platform already wires FAL.
- **Keychain key storage** — at release, once the app is code-signed.
- **Windows build** — *low priority* (only 2–3 Windows users on the team). Tauri was chosen partly for this: the app logic ports for free from one codebase. What is NOT free, the known checklist (surfaced 2026-06-01):
  - **Hotkey** — Option+Space is Mac. On Windows `Alt+Space` opens the window system menu, so pick a different default (e.g. `Ctrl+Space`). Needs per-platform hotkey config.
  - **Palette transparency/rounded look** — uses the macOS private-API flag (`macOSPrivateApi` + `macos-private-api` feature); Windows does transparency via acrylic/mica. Needs platform-specific window effects.
  - **Signing** — Apple notarization vs Windows Authenticode cert: two separate build/sign pipelines.
  - **Dock-off + tray** (see companion behaviors below) are macOS-shaped — Windows equivalents are `skipTaskbar` + a system-tray icon (same idea, different mechanism).
  - Minor: WebView2 (Edge) renders instead of WKWebView; key storage abstraction differs (Keychain vs Credential Manager — the keyring plugin covers both).
  - **Effort:** low-to-moderate — mostly hotkey + window chrome + a second build/sign lane. Deliberate "turn on later," not a rewrite.

## Personalization & control (post-M4, surfaced 2026-05-31)
- **User-definable personality / tone.** Common + expected (cf. ChatGPT Personalization: base style + characteristics + custom instructions). Amber should let the user shape Amber's voice — *but* the lever is per-**mode** (see M4 "mode is the primitive"), not one global slider: research wants pragmatic/straight, a thought-partner wants companion. Inferred from task type, user-overridable. The M2 persona prompt (`vault.rs::system_prompt`) is the current single hardcoded voice — this generalizes it.
- **Curated model selection with guardrails — NOT free choice.** Tucker's key insight: *"if I give people full choice they will always go for the biggest most expensive."* So: the **harness decides the tier** the task needs; the user only picks from a **short curated shortlist within that tier**, framed as intent ("fast" vs "deep"), never as raw model names. Admin holds the ceiling + (eventually) a **per-user token budget / limit** as a control surface. This is the M4 cost lever with a human-friendly face. Directly relevant to cosi-platform (multi-user, cost governance) — Amber is the proving ground.

## Raycast-style window unification (direction set 2026-06-02 — design-led, see [[design]])
Collapse the two-window model (decorated `main` + frameless `palette`) into ONE frameless morphing surface, the Raycast pattern: **no traffic-light window anywhere**; ⌥Space opens the compact bar; on send it **expands into the full chat surface** holding conversation history (multiple threads), the journal, and future features. Supersedes M3's two-window design. Net-new infra it implies: **conversation persistence** (today turns are ephemeral) + the expanded layout/nav. Tucker is designing the visual in [[design]]'s Figma file — build replicates from there. Pairs with the native macOS behaviors below (dock-off + tray + frameless = the Raycast model).

## Native macOS companion behaviors (next clean chunk — surfaced 2026-06-01)
A self-contained increment that makes Amber behave like a real menubar companion (Raycast-pattern). All three are standard Tauri 2 / Rust — confirmed achievable, no fight with the stack:
- **Menu bar (tray) icon** — `TrayIcon` API + the `tray-icon` cargo feature; dropdown menu (Open Amber / Command bar / Quit). Becomes the primary entry point when the dock icon is off.
- **Toggle the dock icon from Settings** — macOS activation policy: `set_activation_policy(Accessory)` hides it (menubar-only agent), `Regular` shows it; runtime-callable so a Settings switch drives it. Default **on**, with a hide toggle (Tucker to confirm if he'd rather ship off). Natural pairing: dock off → tray + Option+Space are how you summon it.
- **Close ≠ quit** — intercept `WindowEvent::CloseRequested` → `prevent_close()` + `window.hide()`; app lives in the tray, real quit via tray menu (optionally also intercept Cmd+Q). Note: switching activation policy at runtime while a window is open can have minor focus quirks — fine in practice.
- Caveat: dock-toggle + tray are macOS-shaped; Windows equivalents (`skipTaskbar` + system tray) live in the Windows-build checklist above.

## Agent UX polish (flagged by Tucker 2026-06-01 on first live test — design pass, deferred)
The agent loop works; these are how it *reads*. He explicitly said "design fixes later, but flag as I see them." A light CSS tone-down shipped 2026-06-01; the structural ones below remain.
- **Separate thinking from the answer.** Inter-step narration ("Let me dig into…") streams into the same bubble as the final answer, concatenated with no spacing (run-on: "…challenges.Let me dig…"). Root cause: every loop step's content tokens go to one place. Fix: route inter-step narration to a distinct *thinking* channel, only the final step's content to the answer bubble.
- **Thinking should fade out + collapse into a dropdown** (ChatGPT/Claude pattern) once the answer starts — not stay inline. Depends on the separation above. This is the big perceived-quality win.
- **Visual hierarchy: step trail + "grounded in" chips were louder than the answer.** Toned down 2026-06-01 (steps → faint borderless lines, brighten on group-hover; chips → muted until hover). A proper design pass still wanted (iconography, spacing, maybe collapse the step trail too).

## UX / polish parking lot
- ~~**Source chips visibility should follow mode**~~ → done in M4 (research shows chips, companion/quick hide them — gated server-side). See [[journal/2026-05-31-m4-task-routing]].
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

### 2026-06-01 — Platform + native-behavior threads parked
Captured two discussion threads as parked work: (1) the **Windows-build checklist** (hotkey, palette transparency, signing, tray/dock equivalents) — low priority, 2–3 Windows users; (2) **native macOS companion behaviors** (tray icon, Settings toggle for the dock icon via activation policy, close-to-hide) — confirmed all doable in Rust/Tauri, a clean self-contained next chunk. Also struck the M4-completed "source chips follow mode" item. Context: the "why Rust not Swift" Q (Tauri = cross-platform + coherent with cosi-platform + Tucker knows it) — rationale already lives in `docs/architecture-and-auth.md` + [[architecture]].
