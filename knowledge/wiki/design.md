---
name: design
type: reference
created: 2026-06-02
last_updated: 2026-06-02
confidence: high
related: [[north-star]], [[architecture]], [[roadmap]]
---

# Design — the surface, and where it's headed

The north-star's "beautiful, intuitive surface" half lives here. Amber's interaction model is **Raycast-inspired**, and Tucker (creative director) will increasingly drive the visual design directly in Figma for the build to replicate.

## Figma — source of truth for visual design
- **Amber — Desktop Agent:** https://www.figma.com/design/uabKVJwD7YW6kpIy3TUJaI/Amber-%E2%80%94-Desktop-Agent
- Started 2026-06-02, currently blank. As it fills in, it becomes the canonical spec for the UI — replicate from it rather than inventing layouts. (Use the Figma MCP / `get_design_context` to pull frames when building.)

## The window model we're moving toward (Raycast-style, stated 2026-06-02)
**Today** (M3): two separate windows — a traditional decorated `main` window (with macOS traffic-light buttons) + a frameless always-on-top `palette` summoned by ⌥Space, ephemeral one-shot.

**Target:** collapse to ONE frameless surface that *morphs*, the Raycast pattern:
- **No traditional window** — no red/yellow/green jewel buttons anywhere.
- ⌥Space opens the **simple bar** (the compact palette).
- On send, that bar **expands into the larger chat surface** — which holds **conversation history (multiple conversations)**, the **journal**, and future features (a left nav / list, essentially).
- One window that grows from bar → full surface, not a handoff between two windows.

Implications to design through when we build it:
- Conversation **persistence** is new infrastructure (today every turn is ephemeral; "see different conversations" needs stored threads).
- The expanded surface needs nav/layout (conversations list, journal view, features) — Figma will spec this.
- Pairs with the **native macOS companion behaviors** (tray icon, dock-icon off, close-to-hide) — dock-off + tray + a frameless morphing window IS the Raycast model.
- Supersedes M3's two-window design (see [[build-status]]); record the change when we execute.

## Current aesthetic
Ember-glow dark theme (`src/App.css` CSS vars: `--amber`, `--amber-deep`, `--amber-glow`, warm dark `--bg`). Rounded floating palette card. Agent activity trail + source chips currently muted (toned down 2026-06-01); a full visual-hierarchy + thinking-fade/collapse pass is parked in [[roadmap]] (agent-UX polish).

## Context log

### 2026-06-02 — Created
Tucker started the Figma file and described the Raycast-style window unification (bar morphs into the full surface; no traffic-light window). Captured here as the design direction; Figma will hold the visual spec.
