---
date: 2026-06-01
session: agent-loop
status: in-progress
related: [[north-star]], [[architecture]], [[build-status]], [[roadmap]]
---

# Agent runtime — the tool-use loop (the spine)

## Context
Per [[north-star]], the post-M4 goal is to turn Amber from chat+RAG into a genuinely agentic
backend. Tucker said "proceed with your recommended order" → push the M3/M4/icon/research/north-star
commits (done), then build the spine. This is that build.

## What changed
- **New `src-tauri/src/agent.rs`** — the model-driven tool-use loop. The model plans → calls a
  tool → sees the result → iterates → answers, up to `MAX_STEPS` (6).
- **Three tools** (OpenRouter function-calling): `search_vault(query)` (vault digest + paths),
  `read_note(path)` (full note, sandboxed to the vault root), `web_search(query)` (reuses
  OpenRouter's web plugin via a small sub-call — no new API key).
- **`vault.rs`** — added `search_digest` (path + 500-char snippets) and `read_note` (full, with
  path-escape guard). `context_block` is now dead (kept `#[allow(dead_code)]`); pre-injection is gone.
- **`router.rs`** — research dropped `:online` (web is a tool now); added `Mode::tools()`
  (quick = none); personas rewritten to tell the model about its tools and to search iteratively.
- **`lib.rs`** — `chat` slimmed to: resolve key → classify mode → emit `Meta` → `agent::run`.
  New `StreamEvent::Tool { name, arg }`. Made `ChatMessage`/`StreamEvent`/`OPENROUTER_URL`/
  `vault_path` crate-visible.
- **Frontend** — `chat.ts` gains the `tool` event + `toolLine()`; `App.tsx` + `CommandBar.tsx`
  render an "Amber is working" activity trail (🔎 searched your vault · X / 🌐 searched the web · X).

## Why this shape
- **Tools, not forced calls.** Old flow force-injected vault context (companion/research) and
  force-ran `:online` (research). Now the model *decides* — this is the "online when needed, don't
  cripple" outcome Tucker wanted, done right (vs blanket `:online` which forced a search every turn).
- **web_search via OpenRouter web plugin** — keeps it within the existing key/stack; no Tavily/Brave
  key to manage for v1. Can swap to a dedicated search API later if needed.
- **read_note is sandboxed** — canonicalize + `starts_with(vault root)` check; vault stays read-only.
- **Streaming tool-calls** are the fiddly part: `arguments` arrive as fragments across deltas,
  concatenated by `index`. Pulled the merge into a pure `merge_tool_calls` + unit-tested it.

## Verification
- `cargo test --lib` → 4 pass (router heuristics + knobs, **agent tool-call accumulator**, vault).
- `tsc --noEmit` clean; dev rebuild Finished clean, app running the current binary.
- **Pending live confirm by Tucker** (the real test — headless can't drive it):
  - research query → should see 🔎/📄/🌐 steps, web URLs cited alongside notes.
  - a recall question in companion → should silently search the vault, then answer in-voice.
  - "format this: …" (quick) → no tools, instant, one shot.

## Open threads
- [ ] Live-verify the loop end-to-end; then mark the spine done in [[build-status]].
- [ ] Latency: companion now may do a tool round-trip before answering recall questions (2 calls).
      Watch the felt snappiness; tune persona "don't over-search" if it searches too eagerly.
- [ ] Web citations: model inlines URLs today; could surface as chips like vault sources.
- [ ] Now that the loop exists, vault **write-back** and **rituals** become tools (M5) — and a
      `MAX_STEPS` hit should probably tell the user it stopped early rather than ending silently.

## Follow-up — first live test: web_search bug + stop button (same day)
Tucker live-tested. The loop **works** — it searched the vault iteratively and (when asked
about outside factors) called `web_search` twice. But two issues + a pile of design flags.

### Bug FIXED: web_search returned stale generic text, not live results
Diagnosed against OpenRouter with Tucker's key: the web plugin **does** run and attaches real,
current results as `message.annotations` (verified: a Feb-2026 Federal Register CARS-rule
withdrawal, a 2025 5th-Circuit ruling). But my `web_search` returned `message.content` — and the
cheap haiku sub-model *ignored the injected results and answered from stale training data*, even
disclaiming "I cannot search the web." So the main model got generic guidance.
**Fix:** `web_search` now harvests `annotations[].url_citation` (url + title + content, capped
1200 chars each, UTF-8-safe) and returns THOSE; the sub-model's prose is discarded (`max_tokens:16`,
it only exists to trigger the plugin search). Confirmed annotations carry real fresh content.

### Added: stop button (Tucker asked)
Cooperative cancel: `CancelFlag(AtomicBool)` managed state; `stop_chat` command sets it; the agent
loop checks it at the top of each step AND mid-stream (between SSE chunks → drops the stream,
reqwest aborts). `chat` clears it at turn start. UI: composer button flips to ■ while streaming
(App), palette shows a ■ in the input row (CommandBar). One global flag (a turn is one-at-a-time).

### Design flags from Tucker — DEFERRED to an agent-UX polish pass (he said "design fixes later")
Parked in [[roadmap]], NOT built (only a light CSS tone-down done now):
- **Run-on narration.** Inter-step "Let me dig into…" narration concatenates into the answer with
  no spacing ("…challenges.Let me dig…"). Root cause: every step's content tokens stream into one
  bubble. Real fix = separate *thinking* (inter-step narration) from the *final answer*.
- **Thinking should fade + collapse** into a dropdown (ChatGPT/Claude pattern), not stay inline.
  Same fix as above — once thinking is separated, it can collapse.
- **Step icons + "grounded in" chips visually dominate the answer text.** Toned down now (steps →
  faint borderless lines that brighten on hover; chips → muted until hover). Full visual-hierarchy
  pass still wanted.

## Related
- Touched articles: [[north-star]], [[build-status]], [[roadmap]]

### 10:19 — 27bc0f5
Agent runtime: model-driven tool-use loop (the north-star spine)
files: knowledge/journal/2026-06-01-agent-loop.md, knowledge/journal/2026-06-01-session.md, knowledge/wiki/build-status.md, knowledge/wiki/roadmap.md, src-tauri/src/agent.rs, src-tauri/src/lib.rs, src-tauri/src/router.rs, src-tauri/src/vault.rs, src/App.css, src/App.tsx, src/CommandBar.tsx, src/lib/chat.ts
