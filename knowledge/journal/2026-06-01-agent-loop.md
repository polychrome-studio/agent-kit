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

## Related
- Touched articles: [[north-star]], [[build-status]], [[roadmap]]
