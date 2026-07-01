# Amber — Knowledge Layer (AGENTS.md)

> **What Amber is.** A fast, beautiful Mac desktop AI second-brain — a Raycast-style command bar over a folder of plain-markdown knowledge, routing model calls by task through OpenRouter. The agent is the runtime, the filesystem is the memory, the command bar is the surface. The bridge product before the full cosi-platform desktop app.

This is the **canonical** agent-instructions file for this knowledge layer, per the cross-tool [AGENTS.md](https://agents.md) standard — any agent (Claude Code, Cursor, Codex, Aider, …) reads it. The sibling `knowledge/CLAUDE.md` is a thin pointer back here; keep the orientation in this file, not duplicated there. Claude Code discovers this layer when a session opens `amber/knowledge/` (the `CLAUDE.md` pointer triggers progressive loading, which leads here). Read it before doing architectural work. The wiki below documents how Amber is built, why it's shaped that way, and what's still open.

---

## What this knowledge layer is

A mini wiki + per-session journal + decision records + roadmap, living inside the Amber code repo at `knowledge/`. It exists because ideas and rationale surface in build sessions, get parked verbally, then get lost. This layer makes the build legible session-to-session — for the next agent and for Tucker six months from now.

### The doc surfaces (don't confuse them)

- **`knowledge/`** (this layer) — *build history.* Decisions made while building, per-session journal, curated wiki of how Amber actually works.
- **`docs/`** (repo root) — *the seeded design source.* `architecture-and-auth.md`, `build-plan.md`, `knowledge-layer.md`, `cost-model.md` — the living source of the original plan. Read these for *why we're building this at all*; `knowledge/` records *what we actually did*.
- **`CLAUDE.md`** (repo root) + **`KICKOFF.md`** — orientation + the M0 runbook for a fresh session.
- **The FOUNDRY vault** (`/Users/tucker/FOUNDRY`) — the *app's* knowledge memory, read at runtime by the app Amber is building. Unrelated to this build layer — Amber only reads it (write-back is a deliberate later milestone, M5, opt-in and explicit when it lands, never silent).

## How it's organized

```
knowledge/
  AGENTS.md          this file (canonical) — orientation, the three rules, the formats
  CLAUDE.md          thin pointer to AGENTS.md (so Claude Code discovers the layer)
  wiki/              curated reference, one article per subsystem/topic — append-only context logs
  journal/           per-session ADR-style entries — YYYY-MM-DD-slug.md, written continuously
  decisions/         atomic decision records — YYYY-MM-DD-slug.md, ADR format
  wiki/roadmap.md    parking lot — "at some point we should…"
```

`wiki/` is the curated layer. `journal/` and `decisions/` are the firehose. `roadmap.md` is the parking lot.

## Code-map — structural WHAT/HOW (consult before grepping)

`knowledge/wiki/_codemap.md` (+ machine-readable `knowledge/_codemap.json`) is an auto-generated structural index of this repo's source files — top-level symbols and import edges, extracted by tree-sitter (deterministic, no LLM). Covers rust/tsx/typescript. **Before grepping for "where does X live" or "what calls Y," read `_codemap.md` first.** The curated wiki and ADRs hold the *why*; the codemap holds the *what/how*. It is not wired to a commit hook today — regenerate manually after a source-touching session:

```
uv run --with tree-sitter --with tree-sitter-language-pack \
    python3 ~/.claude/skills/knowledge-layer/scripts/codemap.py [repo-root]
```

## README & research conventions

- **The README is WHAT / WHY / DIRECTION / NAVIGATION only.** It's the canonical one-page outline — what this is, why it exists, where it's going, how to navigate. Same structure as the `knowledge-layer` skill's `references/readme-template.md` (`~/.claude/skills/knowledge-layer/references/readme-template.md`), so this repo stays consistent with the pattern used across Tucker's other personal builds.
- **Research never goes in the README.** Findings, evaluations, benchmarks, option-analysis, cost estimates → `knowledge/research/<YYYY-MM>-<topic>.md` (create the folder the first time it's needed — Amber doesn't have one yet). The README states the *decision* that came out of it and links; it never reproduces the research. Test: *if it reads like findings, it's research; if it reads like direction, it's README.*

## Claim provenance tags (optional, use sparingly)

Beyond article-level `confidence:`, an individual claim inside a wiki article or ADR can carry a provenance tag for *how it was known*, not just how confident you are:
- **`EXTRACTED`** — sourced directly from the code (you read it, it's ground truth for the current state)
- **`INFERRED`** — assembled from reading or extrapolation (verify before acting)

Use these inline, only when the distinction matters for the reader.

---

## The three rules — no more

### Rule 1 — Journal *continuously*, wiki at the end

The journal is the firehose — write to it **as you go, not at session end.** Batching journal writes to the end loses information: you forget, you compress it away, or the session crashes and it's gone. The journal must survive a crash at any point.

**Checkpoint cadence — append to today's `journal/YYYY-MM-DD-slug.md` after any of these, while fresh:**
- a larger move lands (a feature works, a subsystem changes, a milestone closes)
- a longer code run completes (a big edit pass, a refactor, a tricky debug)
- **right after each `git commit`** — the commit is the natural "larger move" marker; journal the *why* the commit message doesn't capture (a breadcrumb hook automates the prompt)
- before any risky/irreversible operation (so intent is recorded even if it goes wrong)

A rough running entry beats a perfect one that never gets written. The entry is append-only — keep adding sections as the session progresses.

**Wiki, by contrast, is end-of-session only.** At session end an **explicit compile pass** promotes mature journal entries into wiki updates (per Rule 2). Without an explicit trigger the wiki stays untouched — that keeps it intentional and the prompt cache stable. *Journal hot and often, compile to wiki cold and deliberately.*

### Rule 2 — Wiki updates require a real trigger — only three
1. A new subsystem/capability got added → write a new article or new section.
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
type: Journal                      # OKF-required concept type (keep it)
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
- **Decision** — short statement. Rationale + link: [[decisions/YYYY-MM-DD-slug]]

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
type: Decision                     # OKF-required concept type (keep it)
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
Options weighed *before* the decision and why each lost. If there were none, say
"None — only viable path given X" so absence is explicit.

## Sources
- [[journal/YYYY-MM-DD-slug]] — session where this surfaced
```

## Wiki article frontmatter convention

```yaml
---
name: short-kebab-slug
type: subsystem | topic | meta | roadmap
created: YYYY-MM-DD
last_updated: YYYY-MM-DD
confidence: high | medium | low | speculative
related: [[other-article-1]], [[other-article-2]]
---
```

**`confidence:`** — **high** (in code / ratified in a decision / repeatedly confirmed) · **medium** (said once, one source, not contradicted) · **low** (inference / single source, provisional) · **speculative** (extrapolation, revisit before acting). Add it when an article is touched for a real reason, not in a bulk pass.

---

## The hard lines (carry these everywhere — see the decisions)

- **🔒 Auth: API key only, NEVER consumer subscription OAuth.** Amber (architecture A2) is the API client, so it authenticates with an OpenRouter (or direct Anthropic) API key. Using a Free/Pro/Max OAuth token in Amber violates Anthropic's Consumer ToS and is enforced without notice (ban risk). See [[decisions/2026-05-29-api-key-only-never-oauth]] and `docs/architecture-and-auth.md`.
- **A2 (OpenRouter) is the locked architecture.** Not B (wrapping the `claude` CLI), not A1 (direct Anthropic only). Task-routed model calls via OpenRouter — matches cosi-platform. See [[architecture]].
- **The vault is read-only (today).** Amber only reads the FOUNDRY vault. Write-back is a deliberate later milestone (M5), opt-in and explicit when it lands — never silent. See [[vault-retrieval]].
- **The filesystem is the memory.** Retrieval v1 = index file + grep/glob over markdown. No vector DB until the vault demonstrably outgrows it. See [[decisions/2026-05-29-index-grep-retrieval-no-vector-db]].

---

## Context log

### 2026-07-01 — Migrated to canonical AGENTS.md
Replaced the old `knowledge/CLAUDE.md` (which held all the orientation directly) with a canonical `knowledge/AGENTS.md` per Tucker's knowledge-layer convention (cross-tool `AGENTS.md` standard + thin `CLAUDE.md` pointer). No content was dropped — the hard lines, doc-surface distinctions, and code-map section carried forward from the old file.

### 2026-06-01 — North star aligned
Tucker set the explicit north star: Amber = his FOUNDRY + Obsidian + Claude Code CLI workflow "and then some," with a beautiful/intuitive surface over a genuinely agentic backend (Hermes/OpenClaw/Pi-class). The spine is a model-driven tool-use loop; web-search/write-back/rituals/proactivity are tools off it. New article [[north-star]] is the canonical statement — read it before architectural work. See [[journal/2026-06-01-session]].

### 2026-05-31 — Knowledge layer created
Scaffolded `amber/knowledge/` to mirror the cosi-platform code-side knowledge layer (`/Users/tucker/Code/cosi/cosi-platform/knowledge/`) so the Amber build has session-to-session documentation and decision history next to the source. Seeded with the M0–M2 build history (journal + decision records) and the first wiki articles. See [[journal/2026-05-31-knowledge-layer-created]].
