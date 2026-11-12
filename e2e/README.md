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

## Chromium selection

The bundled Playwright chromium is dynamically linked against generic Linux
libraries and **cannot launch on stock NixOS** (stub-ld refuses it). The
config resolves this without any OS sniffing:

1. `E2E_CHROMIUM_PATH` — explicit override, wins everywhere
2. **Locally** (any OS): a `chromium` on PATH is preferred — the flake
   devShell ships a natively built one on NixOS, so
   `nix develop -c npm run test:e2e` just works
3. **CI** (`CI` env set): always the bundled browser — installed by the
   workflow with `--with-deps`, never inheriting whatever the runner image
   ships

Alternative system-level fix on NixOS: enable `programs.nix-ld` (+ its
libraries) and use the bundled browser everywhere. The devShell approach is
preferred here because it stays inside the repo.

## Scope

`tests/m0.spec.ts` covers the M0 contract: dashboard + `/health` reachability,
in-shell 404 (sidebar stays usable), clean 404 for missing extension'd assets.
Auth-aware specs arrive with the auth foundation.
