# Amber — First Session Kickoff (M0)

This is the runbook for the **first dev session** in this folder. Read `CLAUDE.md` first (full context), then execute M0 below. After M0, follow `docs/build-plan.md` for M1→M5.

> This folder was pre-seeded (by a FOUNDRY session) with docs + config ONLY — no code yet. M0 scaffolds the actual Tauri app around these files.

---

## 0. Prerequisites (Tucker has these from the Jellybean project — verify)

```bash
rustc --version        # Rust toolchain (rustup)
node --version         # Node 18+ (for Vite)
xcode-select -p        # Xcode Command Line Tools (Mac builds)
gh auth status         # GitHub CLI, logged in as inkxel
which gitleaks         # optional but recommended (brew install gitleaks)
```

If any are missing: `rustup` from https://rustup.rs, Node via `brew install node`, `xcode-select --install`, `brew install gh gitleaks`.

---

## 1. Scaffold the Tauri app (handle the non-empty directory)

This folder already contains `CLAUDE.md`, `KICKOFF.md`, `docs/`, `.githooks/`, `.gitignore`. The Tauri scaffolder wants an empty dir, so scaffold into a temp folder and merge — this preserves the docs:

```bash
cd ~/Code/personal
npm create tauri-app@latest amber-scaffold -- --template react-ts --manager npm
# move generated files into the real folder WITHOUT clobbering our docs:
rsync -a --ignore-existing amber-scaffold/ amber/
# bring over files the scaffold owns that we don't have (package.json, src/, src-tauri/, index.html, vite config, tsconfig):
cp -n amber-scaffold/package.json amber/ 2>/dev/null || true
rsync -a amber-scaffold/src/ amber/src/
rsync -a amber-scaffold/src-tauri/ amber/src-tauri/
cp -n amber-scaffold/index.html amber/ 2>/dev/null || true
cp -n amber-scaffold/vite.config.* amber/ 2>/dev/null || true
cp -n amber-scaffold/tsconfig*.json amber/ 2>/dev/null || true
rm -rf amber-scaffold
cd amber
```

> NOTE: do NOT let the scaffold overwrite this folder's `CLAUDE.md`, `.gitignore`, or `docs/`. The `--ignore-existing` / `cp -n` flags above protect them. If the scaffold created its own `README.md`, keep it or replace freely — Amber's real context is in `CLAUDE.md` + `docs/`.

Set the app identifiers in `src-tauri/tauri.conf.json`:
- `productName`: `Amber`
- `identifier`: `co.collier-simon.amber` (or `com.inkxel.amber` for the personal build)
- window title: `Amber`

Verify it runs:

```bash
npm install
npm run tauri dev      # an empty Amber window should open — that's the M0 milestone
```

---

## 2. Wire up Git + GitHub + the gitleaks hook

```bash
cd ~/Code/personal/amber
git init
git config core.hooksPath .githooks      # enables the pre-commit gitleaks scan
chmod +x .githooks/pre-commit
git add -A
git commit -m "M0: Tauri + Vite + React scaffold + docs"
gh repo create inkxel/amber --private --source . --remote origin --push
```

(If `gh` isn't authed: `gh auth login` first. If you want it in the CoSi org instead, swap `inkxel/amber` → `Collier-Simon/amber`.)

---

## 3. M0 done when:
- [ ] `npm run tauri dev` opens an empty Amber window
- [ ] Repo exists at `inkxel/amber`, pushed, with the docs + scaffold committed
- [ ] gitleaks pre-commit hook active (`git config core.hooksPath` → `.githooks`)

Then proceed to **M1 (talk to a model via OpenRouter)** in `docs/build-plan.md`.

---

## Quick reminders for the build
- **Auth: API key only, never subscription OAuth** (see `CLAUDE.md` / `docs/architecture-and-auth.md` — this is a ban-risk hard line).
- Keys go in the OS keychain or a gitignored `.env` — never committed (the hook will catch leaks).
- Knowledge vault = a folder of markdown Amber points at (Tucker's is `/Users/tucker/FOUNDRY`). Point-at-folder, don't copy.
- Models via OpenRouter; route cheap models for simple tasks (the cost lever — `docs/cost-model.md`).
