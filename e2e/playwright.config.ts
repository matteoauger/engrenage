import { execSync } from 'node:child_process'
import { defineConfig } from '@playwright/test'

const PORT = 8765
const baseURL = `http://127.0.0.1:${PORT}`
const dataDir = '/tmp/engrenage-e2e-data'

/**
 * Chromium selection: `E2E_CHROMIUM_PATH` wins everywhere. Locally (any OS),
 * a chromium on PATH is preferred over the bundled one — the flake devShell
 * provides a natively built one on NixOS, where the bundled binary cannot
 * launch (dynamically linked, stub-ld). CI always uses the bundled browser:
 * we install it ourselves and never inherit whatever the runner image ships.
 */
const explicitPath = process.env.E2E_CHROMIUM_PATH || undefined
const fromPath = process.env.CI
    ? undefined
    : (() => {
          try {
              return execSync('command -v chromium').toString().trim()
          } catch {
              return undefined // bundled browser it is
          }
      })()

const executablePath = explicitPath ?? fromPath

export default defineConfig({
    testDir: 'tests',
    timeout: 30_000,
    use: {
        baseURL,
        ...(executablePath ? { launchOptions: { executablePath } } : {}),
    },
    webServer: {
        // Playwright runs this from the config dir (e2e/): step up to the
        // workspace root first. The data dir goes through a throwaway config
        // file (no --data-dir flag; config file discovery is a real code path).
        // ENGRENAGE_BOOTSTRAP_PASSWORD gives the suite a known admin account
        // (default flow stays random).
        command: `cd .. && rm -rf ${dataDir} && printf 'data_dir = "%s"\\n' ${dataDir} > /tmp/engrenage-e2e.toml && npm run build -w web && cargo build --bin engrenage && ENGRENAGE_BOOTSTRAP_PASSWORD=engrenage-e2e-password ./target/debug/engrenage --config /tmp/engrenage-e2e.toml --listen 127.0.0.1:${PORT}`,
        url: `${baseURL}/health`,
        reuseExistingServer: !process.env.CI,
        // Generous ceiling for cold CI caches: the command chains the Vite
        // build, a cold cargo compile (first run has an empty rust-cache)
        // and the server boot on 2-core runners.
        timeout: 420_000,
    },
})
