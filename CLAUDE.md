# Amber

Amber is a **Mac desktop AI second-brain / personal assistant** — a Raycast-style command bar with a markdown knowledge vault as its background memory. It's Tucker's build (Head of Creative Technology at Collier Simon). This file is the canonical context for any agent working in this repo. Read it fully before doing anything.

> **If you're a fresh session: read this file, then read `KICKOFF.md` for exactly what to do first. The deeper rationale lives in `docs/`.**
>
> **Build history, decisions, and the current state of the build live in `knowledge/` — read `knowledge/CLAUDE.md` to see what's been built and why before doing architectural work. (`docs/` = the seeded plan; `knowledge/` = what we actually did.)**

---

## What Amber is (and isn't)

**Is:** a fast, beautiful desktop app where your second brain is one hotkey away. It reads/writes a folder of plain markdown (the knowledge vault), routes AI model calls by task through OpenRouter, and runs "rituals" (e.g. a morning briefing, journaling, capture). Companion feel — you work *with* Amber. Ember-glow aesthetic, Raycast-inspired interaction grammar.

**Isn't:** a chat app with memory bolted on; a cloud SaaS; a wrapper that stores your data on someone's server. The vault is local plain markdown you own. The agent is the runtime, the filesystem is the memory, the command bar is the surface.

**Strategic role:** Amber is the **bridge product** before the full **cosi-platform desktop app** (CoSi's internal AI platform, codename Praxis). It ships personal-scale first, proves the distribution + invocation UX, and stays architecturally coherent with cosi-platform so it's a faithful preview, not a throwaway. cosi-platform already runs on OpenRouter + FAL — Amber matches that.

---

## Architecture decision — READ THIS, it has a hard ToS boundary

There were three candidate architectures. **Amber uses A2 (OpenRouter).** This is locked.

| | What it is | Auth | Cost | Why not chosen |
|---|---|---|---|---|
| B — Claude Code terminal | App wraps the official `claude` CLI | CLI's own native OAuth (sub) | $0 (rides Max/Team seats) | Claude-only, terminal-bound, **diverges from cosi-platform** |
| A1 — Direct Anthropic API | App calls Anthropic directly | **API key** | pay/token, Claude-only | No model flexibility; doesn't match platform |
| **A2 — OpenRouter** ✅ | App calls all models via OpenRouter, routed by task | **API key** | pay/token, but task-routing keeps it cheap | — (chosen) |

### 🔒 HARD RULE — auth (do not violate, ban risk)
**Amber authenticates to model providers with an API key (OpenRouter key, or a direct Anthropic API key). It must NEVER use a consumer (Free/Pro/Max) subscription OAuth token to make API calls.**

Anthropic's policy (official Claude Code legal doc): consumer OAuth tokens are *exclusively* for "ordinary use of Claude Code and other native Anthropic applications." Using them in any other product/tool/service — including the Agent SDK — violates the Consumer ToS, and Anthropic **enforces this without notice** (blocking began Jan 9 2026, clarified Feb 19 2026). Because Amber (architecture A2) **is the API client**, it must use an API key. There is no "ride the Max subscription" shortcut for an app that makes its own calls. (The only way subscription auth is legal is architecture B — wrapping the genuine unmodified `claude` CLI so *Claude Code itself* authenticates — which Amber is NOT doing.)

Full reasoning: `docs/architecture-and-auth.md`.

---

## Stack

- **Tauri 2** — Rust core + system webview. Small (~10MB), fast, native, cross-platform later. (Tucker already knows Tauri from the Jellybean project — same toolchain.)
- **Frontend:** Vite + React + TypeScript. (Tucker prefers Vite+React over Next.js.)
- **Rust backend (Tauri commands):**
  - Vault file I/O — read/write markdown in the knowledge folder
  - OpenRouter calls via `reqwest` with streaming — the model layer + task router
  - FAL calls for media generation (later)
  - API keys stored in the OS keychain (`tauri-plugin-stronghold` or a keyring plugin) — **never in the vault, never committed**
- **Knowledge retrieval (v1):** index file + grep/glob over the vault. **No vector DB initially** — "the filesystem is the memory." Add vector search only if scale demands.
- **UX:** global hotkey → floating, borderless, always-on-top command bar (Raycast pattern; Tauri global-shortcut + a second window). Main window = markdown viewer pane + conversation pane.

---

## The knowledge vault

Amber points at a folder of plain markdown — for Tucker, that's his FOUNDRY vault (`/Users/tucker/FOUNDRY`). The "reconnect" model: Amber asks "where's your vault?" and points at the folder; zero import, zero conversion. Format discipline (plain markdown + standard YAML frontmatter + `[[wikilinks]]`) is what makes this portable.

Conventions worth encoding into how Amber reads/writes the vault (from production practitioners — see `docs/knowledge-layer.md`):
1. **Every task produces two outputs** — the answer AND updates to relevant notes. Don't let knowledge evaporate into chat.
2. **Type-aware ingestion** — classify a source (transcript / article / decision / reference / note) before extracting.
3. **TL;DR block atop notes** — index-scan → read TL;DR → decide whether to dig. Token-efficient.
4. **Speculative `[[wikilinks]]`** — link to notes that should exist; they're gap signals, not errors.
5. **Token-budget tiers** — load project context (small) → index → search results → full notes, in that order, only as needed.

---

## Repo & workspace conventions

- **Repo:** `inkxel/amber` (private). May transfer to `Collier-Simon/amber` later if it becomes the official CoSi bridge product (the same playbook Praxis→cosi-platform followed). Start personal for fast iteration.
- **Local path:** `~/Code/personal/amber` — deliberately OUTSIDE Dropbox and the FOUNDRY vault (dev tooling and cloud-sync hygiene).
- **Secrets:** never commit. API keys go in the OS keychain at runtime, or a gitignored `.env` for local dev. A **gitleaks pre-commit hook** is set up (`.githooks/pre-commit`) — enable it with `git config core.hooksPath .githooks`.
- **Commits:** small, milestone-tagged. Branch off `main` for features; `main` stays releasable.

---

## Build milestones (do in order — each de-risks the next)

- **M0 — Scaffold + repo** (~½ day): Tauri+Vite+React scaffold, empty window runs, repo on GitHub, gitleaks hook on. *Proves the toolchain.*
- **M1 — Talk to a model** (1–2 days): chat panel → OpenRouter (key from keychain) → streamed response, one hardcoded model. *Proves model wiring.*
- **M2 — Read the vault** (1–2 days): point at the vault folder, load index + a note, answer a question grounded in vault content. *Proves the knowledge layer.*
- **M3 — Command bar** (2–3 days): global hotkey → floating Raycast-style bar → type → streamed answer. *The signature UX.*
- **M4 — Task routing** (1–2 days): router picks cheap vs frontier model by task type. *The cost lever.*
- **M5 — Rituals** (ongoing): morning briefing, journaling, capture-to-vault, write-back. *The differentiation.*

Full plan: `docs/build-plan.md`.

---

## Tucker — working style (mirror of his global preferences)
- Short, direct, opinionated — sentences not paragraphs. No preamble, no trailing summaries. Lead with the answer/action.
- Systems thinker; prefers owning tools over renting SaaS; builder mentality (prototypes over presentations); automate what can be automated.
- Primary stack: n8n (being retired), Supabase, Claude API, Figma API, React. Background in type design + creative direction + technical systems.
- Act first, then ask at most 1–2 focused questions. Minimize round-trips.

## Pointers back to FOUNDRY (Tucker's vault — read if you have access)
- Vision + full three-architecture rationale + cost model: `/Users/tucker/FOUNDRY/knowledge/stm/foundry-bridge-desktop-app.md`
- Build plan (source of `docs/build-plan.md`): `/Users/tucker/FOUNDRY/knowledge/plans/amber-build-plan.md`
- Desktop distribution research (nohmitaina, Tauri patterns, reconnect model): `/Users/tucker/FOUNDRY/knowledge/stm/foundry-desktop-distribution.md`
- These are mirrored into `docs/` here so the repo is self-contained — but the FOUNDRY copies are the living source.
