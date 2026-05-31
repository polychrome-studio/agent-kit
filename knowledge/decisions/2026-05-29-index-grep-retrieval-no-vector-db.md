---
date: 2026-05-29
status: accepted
deciders: tucker
related: [[vault-retrieval]], [[knowledge-layer]]
---

# Decision: Vault retrieval v1 is index + keyword-grep over markdown — no vector DB

## Context
M2 needed Amber to answer from Tucker's FOUNDRY vault, not just the base model. The vault is ~3,091 markdown files, but it has a curated layer: a master index (`knowledge/_index/master.md`, `[[wikilink]] — summary` lines) and ~192 distilled topic articles in `knowledge/wiki/`. "The filesystem is the memory" — embeddings add a build step, an index to maintain, and a dependency, for a corpus this size and this well-curated.

## Decision
**Retrieval v1 = read an index file + keyword-score every markdown note, return the top few under a size budget.** No vector DB. Scoring boosts filename/path matches and the distilled `knowledge/wiki/` layer over raw transcripts/journal. Add vector search **only** if the vault demonstrably outgrows this (revisit at a few hundred+ uncurated notes, or when grep recall visibly fails).

## Consequences
- **Positive:** Zero infra, zero index to keep fresh, fully transparent (you can see why a note ranked); fast — scans all 3,091 files in ~0.7s. Surfaced the correct `praxis-platform.md` wiki article #1 for a praxis query in the smoke test.
- **Negative:** Pure lexical — misses pure-synonym/semantic matches a query shares no keywords with. Capped at top-3 notes + a size budget per note, so deep multi-note synthesis is shallow in v1.
- **Neutral:** Retrieval is generic over any vault (index-file candidates + markdown walk), not hardcoded to FOUNDRY's layout — though it boosts a `wiki/` convention when present.

## Dissent / Alternatives Considered
- **Vector DB (embeddings + similarity)** — better semantic recall, but adds an embed pipeline, a store to maintain, and a dependency, for a corpus that's small and already human-curated. Deferred, not rejected — it's the documented upgrade path.
- **Send the whole vault as context** — impossible at 3,091 files; token-blind and expensive.

## Sources
- [[journal/2026-05-29-m0-m2-foundation]]
- [[vault-retrieval]] — the subsystem article
- `docs/knowledge-layer.md` — "Retrieval v1 — index + grep, no vector DB"
