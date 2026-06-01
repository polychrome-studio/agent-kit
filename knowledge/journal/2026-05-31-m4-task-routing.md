---
date: 2026-05-31
session: m4-task-routing
status: in-progress
related: [[architecture]], [[build-status]], [[vault-retrieval]], [[roadmap]]
---

# M4 — Task routing ("mode is the primitive")

## Context
M3 shipped + the persona/model were tuned (companion voice, Sonnet 4.6 default). M4 is
the cost lever: route each turn to the right model instead of paying frontier prices for
everything. Tucker's refinement (surfaced during the persona tuning, [[journal/2026-05-31-m3-command-bar]]):
don't build tone, model, and source-chip visibility as three settings — derive them from
ONE classification. So the unit of routing is a **mode**, not just a model.

## What changed
- New `src-tauri/src/router.rs`: `Mode { Quick, Companion, Research }` + the classifier.
- `vault.rs`: `system_prompt` → `context_block` — persona moved out (now per-mode in router),
  this just formats the index+notes block.
- `lib.rs`: `chat` now classifies first, emits a `Meta { mode, model }` event, composes the
  system message from `mode.persona()` + the context block, routes to `mode.model()`, and
  only emits `Sources` when `mode.show_sources()`.
- Frontend: `chat.ts` gains the `meta` event + `prettyModel()`; `App.tsx` and `CommandBar.tsx`
  show a small "✦ sonnet 4.6 · companion" label under the answer.

## The mode table (the three knobs off one classification)
| Mode | Model | Voice | Vault | Source chips |
|---|---|---|---|---|
| `quick` | `claude-haiku-4.5` | terse, result-only | skipped | hidden |
| `companion` (default) | `claude-sonnet-4.6` | the tuned companion voice | grounded | hidden |
| `research` | `claude-opus-4.8` | pragmatic, may name notes | grounded | **shown** |

## Decisions made
- **Hybrid classifier** (Tucker's pick). Instant heuristics handle the obvious cases so the
  common path (short conversational queries → Companion) has ZERO added latency; a cheap
  `claude-3.5-haiku` call breaks the tie ONLY for long, unsignalled queries. Heuristic tuned
  so the model call is the exception, not the rule — keeps the command bar snappy.
  - Quick signals: format/rewrite/translate/shorten/proofread… → Quick.
  - Research signals: "my notes", "according to", "sources", "dig into", "summarize the"… → Research.
  - ≤12 words and unsignalled → Companion (instant). Longer + unsignalled → model call.
  - Model-call failure defaults to Companion (never blocks the answer).
- **Persona lives in the router, not the vault.** Voice is a mode concern; retrieval is a vault
  concern. `Mode::persona()` is always injected (even with no vault hit) so Amber's register is
  consistent; the vault `context_block` is appended after it for vault-using modes.
- **Sources gated server-side.** Backend only emits the `Sources` event in research mode, so the
  existing chip UI hides automatically in companion/quick — matches Tucker's "I don't need to see
  it's pulling from the vault" in conversational use.
- **Models per tier:** Haiku 4.5 / Sonnet 4.6 / Opus 4.8 — Opus dropped to $5/$25 so "frontier
  when earned" is cheap now. Slugs confirmed live against the OpenRouter models API.

## What was tried and abandoned
- Considered classifying EVERY query with a model call (simpler) — rejected: adds ~300ms to the
  hot path and dulls the bar. Considered heuristics-only — rejected: misroutes nuance. Hybrid wins.

## Verification
- `cargo test router::` → 2 pass (heuristics route obvious cases; mode knobs correct).
- `cargo check` + `tsc --noEmit` + dev rebuild all clean; app running the current binary.
- **Pending: live confirm by Tucker** — type a format task (→ quick/haiku), a normal question
  (→ companion/sonnet, no chips), and a "what do my notes say about X" (→ research/opus, chips +
  pragmatic voice); watch the model label change. Headless can't drive the GUI.

## Open threads
- [ ] Live-verify the three modes route + label correctly, then flip M4 in [[build-status]].
- [ ] User-overridable mode (force research/companion) — the personalization story on [[roadmap]].
- [ ] Curated model-selection UI + admin token budget — parked on [[roadmap]], not in this cut.
- [ ] Classifier latency: the tie-break call is sequential before first token; could fire it
      speculatively or cache by query shape later.

## Related
- Touched articles: [[build-status]], [[roadmap]], [[vault-retrieval]]
