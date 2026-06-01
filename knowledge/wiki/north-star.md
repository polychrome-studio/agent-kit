---
name: north-star
type: meta
created: 2026-06-01
last_updated: 2026-06-01
confidence: high
related: [[architecture]], [[roadmap]], [[build-status]]
---

# North star — what Amber is aiming at

> Stated by Tucker, 2026-06-01, as a forward-alignment checkpoint. This steers every architectural decision from here. Read it before proposing what to build next.

## The one line
Amber should do what Tucker does today with **FOUNDRY (his markdown vault) + Obsidian + the Claude Code CLI** — *and then some* — but wrapped in a **beautiful, intuitive surface** over a **genuinely agentic backend** with the power of **Hermes (Nous Research), OpenClaw, and Pi**.

## The wedge — the surface is the differentiator
Hermes, OpenClaw, and Pi are *powerful* but built for terminal / chat-app / developer users; none are beautiful, none are designed for a creative director. **Amber = that agentic power behind a Raycast-grade, designed Mac app.** Power on the back, polish on the front. Nobody in that reference set is doing the polished-consumer-surface part — that's the bet.

## The three references (fetched 2026-06-01) — the bar for the backend
- **Pi (pi.dev)** — minimal terminal coding harness; *"primitives, not features"*; skills + TypeScript extensions; 15+ providers, mid-session model switching; tree-structured sessions. Lesson for Amber: **extensible primitives the user/agent builds on**, not a fixed feature set.
- **Hermes Agent (Nous, MIT)** — *"the agent that grows with you."* Lives on your server, **persistent memory + auto-generated skills**, gets more capable the longer it runs. Tools: web search, browser automation, vision, image gen, TTS; multi-model reasoning; real sandboxing; **natural-language cron**; isolated subagents. Lesson: **persistent learning + autonomy + proactivity**.
- **OpenClaw (openclaw.ai)** — open-source **local** personal assistant; privacy/sovereignty; persistent memory, browser control, full system access, **self-authoring skills**, 50+ integrations, background tasks, multi-agent orchestration, proactive **"heartbeat"** check-ins. Lesson: **local sovereignty + proactive agency + self-extension**.

## Shared DNA = the bar, mapped to where Amber is
| Their capability | Amber today | Closes via |
|---|---|---|
| Persistent memory that **learns / grows with you** | vault is memory but **read-only**, no learning loop | vault write-back ("two outputs"), M5 |
| **Model-driven tool-use + multi-step autonomy** | **single-turn chat + RAG**, no agent loop | the tool-use loop (the spine — see below) |
| **Self-authoring / extensible skills** (primitives) | rituals planned, not built | M5 rituals → skills the agent can invoke/author |
| **Proactive / background** (cron, heartbeats, briefings) | none | scheduled rituals; a "heartbeat" later |
| **Local / sovereign, your data on your machine** | ✅ **already** — plain-markdown vault, API-key-only, no SaaS | — (Amber's strongest existing alignment) |

## The leverage point — build the agent loop once
Amber today = `query → classify mode → vault retrieval (+web in research) → stream answer`. That's chat + RAG, not an agent.

The spine that unlocks the whole north star is a **model-driven tool-use loop**: the model plans → calls a tool → observes the result → iterates until done. Build that *once* and every capability becomes a **tool the agent orchestrates**:
- **web search** — the current `:online` forced-search becomes model-decided (**brick #1**; see [[journal/2026-06-01-session]] + [[roadmap]])
- **vault read/write** — the vault becomes read-write *working memory*, not just a read source ("every task produces two outputs")
- **rituals** (M5) — skills the agent or user invokes; eventually self-authored (Pi/OpenClaw lesson)
- **proactivity** — scheduled/heartbeat briefings (Hermes/OpenClaw)

This is exactly the CLAUDE.md thesis — *"the agent is the runtime, the filesystem is the memory, the command bar is the surface."* Amber has built the **memory** (vault retrieval) and the **surface** (command bar); the **agent runtime** is the unbuilt half. The north star is: build it.

## Deliberate tensions to decide (not yet settled)
- **Tool reach / safety.** OpenClaw grants full system + shell access. Amber is vault-scoped today. How far do Amber's tools reach — shell? arbitrary filesystem? or vault + web + a curated set of integrations? A power-vs-safety choice to make deliberately, not by drift.
- **Learning loop.** "Grows with you" implies a memory that *updates*, not just retrieves — which means write-back + some consolidation pass. Ties to the vault-write-back design (must stay opt-in, never silent — see [[architecture]]).
- **Auth stays the hard line.** Whatever the backend grows into, the API-key-only rule holds (no consumer OAuth) — see [[architecture]].

## Context log

### 2026-06-01 — Created
Tucker aligned the team (me) on the north star and gave three backend references (Pi, Hermes, OpenClaw). Captured here so post-M4 roadmap decisions bend toward the agentic-runtime arc, not more chat features. See [[journal/2026-06-01-session]].
