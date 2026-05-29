# Amber

A Mac desktop AI second-brain — a Raycast-style command bar with your markdown knowledge vault as its memory.

Amber puts your second brain one hotkey away: ask anything grounded in your own notes, run rituals (morning briefing, journaling, capture), and route AI work across models by task — all reading and writing a folder of plain markdown you own.

- **Stack:** Tauri 2 · Vite + React + TypeScript · Rust backend
- **Models:** OpenRouter (route by task — cheap models for simple work, frontier when it's earned)
- **Memory:** a local markdown vault you point at (no cloud, no lock-in)

## Status
Pre-M0 — scaffold pending. See `KICKOFF.md` to start, `CLAUDE.md` for full context, and `docs/` for the build plan, architecture/auth rules, cost model, and knowledge-layer design.

## ⚠️ Auth
Amber authenticates to model providers with an **API key** (OpenRouter or direct Anthropic). It must **never** use a consumer subscription OAuth token — that's an Anthropic ToS violation, enforced without notice. See `docs/architecture-and-auth.md`.

---
*Built by Tucker (Polychrome / Collier Simon). The bridge before the cosi-platform desktop app.*
