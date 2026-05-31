---
date: 2026-05-29
status: accepted
deciders: tucker
related: [[architecture]], [[2026-05-29-api-key-only-never-oauth]]
---

# Decision: Amber uses architecture A2 — all model calls via OpenRouter, routed by task

## Context
Three candidate architectures were on the table for how Amber reaches models:
- **B — wrap the `claude` CLI:** app shells out to the official Claude Code CLI, which authenticates with its own native subscription OAuth. $0 marginal cost (rides Max/Team seats) but Claude-only, terminal-bound, and **diverges from cosi-platform**.
- **A1 — direct Anthropic API:** app calls Anthropic directly with an API key. Pay-per-token, Claude-only, no model flexibility.
- **A2 — OpenRouter:** app calls all models through OpenRouter, routed by task type. Pay-per-token, but task routing keeps it cheap.

The strategic constraint: Amber is the **bridge product** before the full cosi-platform desktop app, and must stay architecturally coherent with it so it's a faithful preview, not a throwaway. cosi-platform already runs on OpenRouter + FAL.

## Decision
**Amber uses A2 — OpenRouter for every model call, with a task router that picks model tier (cheap vs frontier) by task type.** Locked. FAL for media generation comes later (same as cosi-platform).

## Consequences
- **Positive:** Model flexibility (any provider via one API); architectural parity with cosi-platform; task routing is the cost lever that keeps pay-per-token affordable; one integration surface.
- **Negative:** Pay-per-token (no free ride on a subscription); depends on OpenRouter availability + data governance (fine for personal v1; SOC 2 watch item once Amber touches client data).
- **Neutral:** Amber is the API client itself — which forces the auth decision below.

## Dissent / Alternatives Considered
- **B (wrap `claude` CLI)** — attractive for $0 cost riding the Max subscription. Lost on three counts: Claude-only, terminal-bound, and divergent from cosi-platform. It's also the *only* architecture where subscription OAuth is legal (because Claude Code itself authenticates) — and Amber is explicitly not doing that.
- **A1 (direct Anthropic)** — simplest integration, but Claude-only with no routing/cost lever and no platform parity. Lost to A2's flexibility.

## Sources
- [[journal/2026-05-29-m0-m2-foundation]] — the build sessions this governs
- `docs/architecture-and-auth.md` — full three-architecture rationale + cost model (FOUNDRY-mirrored)
- [[architecture]] — the wiki article carrying this forward
