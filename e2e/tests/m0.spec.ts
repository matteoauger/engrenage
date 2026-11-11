import { expect, test } from '@playwright/test'

/**
 * M0 e2e suite — no auth yet (arrives with the auth foundation issue).
 * Covers the contract behaviors implemented so far.
 */

test('dashboard shows backend reachable', async ({ page }) => {
    await page.goto('/')
    await expect(page).toHaveTitle('Engrenage')
    await expect(page.getByText('reachable')).toBeVisible()
})

test('unknown route renders the in-shell 404', async ({ page }) => {
    await page.goto('/definitely-not-a-page')
    await expect(page.getByText('Page not found')).toBeVisible()
    // The shell must stay usable: sidebar link intact (routing contract).
    await expect(
        page.getByRole('link', { name: 'Back to dashboard' })
    ).toBeVisible()
    // exact: true — "Dashboard" would otherwise also match
    // "Back to dashboard" (role-name matching is a case-insensitive
    // substring match).
    await expect(
        page.getByRole('link', { name: 'Dashboard', exact: true })
    ).toBeVisible()
})

test('missing asset returns a clean 404', async ({ request }) => {
    // Extension'd paths are never SPA-fallbacked (assets.rs contract).
    const response = await request.get('/assets/e2e-missing.js')
    expect(response.status()).toBe(404)
    const body = await response.json()
    expect(body.error).toContain('no such asset')
})
