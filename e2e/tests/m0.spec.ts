import { expect, test } from '@playwright/test'

const BOOTSTRAP_PASSWORD = 'engrenage-e2e-password'
const NEW_PASSWORD = 'changed-password-1'

/**
 * M0 e2e suite — auth foundation edition.
 *
 * The full login flow runs against the real crypto implementation; the
 * webServer boots with a known ENGRENAGE_BOOTSTRAP_PASSWORD. The flow
 * restores the bootstrap password at the end so it is repeatable against a
 * reused server — but the forced-change flag is consumed by the first run
 * (by design: any password change clears it), so the test handles both
 * states: fresh DB → full policy flow, reused DB → login/dashboards only.
 */

test('unauthenticated users land on the login page', async ({ page }) => {
    await page.goto('/')
    await expect(page.getByLabel('Username')).toBeVisible()
    await expect(page.getByLabel('Password')).toBeVisible()
})

test('unknown routes redirect unauthenticated users to login', async ({
    page,
}) => {
    await page.goto('/definitely-not-a-page')
    await expect(page.getByLabel('Username')).toBeVisible()
})

test('wrong password shows an inline error', async ({ page }) => {
    await page.goto('/login')
    await page.getByLabel('Username').fill('admin')
    await page.getByLabel('Password').fill('wrong-password')
    await page.getByRole('button', { name: 'Sign in' }).click()
    await expect(page.getByText('invalid credentials')).toBeVisible()
    // Still on the login page.
    await expect(page.getByLabel('Username')).toBeVisible()
})

test('missing asset returns a clean 404', async ({ request }) => {
    // Static assets are outside the /api auth scope.
    const response = await request.get('/assets/e2e-missing.js')
    expect(response.status()).toBe(404)
    const body = await response.json()
    expect(body.error).toContain('no such asset')
})

test('full login flow: sign in, forced password change, dashboard', async ({
    page,
}) => {
    // The webServer boots with ENGRENAGE_BOOTSTRAP_PASSWORD=engrenage-e2e-password.
    await page.goto('/')
    await page.getByLabel('Username').fill('admin')
    await page.getByLabel('Password').fill(BOOTSTRAP_PASSWORD)
    await page.getByRole('button', { name: 'Sign in' }).click()

    // Either the forced-change interstitial (fresh DB) or the dashboard
    // (reused DB: a previous run already consumed the flag).
    await expect(
        page.getByText(/Change your password|Server status and activity/)
    ).toBeVisible()
    const forcedChange = await page
        .getByText('Change your password')
        .isVisible()

    if (forcedChange) {
        // Interactive policy: a short password blocks submission.
        await page.getByLabel('Current password').fill(BOOTSTRAP_PASSWORD)
        await page.getByLabel('New password', { exact: true }).fill('short')
        await expect(page.getByText('At least 8 characters')).toBeVisible()
        await expect(
            page.getByRole('button', { name: 'Save new password' })
        ).toBeDisabled()

        // A passing password (matched confirmation) unlocks the form.
        await page
            .getByLabel('New password', { exact: true })
            .fill(NEW_PASSWORD)
        await page.getByLabel('Confirm new password').fill(NEW_PASSWORD)
        await expect(page.getByText('Good password')).toBeVisible()
        await expect(
            page.getByRole('button', { name: 'Save new password' })
        ).toBeEnabled()
        await page.getByRole('button', { name: 'Save new password' }).click()
        await expect(page.getByText('reachable')).toBeVisible()

        // Visit a non-dashboard page: this is the location RequireAuth will
        // remember across sign-out (?from=).
        await page.goto('/definitely-not-a-page')
        await expect(page.getByText('Page not found')).toBeVisible()

        // The new hash works: sign out, log back in with the changed
        // password — RequireAuth's redirect-back lands on the 404 page,
        // e2e-verifying the from contract.
        await page.getByRole('button', { name: 'Sign out' }).click()
        await expect(page.getByLabel('Username')).toBeVisible()
        await page.getByLabel('Username').fill('admin')
        await page.getByLabel('Password').fill(NEW_PASSWORD)
        await page.getByRole('button', { name: 'Sign in' }).click()
        await page.waitForURL('**/definitely-not-a-page')
        await expect(page.getByText('Page not found')).toBeVisible()

        // Back to the dashboard via the in-shell link.
        await page.getByRole('link', { name: 'Back to dashboard' }).click()
        await expect(page.getByText('reachable')).toBeVisible()

        // Restore the bootstrap password so the suite stays repeatable
        // against a reused server. page.request shares the browser context's
        // cookies.
        const restore = await page.request.post('/api/auth/change-password', {
            data: {
                currentPassword: NEW_PASSWORD,
                newPassword: BOOTSTRAP_PASSWORD,
            },
        })
        expect(restore.status()).toBe(200)

        // Repeatability proof: the original credentials work again.
        const relaunch = await page.request.post('/api/auth/login', {
            data: { username: 'admin', password: BOOTSTRAP_PASSWORD },
        })
        expect(relaunch.status()).toBe(200)
    } else {
        // Reused DB: the flag was consumed by a previous run. Sign out and
        // back in to prove the bootstrap credentials remain valid.
        await page.getByRole('button', { name: 'Sign out' }).click()
        await expect(page.getByLabel('Username')).toBeVisible()
        await page.getByLabel('Username').fill('admin')
        await page.getByLabel('Password').fill(BOOTSTRAP_PASSWORD)
        await page.getByRole('button', { name: 'Sign in' }).click()
        await expect(page.getByText('reachable')).toBeVisible()
    }
})

test('crafted ?from targets cannot escape the app', async ({ page }) => {
    // Runs after the full flow: the bootstrap password is always valid
    // here (the flag was consumed by a fresh DB's first run).
    await page.goto('/login?from=//evil.com')
    await page.getByLabel('Username').fill('admin')
    await page.getByLabel('Password').fill(BOOTSTRAP_PASSWORD)
    await page.getByRole('button', { name: 'Sign in' }).click()

    // The crafted protocol-relative target is normalized to '/'.
    await expect(page).toHaveURL(/\/$/)
    await expect(page.getByText('reachable')).toBeVisible()
})
