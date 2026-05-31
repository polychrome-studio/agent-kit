---
date: 2026-05-29
status: accepted
deciders: tucker
related: [[architecture]], [[2026-05-29-openrouter-a2-architecture]]
---

# Decision: Amber authenticates to model providers with an API key — NEVER a consumer subscription OAuth token

## Context
Because Amber is architecture A2 (it **is** the API client — see [[2026-05-29-openrouter-a2-architecture]]), it has to authenticate its own model calls. The tempting shortcut — "ride Tucker's Claude Max subscription" — is a **ban-risk ToS violation**, not an option.

Anthropic's official policy (Claude Code legal doc): consumer (Free/Pro/Max) OAuth tokens are *exclusively* for "ordinary use of Claude Code and other native Anthropic applications." Using them in any other product/tool/service — including via the Agent SDK — violates the Consumer ToS. Anthropic **enforces this without notice**: blocking began 2026-01-09, clarified 2026-02-19. The only architecture where subscription auth is legal is B (wrapping the genuine unmodified `claude` CLI, so Claude Code itself authenticates) — which Amber is NOT.

## Decision
**Amber authenticates with an API key — an OpenRouter key, or a direct Anthropic API key. It must NEVER use a consumer-subscription OAuth token to make API calls.** This is a hard line, not a preference. No "ride the Max subscription" path exists for an app that makes its own calls.

## Consequences
- **Positive:** Zero ban risk; clean, defensible auth story; matches cosi-platform.
- **Negative:** Pay-per-token — no free subscription ride. (Mitigated by task routing — the cost lever.)
- **Neutral:** Keys live in the OS app-config dir / keychain, never in the vault, never committed. A gitleaks pre-commit hook enforces the never-committed half.

## Dissent / Alternatives Considered
- **Subscription OAuth (ride Max)** — attractive for $0 cost, rejected outright as a ToS violation Anthropic enforces by banning. Not a real alternative; documented so it's never re-litigated.

## Sources
- [[journal/2026-05-29-m0-m2-foundation]]
- `CLAUDE.md` (repo root) — the 🔒 HARD RULE section
- `docs/architecture-and-auth.md` — full reasoning
