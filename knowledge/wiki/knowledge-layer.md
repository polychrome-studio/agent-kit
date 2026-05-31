---
name: knowledge-layer
type: meta
created: 2026-05-31
last_updated: 2026-05-31
confidence: high
related: [[architecture]], [[build-status]]
---

# Knowledge layer — this build-documentation system, documented

This article documents `amber/knowledge/` itself — the code-side build-knowledge layer. Tucker keeps one inside every larger project; this mirrors the cosi-platform pattern (`/Users/tucker/Code/cosi/cosi-platform/knowledge/`), which in turn mirrors FOUNDRY's.

## Why it exists

Build sessions surface rationale ("we chose A2 because…", "the keychain looped because…") that gets parked verbally and lost. For a solo/contracted build, strong documentation is the survival strategy — it's how the next agent (or Tucker, months later) reconstructs *why*, not just *what*. The cost of losing it compounds as Amber's surface area grows (model layer, vault, command bar, rituals).

## The shape

- **`journal/`** + **`decisions/`** — append-only firehose. Journal = per-session ADR-flavored entries. Decisions = atomic ADRs (with a Dissent/Alternatives section so rejected paths aren't re-litigated).
- **`wiki/`** — curated layer, one article per subsystem/topic, append-only context logs with dated entries + a `confidence:` field.
- **`roadmap.md`** — parking lot.
- **`CLAUDE.md`** — orientation + the three rules + the formats. Read first.

Governed by **three rules**: (1) journal during, wiki at the end via an explicit compile pass; (2) wiki updates need a real trigger — new subsystem / contradicted decision / Tucker said so; (3) append-only, `[[wiki-links]]` everywhere.

## Don't confuse the three doc surfaces

| Surface | Holds | Role |
|---|---|---|
| `knowledge/` (this) | build history, decisions, how Amber works | what we actually did |
| `docs/` | architecture-and-auth, build-plan, knowledge-layer, cost-model | the seeded plan (FOUNDRY-mirrored) — why we're building this |
| FOUNDRY vault | Tucker's actual notes | the *app's* runtime memory — Amber reads it, unrelated to this layer |

Note the name collision: `docs/knowledge-layer.md` is about **how the app reads a vault**; this `wiki/knowledge-layer.md` is about **the build-documentation system**. Different things.

## Context log

### 2026-05-31 — Layer created
Scaffolded to mirror cosi-platform's code-side layer at Tucker's request, so Amber has session-to-session context. Seeded with M0–M2 history. See [[journal/2026-05-31-knowledge-layer-created]].
