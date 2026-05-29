# Amber — Knowledge Layer

Amber's memory is a **folder of plain markdown** it points at (Tucker's is `/Users/tucker/FOUNDRY`). This doc covers how Amber should read/write that vault.

## The reconnect model (point-at-folder, don't import)
On first run Amber asks "where's your vault?" and points at the folder. No import, no conversion, no proprietary store. The vault stays usable in Obsidian, any editor, or a future cosi-platform desktop app — all pointing at the same files. Portability discipline is the whole game:
- Plain markdown + standard YAML frontmatter only — nothing Amber-specific in the files.
- `[[wikilinks]]` for cross-references (resolve by filename).
- Assets in an `assets/` subfolder, referenced by relative path.
- App state (API keys, config, window prefs) lives in the OS keychain / app-config dir — **never in the vault.**

## Retrieval v1 — index + grep, no vector DB
"The filesystem is the memory." For a moderate vault, an index file + grep/glob is enough and far simpler than embeddings.
- Read a top-level index/catalog of the vault first (cheap, gives the map).
- grep/glob for notes matching the query; read the most relevant in full.
- Inject as context into the model call.
- Add vector search ONLY if the vault outgrows this (revisit at a few hundred+ notes).

## Conventions to encode (from production practitioners — see FOUNDRY `knowledge/stm/karpathy-wiki-forks-scan.md`)
These make "agent over a vault" compound instead of decay:

1. **Every task produces two outputs** — the answer the user asked for AND updates to relevant notes. Hardcode this in how rituals work; don't leave knowledge in chat history.
2. **Type-aware ingestion** — classify a source (transcript / article / decision / reference / note) before extracting; apply a type-specific template (a transcript needs speakers + action items; an article needs methodology + findings).
3. **TL;DR block atop notes** — agents index-scan, read the TL;DR, then decide whether to read the full note. Token-efficient. Write one when creating/updating notes.
4. **Speculative `[[wikilinks]]`** — link to notes that *should* exist even if they don't yet; they're growth signals, not errors. Track them as gaps.
5. **Token-budget tiers** — load in order, only as needed: project context (small, always) → index → search results → full notes. Stops the agent reading too little or burning context reading everything.

## Read order for a fresh agent session over the vault
If the vault has an `Agent Notes` / handoff convention (FOUNDRY journals do), read the most recent handoff FIRST for current context, then the index, then drill into specific notes.

## Write-back
When a ritual or task produces something durable, write it back to the right note in the vault (the "two outputs" rule), using the vault's existing conventions (frontmatter, wikilinks, append-only context logs where the vault uses them). Match the surrounding style; don't impose Amber's own format on a user's vault.
