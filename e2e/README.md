# e2e — Playwright suite

Full-stack tests: the `webServer` hook builds the SPA, builds the axum binary
and boots it on `127.0.0.1:8765` with a throwaway data dir (config file under
`/tmp`), waiting on `/health`.

## Run

```sh
npm run test:e2e          # from the repo root
```

CI always starts a fresh server; locally, an already-running one on the test
port is reused (`reuseExistingServer: !process.env.CI`).

## NixOS caveat

Playwright's bundled chromium is a dynamically linked generic-Linux binary and
**cannot launch on stock NixOS** (stub-ld refuses it). The config handles this
automatically, with an explicit priority:

1. `E2E_CHROMIUM_PATH` — any explicit override (any OS)
2. **NixOS only**: the natively built chromium from PATH — the flake devShell
   ships it, so `nix develop -c npm run test:e2e` just works
3. Everything else (CI, macOS, plain Linux): Playwright's bundled browser

Alternative system-level fix: enable `programs.nix-ld` (+ its libraries) in
your NixOS configuration and use the bundled browser everywhere. The devShell
approach is preferred here because it stays inside the repo.

CI (Ubuntu runners) is unaffected: no flake, no PATH chromium, bundled browser
with `--with-deps` system libraries.

## Scope

`tests/m0.spec.ts` covers the M0 contract: dashboard + `/health` reachability,
in-shell 404 (sidebar stays usable), clean 404 for missing extension'd assets.
Auth-aware specs arrive with the auth foundation.
