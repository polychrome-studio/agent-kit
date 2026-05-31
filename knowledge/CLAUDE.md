# Amber — Knowledge Layer

> **What Amber is.** A fast, beautiful Mac desktop AI second-brain — a Raycast-style command bar over a folder of plain-markdown knowledge. The agent is the runtime, the filesystem is the memory, the command bar is the surface. The bridge product before the full cosi-platform desktop app.

This file is read first by any Claude Code session that opens `amber/knowledge/` (progressive `CLAUDE.md` loading). Read it before doing architectural work. The wiki below documents how Amber is built, why it's shaped that way, and what's still open.

---

## What this knowledge layer is

A mini wiki + per-session journal + decision records + roadmap, living inside the Amber code repo at `knowledge/`. It mirrors FOUNDRY's knowledge-layer pattern (and cosi-platform's code-side layer) at product-development scale. It exists because ideas and rationale surface in build sessions, get parked verbally, then get lost. This layer makes the build legible session-to-session — for the next agent and for Tucker six months from now.

### How this relates to the other doc surfaces (don't confuse them)

- **`knowledge/`** (this layer) — *build history.* Decisions made while building, per-session journal, curated wiki of how Amber actually works. Append-only firehose (`journal/`, `decisions/`) → curated layer (`wiki/`).
- **`docs/`** (repo root) — *seeded design source.* `architecture-and-auth.md`, `build-plan.md`, `knowledge-layer.md`, `cost-model.md` — mirrored from FOUNDRY, the living source of the original plan. Read these for the *why we're building this at all*. This layer (`knowledge/`) records *what we actually did*.
- **`CLAUDE.md`** (repo root) + **`KICKOFF.md`** — orientation + the M0 runbook.
- **The FOUNDRY vault** (`/Users/tucker/FOUNDRY`) — the *app's* knowledge memory that Amber reads at runtime. Nothing to do with this build layer. Amber never writes there.

## How it's organized

```
knowledge/
  CLAUDE.md              this file — orientation, the three rules, the formats
  wiki/                  curated reference, one article per topic — append-only context logs
    architecture.md             the A2/OpenRouter decision + Tauri stack + how the layers compose
    vault-retrieval.md          M2: index + grep grounding, scoring, the Sources event
    knowledge-layer.md          this layer, documented
    build-status.md             M0→M5 milestone tracker — current state of the build
    glossary.md                 Amber vocabulary
    roadmap.md                  parking lot — "at some point we should…"
  journal/               per-session ADR-style entries — YYYY-MM-DD-slug.md, one per meaningful session
  decisions/             atomic decision records — YYYY-MM-DD-slug.md, ADR format
```

`wiki/` is the curated layer. `journal/` and `decisions/` are the firehose layer. `roadmap.md` is the parking lot.

## The three rules — no more

### Rule 1 — Journal during, wiki at the end
During a session: write liberally to `journal/` and `decisions/`. Both are append-only — nothing edits prior entries. At session end: an **explicit compile pass** promotes mature journal entries into wiki updates. Without an explicit trigger, the wiki stays untouched. Keeps the wiki intentional, keeps noise out of active builds, keeps the prompt cache stable.

### Rule 2 — Wiki updates require a real trigger — only three
1. A new capability/subsystem got added → write a new article or new section.
2. A previously-documented decision is now contradicted → update the article + log the contradiction.
3. Tucker explicitly said "document this in the wiki."

No "just in case" updates. No proactive rewrites of articles that aren't broken.

### Rule 3 — Append-only, wiki-links everywhere
Wiki articles are append-only context logs with dated entries. Journal and decision entries are append-only. All cross-references use `[[wiki-links]]`. Never edit prior entries; if something is wrong, append a correction with a link to the contradicting source.

---

## Journal entry format — ADR-flavored

Each `journal/YYYY-MM-DD-slug.md`:

```markdown
---
date: YYYY-MM-DD
session: short-slug
status: in-progress | shipped | abandoned | superseded-by [[YYYY-MM-DD-slug]]
related: [[architecture]], [[vault-retrieval]]
---

# Session Title — What we worked on

## Context
Where we were when we started. What problem this session was responding to.

## What changed
- Paths touched: `src-tauri/src/lib.rs`, `src/App.tsx`
- Subsystems affected: [[vault-retrieval]]
- Behavior shipped: brief description

## Decisions made
- **Decision 1** — short statement. Rationale + link: [[decisions/YYYY-MM-DD-slug]]

## What was tried and abandoned
- Tried X — dropped because Y. Saves the next teammate from re-litigating.

## Open threads
- [ ] Next-session item with [[wiki-link]]

## Related
- Touched articles: [[architecture]]
```

## Decision record format — standard ADR

Each `decisions/YYYY-MM-DD-slug.md`:

```markdown
---
date: YYYY-MM-DD
status: accepted | proposed | deprecated | superseded-by [[YYYY-MM-DD-slug]]
deciders: tucker
related: [[architecture]]
---

# Decision: One-sentence statement (verb-led, decisive)

## Context
The forces at play. What we were deciding against. Why now.

## Decision
What we're doing.

## Consequences
- **Positive:** what gets easier
- **Negative:** what gets harder, what we're trading away
- **Neutral:** what changes that's neither good nor bad

## Dissent / Alternatives Considered
Options weighed *before* the decision and why each lost. If there were none, say "None — only viable path given X" so absence is explicit.

## Sources
- [[journal/YYYY-MM-DD-slug]] — session where this surfaced
```

## Wiki article frontmatter convention

```yaml
---
name: short-kebab-slug
type: topic | subsystem | meta | roadmap
created: YYYY-MM-DD
last_updated: YYYY-MM-DD
confidence: high | medium | low | speculative
related: [[other-article-1]], [[other-article-2]]
---
```

**`confidence:`** — **high** (in code / ratified in a decision / repeatedly confirmed) · **medium** (Tucker said it, one source, not contradicted) · **low** (inference / single source, provisional) · **speculative** (extrapolation, revisit before acting). Add it when an article is touched for a real reason, not in a bulk pass.

---

## The hard lines (carry these everywhere — see the decisions)

- **🔒 Auth: API key only, NEVER consumer subscription OAuth.** Amber (architecture A2) is the API client, so it authenticates with an OpenRouter (or direct Anthropic) API key. Using a Free/Pro/Max OAuth token in Amber violates Anthropic's Consumer ToS and is enforced without notice (ban risk). See [[decisions/2026-05-29-api-key-only-never-oauth]] and `docs/architecture-and-auth.md`.
- **A2 (OpenRouter) is the locked architecture.** Not B (wrapping the `claude` CLI), not A1 (direct Anthropic only). Task-routed model calls via OpenRouter — matches cosi-platform. See [[architecture]].
- **The vault is read-only (today).** Amber only reads the FOUNDRY vault. Write-back is a deliberate later milestone (M5), opt-in and explicit when it lands — never silent. See [[vault-retrieval]].
- **The filesystem is the memory.** Retrieval v1 = index file + grep/glob over markdown. No vector DB until the vault demonstrably outgrows it. See [[decisions/2026-05-29-index-grep-retrieval-no-vector-db]].

---

## Context log

### 2026-05-31 — Knowledge layer created
Scaffolded `amber/knowledge/` to mirror the cosi-platform code-side knowledge layer (`/Users/tucker/Code/cosi/cosi-platform/knowledge/`) so the Amber build has session-to-session documentation and decision history next to the source. Seeded with the M0–M2 build history (journal + decision records) and the first wiki articles. See [[journal/2026-05-31-knowledge-layer-created]].
