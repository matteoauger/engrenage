import { execSync } from 'node:child_process'
import { readFileSync } from 'node:fs'
import { defineConfig } from '@playwright/test'

const PORT = 8765
const baseURL = `http://127.0.0.1:${PORT}`
const dataDir = '/tmp/engrenage-e2e-data'

/**
 * Chromium selection:
 *   1. `E2E_CHROMIUM_PATH` — explicit override, wins everywhere.
 *   2. On NixOS only: the natively built chromium from PATH (the devShell
 *      provides it), because Playwright's bundled browser is dynamically
 *      linked against generic Linux libs and cannot launch under stub-ld.
 *   3. Otherwise (CI, macOS, plain Linux): the bundled browser.
 * The NixOS gate is deliberate: CI runners must never silently switch to a
 * system browser, whatever future runner images ship.
 */
const explicitPath = process.env.E2E_CHROMIUM_PATH || undefined

const isNixOS = (() => {
    try {
        return readFileSync('/etc/os-release', 'utf8').includes('ID=nixos')
    } catch {
        return false
    }
})()

const autoDetectedPath = isNixOS
    ? (() => {
          try {
              return execSync('command -v chromium').toString().trim()
          } catch {
              // No chromium in PATH: the bundled launch will fail with the
              // documented stub-ld error (see README).
              return undefined
          }
      })()
    : undefined

const executablePath = explicitPath ?? autoDetectedPath

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
        command: `cd .. && rm -rf ${dataDir} && printf 'data_dir = "%s"\\n' ${dataDir} > /tmp/engrenage-e2e.toml && npm run build -w web && cargo build --bin engrenage && ./target/debug/engrenage --config /tmp/engrenage-e2e.toml --listen 127.0.0.1:${PORT}`,
        url: `${baseURL}/health`,
        reuseExistingServer: !process.env.CI,
        // Generous ceiling for cold CI caches: the command chains the Vite
        // build, a cold cargo compile (first run has an empty rust-cache)
        // and the server boot on 2-core runners.
        timeout: 420_000,
    },
})
