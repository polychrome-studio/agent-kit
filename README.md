# agent-kit

> **Status: early RFC (v0.0.x).** Directional and unstable. Shared early to invite discussion; the spec hardens from real-world use, not from a committee.

An agent is a directory. `agent-kit` is the open, runtime-agnostic shape for that directory — what goes where, what's required, what's optional — so a harness (a local console, a deployed service, whatever reads it) can pick up any conformant agent and run it.

## The shape

```
my-agent/
  agent/
    instructions.md   required — the always-on system prompt
    tools/             optional — typed functions the agent can call
    skills/            optional — procedures loaded on demand (skills-kit format)
    subagents/         optional — delegated workers
    channels/          optional — where the agent is reachable (Slack, HTTP, ...)
    schedules/         optional — recurring jobs
    agent.yaml         optional — model/runtime config
```

`instructions.md` is the only required file. Everything else is additive — an agent that's just `instructions.md` is a valid, minimal agent-kit agent.

## Why this shape, and why it isn't invented here

This structure is deliberately not novel. It's the same shape [Vercel's `eve`](https://github.com/vercel/eve) landed on — "an agent is a directory," `instructions.md` + `tools/`/`skills/`/`channels/`/`schedules/` — and eve's own author pairs it explicitly with Google's Open Knowledge Format: *"eve is the standard for how we structure our agent, and OKF is the standard for how we structure the knowledge bases we attach the agents to."* Since [dotKnowledge](https://github.com/polychrome-studio/dotKnowledge) is OKF-conformant, that pairing is the intended one — agent-kit exists to make it explicit and to keep it usable outside eve's own runtime.

**The actual difference from eve:** eve is a full framework — an npm package that *compiles and deploys* a conformant directory to a hosted, checkpointed production service. agent-kit defines only the *shape*, with no runtime opinion at all. A lightweight, local, markdown-only harness (a Claude Code console, for instance) can conform to agent-kit without needing eve, a compile step, or Node — the same way a `.knowledge` bundle doesn't need a specific app to be a valid bundle. eve-conformant agents should also read as valid agent-kit agents; the reverse isn't required, since agent-kit's baseline is deliberately lighter.

## How it mounts a knowledge bundle

An agent-kit agent doesn't contain knowledge — it mounts a [dotKnowledge](https://github.com/polychrome-studio/dotKnowledge) bundle the same way [foundry](https://github.com/polychrome-studio/foundry) does: the bundle stays a separate, portable, sealed package; the agent reads it, never owns it. Swap the mounted bundle and the same agent shell runs for a different subject.

## Skills, addressed the same way twice

`agent/skills/` here and `<subject>.knowledge/skills/` in a [knowledge-kit](https://github.com/polychrome-studio/knowledge-kit) bundle are the same file format — see [skills-kit](https://github.com/polychrome-studio/skills-kit). A skill in an agent is role-general (true regardless of which bundle is mounted); a skill in a bundle is subject-specific (true about that one subject, swapped when the bundle swaps). Same shape, different mount point.

## Status & roadmap

v0 RFC. [foundry](https://github.com/polychrome-studio/foundry) already runs a conductor shaped like this, so it's being dogfooded there before anything here is declared stable. No `SPEC.md` yet; this README is the whole spec until real use says it needs to be split out.

This repo's earlier life was a Mac app (Amber, a desktop agent shell) — that project's dead, and its build scaffold has been removed. foundry is the reference implementation.

## License

[CC-BY-4.0](./LICENSE) for the specification text — matching dotKnowledge's license, since this is a format document, not code. A reference implementation, if one gets built here, would be separately licensed (Apache-2.0 or MIT), the same way Throughline is separate from dotKnowledge's own license.

## Contributing

Early and open. File an issue to discuss the shape, the eve relationship, or the skills-addressing convention.
