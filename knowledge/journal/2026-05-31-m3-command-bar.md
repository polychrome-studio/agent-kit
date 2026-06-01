---
date: 2026-05-31
session: m3-command-bar
status: shipped
related: [[architecture]], [[build-status]], [[vault-retrieval]]
---

# M3 — The command bar (global hotkey → floating Raycast-style window)

## Context
M0–M2 were done and verified (scaffold, streaming OpenRouter chat, vault retrieval).
M3 is the signature invocation UX: a global hotkey summons a borderless, always-on-top
command bar; you type a query; the answer streams in the bar; Esc dismisses. This was the
first multi-window work, so it forced the single `App.tsx` to be componentized.

## What changed
- Paths touched: `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`,
  `src-tauri/capabilities/palette.json` (new), `src/main.tsx`, `src/App.tsx`,
  `src/App.css`, `src/CommandBar.tsx` (new), `src/lib/chat.ts` (new).
- Subsystems: new **command-bar** surface; refactor of the chat plumbing into a shared lib.
- Behavior: Option+Space toggles a second `palette` window; type → streamed answer grounded
  in the vault (reuses the M2 `chat` command + Sources event); Esc or blur dismisses.

## How it's built (the shape that matters)
- **Two windows, one bundle.** Both `main` and `palette` load the same Vite bundle.
  `src/main.tsx` reads `getCurrentWindow().label` and renders `<App/>` for `main`,
  `<CommandBar/>` for `palette`. No second HTML entry — label-routing is the Tauri-idiomatic
  way and keeps one build.
- **Palette window is config-defined**, not runtime-created: `decorations:false`,
  `transparent:true`, `alwaysOnTop:true`, `skipTaskbar:true`, `visible:false`, `center`,
  `focus:false`. Defined in `tauri.conf.json` so it's warm at startup → the hotkey is instant.
  Transparency needs `app.macOSPrivateApi:true` + the `macos-private-api` cargo feature on
  `tauri` (both added). The rounded floating card = transparent body + a `border-radius`
  card filling the viewport (`body.palette-window { background: transparent }`).
- **Global shortcut registered in Rust** (`tauri-plugin-global-shortcut`), not JS. Handler
  toggles the palette: visible → hide; hidden → show + focus + `emit("palette:show")`.
  The emit tells the React side to reset (one-shot grammar) and refocus the input.
  Registration failure is **non-fatal** — logged, app still launches — so a hotkey conflict
  can never brick startup.
- **Shared `src/lib/chat.ts`** — `streamChat(messages, handlers)` wraps the Channel/invoke
  handshake. Both surfaces use it; `App.tsx`'s inline Channel logic was removed. This is the
  componentization M3 was supposed to force.
- **Capabilities:** added `capabilities/palette.json` scoped to the `palette` window with
  `core:window:allow-hide/show/set-focus` (the bar hides itself on Esc/blur from JS). App
  commands like `chat` aren't ACL-gated in Tauri 2, so no extra permission was needed there.

## Decisions made
- **Hotkey = Option+Space.** Tucker asked for **double-tap right Shift**; that's not reachable
  with the global-shortcut plugin (it registers key *combinations* only — no modifier-only
  press, no left/right-side distinction, no double-tap). Doing it would need a macOS
  `CGEventTap` + an Accessibility-permission prompt — a real detour off-plan. Used his
  fallback (Option+Space) and parked double-tap-Shift on [[roadmap]].
- **Command bar scope = ephemeral one-shot** (Tucker's pick). Each summon is a fresh query;
  no shared thread with the main window. Pure Raycast grammar; matches the M3 spec.
- **Dismiss on blur**, not just Esc — the Raycast grammar (click away = gone). Implemented via
  `onFocusChanged`. If it's annoying in dev (devtools steal focus), it's a one-line guard.

## What was tried and abandoned
- Considered a separate `palette.html` Vite entry — rejected for label-routing (one bundle,
  less config, the standard Tauri-2 multi-window pattern).
- Considered creating the palette window at runtime on first hotkey — rejected for a
  config-defined hidden window so the first summon has zero cold-start.

## Verification
- `tsc --noEmit` clean; `cargo check` clean; `npm run build` (prod bundle) clean.
- `npm run tauri dev` boots with **no panic** — both windows construct (transparent +
  capability wiring valid) and Option+Space registers without error.
- **Pending: live GUI confirm by Tucker** — press Option+Space, see the bar, watch an answer
  stream, Esc to dismiss. Headless can't drive a global hotkey / floating window.

## Open threads
- [ ] Tucker live-verifies the full loop; then flip M3 to "verified live" in [[build-status]].
- [ ] Palette is fixed-height (480px) with an internal scroll; could auto-grow with the answer
      later (needs window-resize permission). Parked.
- [ ] Hotkey should become user-configurable (Settings) eventually — hardcoded for now.

## Follow-on (same session) — persona + model tuning, after M3 live-verified
M3 worked live (Tucker confirmed). But the *first answer* felt robotic / vault-first /
over-citing. Root cause was two things, not one:
- **Model:** default was `anthropic/claude-3.5-haiku` (the M1 placeholder) — too small/literal
  to weave knowledge in like a companion. Bumped default to **`anthropic/claude-sonnet-4.6`**
  (`lib.rs`). Confirmed live OpenRouter slugs + pricing first; M4's router will reclaim the
  cheap tier for trivial tasks. Sonnet is the conversational sweet spot.
- **Prompt:** `vault.rs::system_prompt` literally said "Answer using the CONTEXT… cite the note
  filename(s)." That *is* a retrieval bot. Rewrote it to a **companion frame**: the notes are
  Amber's *memory of Tucker*, not documents to quote; never say "according to your vault," never
  cite filenames unless asked; warm, direct, sentences-not-paragraphs; weave in what he has in
  flight and offer to go deeper. Result was night-and-day (the "you've got three dogs — Nova's
  the chaos agent… what did you want to dig into?" reply). Tucker: "much much better."

### Product direction surfaced (parked to [[roadmap]], not built)
- **Mode is the primitive.** Tone, model tier, and source-chip visibility are all outputs of
  one "what kind of work is this" classification — which M4's task router already computes.
  Research mode → straight + cite + sources visible; thought-partner → companion + hidden. Build
  *mode*, not three separate settings.
- **User-definable personality** (ChatGPT-Personalization-style) — but per-mode, not one global
  slider. Generalizes the now-single hardcoded persona prompt.
- **Curated model selection, never free choice** — Tucker: "if I give people full choice they
  will always go for the biggest most expensive." Harness picks the *tier*; user picks intent
  ("fast"/"deep") from a curated shortlist; admin holds the ceiling + a token budget. The M4
  cost lever with a human face. Feeds cosi-platform's cost governance.

## Related
- Touched articles: [[build-status]], [[roadmap]]

### 15:34 — 77db663
M3: command bar — global hotkey, floating palette window, shared chat lib
files: knowledge/journal/2026-05-31-m3-command-bar.md, knowledge/wiki/build-status.md, knowledge/wiki/roadmap.md, src-tauri/Cargo.lock, src-tauri/Cargo.toml, src-tauri/capabilities/palette.json, src-tauri/src/lib.rs, src-tauri/tauri.conf.json, src/App.css, src/App.tsx, src/CommandBar.tsx, src/lib/chat.ts, +1 more

### 20:30 — 2c70fa9
Persona + model tuning: companion voice, default to Sonnet 4.6
files: knowledge/journal/2026-05-31-m3-command-bar.md, knowledge/wiki/roadmap.md, src-tauri/src/lib.rs, src-tauri/src/vault.rs

### 20:30 — 86b3f4e
Mark M3 verified live in build-status
files: knowledge/journal/2026-05-31-m3-command-bar.md, knowledge/wiki/build-status.md
