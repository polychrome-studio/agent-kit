# Amber — Cost Model & Task Routing

The whole reason A2/OpenRouter is viable on cost: **route by task.** Most agentic work doesn't need a frontier model. Spend cheap on the 90%, reserve Opus/GPT-5 for the 10% that earns it.

## The numbers (modeled 2026-05-29 — APPROXIMATE list prices, verify before quoting anyone)

Assumptions: ~40 interactive turns/user/day · ~20K input (mostly cached) + 3K output per turn · 22 working days · prompt caching on.

| Approach | ~$/user/mo | ~$/mo @ 50 users |
|---|---|---|
| API — everything on Opus | ~$300 | ~$15,000 |
| API — everything on Sonnet | ~$59 | ~$2,950 |
| **API — task-routed (70% Haiku / 25% Sonnet / 5% Opus)** | **~$43** | **~$2,200** |
| Max 5× seat (flat) | $100 | $5,000 |
| Max 20× seat (flat) | $200 | $10,000 |
| Team seat (flat) | ~$30 | ~$1,500 |

**Key finding:** naive all-Opus API ≈ 3× a Max seat (the cost trap). But **disciplined task-routing lands *below* a Max seat** and near flat Team seats — while giving model flexibility + server capability + platform coherence. So routing discipline is worth more than the "ride the seats" subsidy.

## The task router (M4)

Classify each request into a tier, map tier → OpenRouter model ID:

- **Cheap tier** (Haiku / Gemini-Flash / open models): classification, extraction, formatting, routing decisions, short drafts, simple Q&A over retrieved context.
- **Mid tier** (Sonnet): structured summaries, multi-step reasoning, most drafting, code.
- **Frontier tier** (Opus / GPT-5): cross-domain synthesis, creative judgment, high-stakes output, anything where being wrong is expensive.

Default to the cheapest tier that can plausibly do the task; let the user (or a rule) escalate. Surface which model answered, for trust + cost awareness.

This mirrors FOUNDRY's own subagent model-routing philosophy (Haiku for mechanical, Sonnet for judgment-with-defined-output, Opus for strategic/creative).

## Cost controls to build in
- **Prompt caching** on repeated vault context (big input-cost saver).
- **Budget caps / usage meter** — API can spike where a flat seat can't; show spend, optionally cap. This is the one place flat seats beat API (predictable ceiling), so give Amber a ceiling too.
- Prefer small context: load index → search results → full notes only as needed (token-budget tiers).
