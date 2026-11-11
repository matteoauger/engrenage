/**
 * Follow the OS theme by default; a manual toggle arrives with user settings.
 *
 * Note: this is deliberately *not* a hook — it is a one-shot bootstrap side
 * effect for <html>, not render-scoped logic. A real useTheme() hook appears
 * in src/hooks/ once a manual toggle (and thus state) exists.
 */
export function initSystemTheme(): void {
    const media = window.matchMedia('(prefers-color-scheme: dark)')
    const apply = () =>
        document.documentElement.classList.toggle('dark', media.matches)
    apply()
    media.addEventListener('change', apply)
}
