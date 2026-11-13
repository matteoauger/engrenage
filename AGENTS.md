# AGENTS.md

Vintage Story server dashboard: a single self-contained Rust binary (axum REST + WebSocket) serving an embedded vanilla React SPA. Admin-only tool.

## Layout

- `crates/*` — Cargo workspace (edition 2024, `resolver = "3"`):
    - `engrenage` — the app: config, axum router, auth, SQLite state. Lib + thin bin so `run()` and `build_router` are integration-testable in-process.
    - `server`, `bridge` — scaffolded stubs for future server-domain logic (game lifecycle, versions, mods) and the stdin FIFO bridge.
- `web/` — React 19 + Vite + TypeScript + Tailwind SPA. npm workspace.
- `e2e/` — Playwright suite. npm workspace.
- `data/` — runtime state dir (SQLite db), not source.

## Dev environment

- Use `nix develop` — it pins the Rust toolchain from `rust-toolchain.toml` (1.100.0) and provides node, sqlite, chromium, bacon, git-cliff.
- Backend defaults to `127.0.0.1:8080`; config file is `engrenage.toml` (optional, `--config`/`--listen` override).
- Dev flow needs **two** processes: `npm run dev:backend` (bacon) and `npm run dev:frontend` (Vite proxies `/api` and `/health` to the backend). Debug builds serve no embedded SPA.

## Commands

Root npm scripts wrap everything:

- `npm run lint` — web ESLint + `cargo clippy --all-targets -- -D warnings`
- `npm run test` — `cargo test` (unit + integration)
- `npm run build` — **builds web first, then cargo** (order matters, see gotcha below)
- `npm run test:e2e` — Playwright
- `npm run prettier` / `prettier:check` — formats the whole repo (no semicolons, single quotes)
- `cargo fmt --check` — must pass; CI gates on it

Focused runs:

- One Rust crate: `cargo test -p engrenage` (or `server` / `bridge` / `downloader`)
- One Playwright test: `npm run test:e2e -- -g "<test name>"` (from repo root) or run inside `e2e/`

CI parity (`.github/workflows/ci.yml`): Rust side = fmt → clippy → `cargo test --workspace`; web side = `prettier:check` → eslint → `tsc -b && vite build`. Match all of these locally before pushing. E2E only runs on PRs to `main` and on `v*` tags (it's a merge/release gate, not a push gate).

## Gotchas

- **`rust-embed` embeds `web/dist` at compile time.** Building cargo without first running `npm run build -w web` produces a binary with no SPA assets (`dist/` is gitignored). Release packaging does exactly this ordering.
- E2E launches the full stack itself via Playwright's `webServer` (temp config file, `ENGRENAGE_BOOTSTRAP_PASSWORD=engrenage-e2e-password` for a known admin). It prefers a `chromium` on `PATH` locally — required on NixOS, where the Playwright-bundled browser can't launch. `E2E_CHROMIUM_PATH` overrides.
- Rust tests: unit tests inline (`#[cfg(test)]`), integration tests in `tests/` per crate against the public API. No black-box binary tests — the Playwright layer covers the real artifact.
- **SQL lives only in the `db` module** (`db::accounts`, `db::sessions`, `db::audit`, `db::migrate`) as typed repository functions over `rusqlite`. Feature modules (`auth`, routes) must not contain SQL — they call repositories.
- Auth details to preserve when touching auth code: argon2id passwords, session tokens stored **hashed** (SHA-256), opaque token in HttpOnly cookie `engrenage_session`, audit log rows for login/logout/password events.
- After bumping the toolchain channel in `rust-toolchain.toml`, run `nix flake update rust-overlay`.

## External references

- `docs/references.md` — official Vintage Story docs (dedicated server hosting, server commands). Consult before implementing server-domain features.

## Workflow

- Trunk-based: short-lived `feat/<name>` branches, squash-merge PRs into `main`.
- **Conventional commits** (`feat:`, `fix:`, `chore:` …) — the changelog and release notes are generated from them by git-cliff at tag time; there is no `CHANGELOG.md`. Preview: `git cliff --latest --strip all`.
- Semantic-ish versioning from milestone tags (`v0.x`); commits must be structured for changelog extraction.

## Code quality expectations

- Small reusable components, clear separation of responsibilities, robust to errors, clear `tracing` logging on the backend.
- Keep the hand-written TypeScript API client in `web/src/api/` consistent with the axum routes (no codegen).
