---
name: vault-retrieval
type: subsystem
created: 2026-05-31
last_updated: 2026-05-31
confidence: high
related: [[architecture]], [[knowledge-layer]]
---

# Vault retrieval — grounding answers in the markdown vault (M2)

Amber points at a folder of plain markdown (Tucker's is `/Users/tucker/FOUNDRY`) and grounds answers in it. "The filesystem is the memory." Retrieval v1 is **index + keyword-grep, no vector DB** (see [[decisions/2026-05-29-index-grep-retrieval-no-vector-db]]).

Lives in `src-tauri/src/vault.rs`. **Read-only — Amber never writes to the vault.** (Write-back is a deliberate M5 milestone, opt-in when it lands.)

## How it works (`vault::build_context`)

1. **Read the index.** First match of a candidate list wins: `knowledge/_index/master.md`, then `index.md` / `INDEX.md` / `_index.md` / `README.md` / `Home.md`. Gives the model the "map" cheaply (capped ~6KB).
2. **Keyword-score every note.** Walk all `.md` (skip hidden dirs, cap 4,000 files, read ≤32KB each for scoring). Score = keyword hits in content + a strong boost (+8) for filename/path matches. Then boost the distilled layer: `+5` for `wiki/` paths, `+2` for other `knowledge/` paths.
3. **Take the top 3** under a size budget (≤4KB of each note injected, char-boundary-safe truncation).
4. **Inject** a system message: the index + the notes, with an instruction to ground the answer and **cite the note filenames used**.
5. **Surface sources.** `chat()` emits a `Sources` stream event with the note paths; the UI renders them as amber chips under the answer. Directly answers "is it actually reading my vault?"

Vault config is **optional** — it augments chat, never gates it. No vault → pure model, no chips.

## Config & status

- `set_vault_path` validates the path is a real directory; native folder picker (`tauri-plugin-dialog`) or paste-a-path. `AMBER_VAULT` env overrides.
- A **vault status pill** in the titlebar shows connected (amber, `vault: FOUNDRY`) vs disconnected (muted, `no vault`) — added after the placeholder-vs-value confusion (see context log).

## Stopwords / keywords

Query is lowercased, split on non-alphanumerics, words ≥3 chars kept, a small stopword list dropped (including `amber`). Pure-lexical — the known v1 limitation; semantic recall is the vector-DB upgrade path.

## Context log

### 2026-05-31 — Article created
M2 shipped and verified: a Rust smoke test against the real FOUNDRY vault ranks `knowledge/wiki/praxis-platform.md` #1 for a praxis query, scanning 3,091 files in ~0.7s.

### 2026-05-31 — Vault-connection UX
First live test showed no grounding because the vault was never connected — the settings field's placeholder (`/Users/tucker/FOUNDRY`) looked like a saved value. Fixed with an always-visible titlebar status pill + a native Browse… folder picker. The retrieval code was correct; the gap was that nothing signaled connection state. See [[journal/2026-05-31-knowledge-layer-created]].
