---
name: glossary
type: meta
created: 2026-05-31
last_updated: 2026-05-31
confidence: high
related: [[architecture]], [[vault-retrieval]]
---

# Glossary — Amber vocabulary

- **Amber** — the Mac desktop AI second-brain. A Raycast-style command bar over a markdown knowledge vault. The bridge product before the full cosi-platform desktop app.
- **The vault** — the folder of plain markdown Amber points at and reads (Tucker's is `/Users/tucker/FOUNDRY`). The app's runtime memory. Read-only today.
- **Reconnect model** — point-at-folder, don't import. Amber asks "where's your vault?" and reads the files in place — zero import, zero conversion, stays usable in Obsidian/any editor.
- **A2 / OpenRouter architecture** — the locked architecture: all model calls go through OpenRouter, routed by task. Not B (wrap `claude` CLI), not A1 (direct Anthropic). See [[architecture]].
- **The auth hard line** — 🔒 API key only, never a consumer subscription OAuth token. Ban risk. See [[decisions/2026-05-29-api-key-only-never-oauth]].
- **Task routing** — M4: a router that picks model tier (cheap Haiku/Flash vs frontier Opus/GPT-5) by task type. The cost lever that makes OpenRouter cheaper than all-Opus.
- **Command bar** — M3: the global-hotkey, borderless, always-on-top window. The signature invocation surface.
- **Ritual** — M5: a packaged command (morning briefing, journaling, capture-to-vault) ported from FOUNDRY. Rituals follow the "every task produces two outputs" rule.
- **Grounding / Sources** — injecting vault notes into a model call so the answer comes from Tucker's brain, not just the base model. The `Sources` stream event surfaces which notes were used (the amber chips). See [[vault-retrieval]].
- **The two outputs rule** — every task produces the answer AND an update to the relevant note. The write-back half is M5; don't let knowledge evaporate into chat.
- **Ember-glow** — Amber's aesthetic: warm near-black background, amber/ember accents, soft glow. In `src/App.css`.
- **cosi-platform / Praxis** — CoSi's internal AI platform (codename Praxis). Amber is its bridge/preview product and stays architecturally coherent with it (OpenRouter + FAL).
- **The three doc surfaces** — `knowledge/` (build history), `docs/` (seeded plan), the FOUNDRY vault (app runtime memory). See [[knowledge-layer]].

## Context log

### 2026-05-31 — Created
Seeded with the vocabulary in play through M2.
