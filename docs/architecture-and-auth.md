# Amber — Architecture & Auth (the ToS-critical doc)

This is the most important doc to get right. It explains *why* Amber is built the way it is, and the **hard auth boundary that must never be crossed** (ban risk). If you're an agent about to change how Amber talks to model providers, read this first.

---

## The three architectures we evaluated

The auth rule is not absolute — it depends entirely on **who makes the model request.**

### B — Claude Code terminal wrapper
The app spawns the genuine, unmodified `claude` CLI, pipes stdin/stdout, renders it nicely. **Claude Code itself** authenticates via its own native OAuth (a Pro/Max/Team subscription). The app never sees a token.
- ✅ This is "ordinary use of Claude Code," which Anthropic's policy explicitly permits → a subscription is **legitimately free** here.
- ❌ Claude-only, terminal-bound, and **diverges from cosi-platform** (which is OpenRouter-based). Rebuild risk when the bridge meets the platform.

### A1 — Direct Anthropic API
The app calls the Anthropic Messages API directly. The **app** is the client → **API key required.** Claude-only. No model flexibility.

### A2 — OpenRouter ✅ CHOSEN
The app calls **OpenRouter**, which fronts all models (Claude, GPT, Gemini, open models). The **app** is the client → **API key required.** But one API gives every model + per-task routing.
- ✅ **cosi-platform already runs on OpenRouter + FAL** — Amber stays coherent and acts as a true preview of the platform's model layer.
- ✅ Task-routing (cheap models for simple work) keeps blended cost low — see `cost-model.md`.
- ⚠️ OpenRouter is a *broker*; for client data heading toward SOC 2 there's a data-governance question (deferred — personal v1 doesn't touch client data).

---

## 🔒 THE HARD RULE

**Amber (architecture A2) is an API client. It authenticates with an API key — an OpenRouter key (`sk-or-...`) or a direct Anthropic API key (`sk-ant-api...`). It must NEVER use a consumer Free/Pro/Max subscription OAuth token (`sk-ant-oat...`) to make calls.**

### Why (the policy, verbatim intent)
From Anthropic's official Claude Code legal/compliance doc (`code.claude.com/docs/en/legal-and-compliance`):
- OAuth authentication is "intended **exclusively**… to support **ordinary use of Claude Code and other native Anthropic applications**."
- "**Developers building products or services**… should use **API key authentication**… Anthropic does not permit third-party developers to… **route requests through Free, Pro, or Max plan credentials on behalf of their users.**"
- "Anthropic reserves the right to… enforce these restrictions and **may do so without prior notice.**"

### Enforcement timeline
- **Jan 9, 2026** — Anthropic began blocking Max OAuth in third-party clients.
- **Feb 19, 2026** — clarified that consumer OAuth tokens used in "any other product, tool, or service — including the Agent SDK" violate the Consumer ToS.
- **June 15, 2026** — Agent SDK / `claude -p` on subscription plans draws from a separate metered monthly credit (Anthropic further walling off programmatic subscription use).

### The bright line, stated simply
- **Be an API client → use an API key.** (Amber.)
- The ONLY legal way to use a subscription is to be a *terminal wrapping the real `claude` CLI* (architecture B) so Claude Code authenticates itself. Amber is NOT doing that.
- **Never** extract, store, or replay an OAuth token. **Never** point Amber at `sk-ant-oat...`. **Never** centralize one account's auth to serve multiple users ("on behalf of users" = the explicit prohibition; for cosi-platform a violation could jeopardize the whole org's Claude access).

If a future requirement makes subscription economics tempting, the answer is *not* to bend this — it's either (a) accept API-key cost with task-routing (cheap, see cost-model), or (b) a *separate* terminal-wrapper product (architecture B), never Amber routing OAuth itself.

---

## Practical implementation notes
- Store the API key in the OS keychain (`tauri-plugin-stronghold` or a keyring plugin). For first-light dev, a **gitignored** `.env` is acceptable — the gitleaks pre-commit hook guards against committing it.
- OpenRouter endpoint: `POST https://openrouter.ai/api/v1/chat/completions` (OpenAI-compatible schema), `Authorization: Bearer $OPENROUTER_KEY`. Supports streaming (SSE).
- Use **prompt caching** where the provider supports it to control input cost on repeated vault context.
- Model IDs are namespaced on OpenRouter, e.g. `anthropic/claude-opus`, `anthropic/claude-haiku`, `google/gemini-flash`, etc. The task router (M4) maps task class → model ID.
