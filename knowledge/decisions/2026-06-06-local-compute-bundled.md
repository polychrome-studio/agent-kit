---
date: 2026-06-06
status: proposed
deciders: tucker
related: [[architecture]], [[north-star]], [[roadmap]], [[2026-05-29-openrouter-a2-architecture]]
---

# Decision-in-waiting: Local compute, bundled into the app

> **Status: proposed / decision-in-waiting.** Not scheduled, not building yet. This records the architecture *before* the work so when local inference becomes active, the shape is already reasoned out. The recommendation (Option B) is a lean, not a lock — it gets ratified when the first local-compute milestone opens. Source thinking: FOUNDRY `knowledge/stm/local-compute-bundled-desktop.md` (2026-06-06), adapted to Amber's real state here.

## Where Amber actually is (so this stays honest)
Today **every** model call routes through OpenRouter — all four router modes (`Quick`/`Companion`/`Research`/`Deep`) map to cloud `anthropic/*` slugs in `src-tauri/src/router.rs`. There is **no** local inference, **no** embeddings, **no** vector store: retrieval is index + grep over the markdown vault (`vault.rs`), per [[2026-05-29-index-grep-retrieval-no-vector-db]]. The agent runtime (`agent.rs`) is a model-driven tool-use loop (`search_vault` / `read_note` / `web_search`) — also fully cloud. So "local compute" is genuinely net-new ground, not a swap of something already present. Good news: the router is already the right seam to add a tier to.

## The question
Can we build something like **oMLX** *into* Amber so local inference "just runs with the app" — no separate install or learning curve, especially once this is the bridge to a 50-person cosi-platform deployment? Tucker (6/6): *"Some level of local compute is most certainly in our future."* Repo that prompted it: https://github.com/jundot/omlx

## What oMLX is
An **MLX-based local inference server** for Apple Silicon (M-series, macOS 15+). Python + a Swift menubar app. Exposes **OpenAI-compatible endpoints at `localhost:8000/v1`** (chat, embeddings, rerank) with continuous batching + tiered KV cache. Ships as .dmg / Homebrew / pip, can run as a managed background service, and is explicitly **"embeddable" via a CLI shim** (`~/.omlx/bin/omlx`) — designed to be driven by another app. But it's still a Python+MLX payload to ship.

## The answer: bundle the ENGINE, not a second app the team installs
Two clean patterns, both fitting a Tauri/Rust app:

**Option A — Sidecar binary (ship an omlx-style server inside the .app).**
Tauri has a first-class **sidecar** mechanism: bundle an external binary, Tauri launches/manages it on the app lifecycle. Team installs ONE app → the local server spins up on launch → Amber calls `localhost`. oMLX's embed-shim fits this exactly. *Caveat:* oMLX is Python+MLX — bundling a Python runtime is heavy (hundreds of MB) and fragile to package (PyInstaller/py2app). This is the route only if MLX's specific speed/memory edge is worth that cost.

**Option B — Embed a native engine as a Rust crate (cleaner for this stack). ← recommended for v1.**
Skip Python entirely. Compile the inference engine *into* Amber: **candle** (HuggingFace, pure-Rust, runs on Metal), **mistral.rs**, or **llama.cpp via `llama-cpp-rs`**. Runs GGUF/safetensors on the Apple GPU, in-process, no separate server, no Python. This is the truest "just runs with the app," and it sits naturally next to Amber's existing Rust backend — the HTTP-call-in-Rust pattern already established for OpenRouter ([[architecture]]) becomes a local-call-in-Rust path. Add a thin local OpenAI-compatible shim (or a plain Rust trait) so the rest of the app never knows where inference ran.

**Model weights:** bundle one small model in the .app, or download-on-first-run into the app-support dir (`~/Library/Application Support/com.inkxel.amber/` — where the key + vault path already live). Invisible to the user.

## Why this matters for Amber specifically
- **The token-bill lever.** Amber's whole cost story today is the router (cheap-by-default, Opus-reserved). A local tier extends that lever to its limit: run the cheap, always-on, high-frequency work **locally at near-zero marginal cost** — embeddings (the moment Amber outgrows grep and needs vector recall — see [[roadmap]] "vector search"), summaries, the planned M5 rituals (morning briefing, journaling), classification — and reserve **cloud Claude** for heavy reasoning. The router already *names* these tiers; this gives the cheapest tier a free home.
- **Reinforces the "local + sovereign" north star.** [[north-star]] lists "local / sovereign, your data on your machine" as Amber's *strongest existing alignment* (plain-markdown vault, API-key-only, no SaaS). Bundled local inference closes the one gap: today the data is local but the *compute* still leaves the machine. This makes Amber local end-to-end — the OpenClaw lesson, delivered.
- **The agentic-agency privacy story** (for the cosi-platform future this bridges to): sensitive vault content summarized/embedded locally, never sent to a cloud provider. Ties directly to the auth hard line and the data-governance watch item in [[roadmap]].
- **Pattern already proven nearby.** FOUNDRY's Scribe runs on local Ollama; the precedent works. Amber's job is to make it **bundled + invisible** instead of a separate Ollama/oMLX install the team has to manage.

## Recommendation (when this opens)
1. **Add a local tier to the router abstraction** (`router.rs`) — the per-task "local vs cloud" decision lives where mode-routing already lives. App/agent code stays engine-agnostic; `Mode::model()` gains a local option for the cheap tiers. This is the one piece worth keeping in mind even now, so the seam doesn't have to be retrofitted.
2. **For local v1: embed candle / llama.cpp on Metal (Option B)** — single install, no Python, in-process, coherent with the Rust backend.
3. **Keep oMLX-as-sidecar (Option A) in the back pocket** for if/when MLX's last-20% speed/memory edge is worth the Python payload.
4. **Apple-Silicon-only is fine** for the CoSi Mac fleet; gate local features behind a capability check (mirrors the macOS-shaped behaviors already gated in [[roadmap]]). Windows falls back to cloud — no regression.

## Consequences if adopted
- **Positive:** near-zero marginal cost on the always-on layer; local-end-to-end privacy; differentiation vs cloud-subscription assistants; the embeddings story for vector search gets a free engine.
- **Negative:** app size grows (model weights + engine); Metal/Apple-Silicon-only for the local path; a real engineering chunk (engine integration, model management, the router seam) — deliberately deferred, not free.
- **Neutral:** doesn't touch the auth hard line (local needs no provider auth at all) or the OpenRouter A2 decision — local is an *additional tier under the router*, not a replacement for cloud.

## Sources
- FOUNDRY `knowledge/stm/local-compute-bundled-desktop.md` — the originating architecture writeup (Amber + cosi-platform)
- https://github.com/jundot/omlx — oMLX reference
- [[architecture]] — current cloud-only model layer this extends
- [[north-star]] — the "local / sovereign" alignment this completes
- [[roadmap]] — vector-search + data-governance items this connects to
