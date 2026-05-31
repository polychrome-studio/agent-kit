---
date: 2026-05-31
session: knowledge-layer-created
status: shipped
related: [[knowledge-layer]], [[vault-retrieval]]
---

# Knowledge layer created — mirroring the cosi-platform build-documentation system

## Context
Tucker keeps a code-side knowledge layer inside every larger project (e.g. `/Users/tucker/Code/cosi/cosi-platform/knowledge/`) so the build has documentation and context session-to-session. Amber didn't have one yet. A quick detour mid-build (between M2 and M3) to set up the same system.

This session also closed out an M2 UX gap that surfaced when Tucker first tested vault grounding.

## What changed

**Vault-connection UX (closing the M2 thread).**
- First live grounding test returned model-only answers with no source chips. Diagnosis (via checking the app-config dir directly): the `vault.path` file didn't exist — the vault was never connected. The settings field's placeholder (`/Users/tucker/FOUNDRY`) looked like a saved value. The retrieval code was correct; the gap was zero signal about connection state.
- Added an always-visible **vault status pill** in the titlebar (amber `vault: FOUNDRY` vs muted `no vault`), refreshed when settings closes.
- Added a native **Browse… folder picker** (`tauri-plugin-dialog` + `dialog:default` capability); picking a folder connects it immediately. Tucker confirmed the vault then read correctly.

**Knowledge layer scaffolded.**
- `knowledge/CLAUDE.md` — orientation, the three rules, journal/decision/wiki formats. Adapted from the cosi-platform layer, Amber-specific. Documents how `knowledge/` / `docs/` / the FOUNDRY vault differ.
- `knowledge/wiki/` — seeded: [[architecture]], [[vault-retrieval]], [[knowledge-layer]], [[build-status]], [[glossary]], [[roadmap]].
- `knowledge/decisions/` — the four real decisions so far (A2, auth, key-storage, retrieval).
- `knowledge/journal/` — [[journal/2026-05-29-m0-m2-foundation]] (backfill) + this entry.
- Added a pointer to `knowledge/` from the repo-root `CLAUDE.md`.

- Paths touched: `src/App.tsx`, `src/App.css`, `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`, `src-tauri/capabilities/default.json`, `package.json`, new `knowledge/**`, `CLAUDE.md`.
- Subsystems: [[vault-retrieval]] (UX), [[knowledge-layer]] (new).

## Decisions made
- No new architectural decisions — the four pre-M3 decisions were captured retroactively into `decisions/` this session.
- **Adopt the cosi-platform knowledge-layer pattern verbatim** (three rules, ADR formats, confidence field) rather than invent an Amber-specific variant — consistency across Tucker's projects is the point.

## What was tried and abandoned
- Nothing dropped. (Confirmed the app's vault retrieval reads FOUNDRY, not the amber repo, so `knowledge/` here doesn't get pulled into Amber's own answers — no interference.)

## Open threads
- [ ] **M3 — command bar** is the next build milestone. See [[build-status]].
- [ ] Offered but not built: a guardrail test asserting nothing writes under the vault root (read-only by construction). In [[roadmap]].
- [ ] Going forward: journal each meaningful session here; run a compile pass at session end per Rule 1.

## Related
- Touched articles: [[knowledge-layer]], [[vault-retrieval]], [[architecture]], [[build-status]], [[glossary]], [[roadmap]]
- Pattern source: `/Users/tucker/Code/cosi/cosi-platform/knowledge/` (code-side layer); FOUNDRY's knowledge layer (the original)
