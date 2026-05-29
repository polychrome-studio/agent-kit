# Amber — Build Plan

Mirror of `FOUNDRY/knowledge/plans/amber-build-plan.md` (FOUNDRY copy is the living source). Milestones are ordered so each de-risks the next — build them in sequence, don't skip ahead.

## M0 — Scaffold + repo (~½ day)
See `KICKOFF.md`. Tauri + Vite + React scaffold, empty window runs, repo on GitHub, gitleaks hook on.
**Proves:** the toolchain works end to end.

## M1 — Talk to a model (1–2 days)
- A conversation panel in the React UI.
- A Rust Tauri command `chat(messages) -> stream` that calls **OpenRouter** (`https://openrouter.ai/api/v1/chat/completions`) via `reqwest` with streaming.
- API key read from the OS keychain (or a gitignored `.env` for first light) — a settings screen to paste/store it.
- Hardcode ONE model first (e.g. `anthropic/claude-haiku` or `anthropic/claude-sonnet`). Stream tokens into the panel.
**Proves:** model wiring + streaming + key handling. **Auth = API key, never OAuth.**

## M2 — Read the vault (1–2 days)
- A Rust command to set + persist the vault path (Tucker's = `/Users/tucker/FOUNDRY`).
- Read the vault index/most-relevant note(s) and inject as context into the chat call.
- Ask a question grounded in vault content; confirm the answer cites/uses real notes.
- Retrieval v1 = read an index file + grep/glob for matching notes. **No vector DB yet.**
**Proves:** the knowledge layer — Amber answers from *your* brain, not just the base model.

## M3 — Command bar (2–3 days)
- Register a **global hotkey** (Tauri global-shortcut plugin).
- A second, borderless, always-on-top window = the Raycast-style command bar.
- Type a query/command → routes to the model → answer streams in the bar; Esc dismisses.
**Proves:** the signature invocation UX — one keystroke to your brain.

## M4 — Task routing (1–2 days)
- A router that classifies the task and picks a model tier: cheap (Haiku/Gemini-Flash/open) for simple work (classify, format, extract, short drafts); frontier (Opus/GPT-5) only when the task earns it.
- Surface which model answered (small label) for trust + cost awareness.
**Proves:** the cost lever — see `docs/cost-model.md`. This is what makes OpenRouter cheaper than all-Opus.

## M5 — Rituals (ongoing)
- Port FOUNDRY rituals as commands: morning briefing (`/today`-style), journaling, capture-to-vault.
- Apply the "every task produces two outputs" rule: the answer + a write-back to the right note.
**Proves:** the FOUNDRY-specific differentiation vs a generic second-brain app.

## Later
- FAL media generation (images/video) — cosi-platform already wires FAL.
- Vector search if/when the vault outgrows index+grep.
- Design polish pass (ember-glow aesthetic, the actual "much better looking" goal).
- Windows build (Tauri is cross-platform).
- Eventual `Collier-Simon/amber` transfer if Amber becomes the official CoSi bridge.

## Open questions
- Repo home: `inkxel/amber` (default, recommended) vs `Collier-Simon/amber` now.
- Confirm point-at-folder vs embed-a-vault → point-at-folder (the reconnect model).
- OpenRouter data governance once Amber touches client data (SOC 2) — personal v1 is fine; see FOUNDRY `knowledge/stm/cosi-platform-vps-options.md` SOC 2 watch list.
