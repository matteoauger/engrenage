import { useNavigate } from 'react-router'
import { NavLink, Outlet } from 'react-router'
import { useQueryClient } from '@tanstack/react-query'
import { apiPost } from '../../api/client'
import { cn } from '../../lib/cn'
import { useMe } from '../../hooks/useMe'

/**
 * App shell: sidebar with placeholder navigation + content area.
 * Only Dashboard is real for now; the other entries stay disabled until
 * their milestones land (M1 console, M3 mods, M4 players/config, M5 backups).
 */
const NAV_ITEMS = [
    { label: 'Dashboard', to: '/', active: true },
    { label: 'Console', to: '/console', active: false },
    { label: 'Mods', to: '/mods', active: false },
    { label: 'Players', to: '/players', active: false },
    { label: 'Configuration', to: '/config', active: false },
    { label: 'Backups', to: '/backups', active: false },
] as const

export function AppLayout() {
    return (
        <div className="flex h-svh">
            <aside className="flex w-56 flex-col border-r bg-surface">
                <div className="flex items-center gap-2 border-b p-4">
                    <span className="text-lg font-semibold tracking-tight">
                        Engrenage
                    </span>
                </div>
                <nav className="flex flex-col gap-1 p-2">
                    {NAV_ITEMS.map((item) =>
                        item.active ? (
                            <NavLink
                                key={item.to}
                                to={item.to}
                                end
                                className={({
                                    isActive,
                                }: {
                                    isActive: boolean
                                }) =>
                                    cn(
                                        'rounded-md px-3 py-2 text-sm',
                                        isActive
                                            ? 'bg-accent/10 font-medium text-accent'
                                            : 'text-text-muted hover:bg-surface-hover hover:text-text'
                                    )
                                }
                            >
                                {item.label}
                            </NavLink>
                        ) : (
                            <span
                                key={item.label}
                                aria-disabled
                                title="Coming in a later milestone"
                                className="cursor-not-allowed rounded-md px-3 py-2 text-sm text-text-muted/60"
                            >
                                {item.label}
                            </span>
                        )
                    )}
                </nav>
                <div className="mt-auto border-t p-4">
                    <UserBox />
                    <p className="mt-2 text-xs text-text-muted">v0.1.0-dev</p>
                </div>
            </aside>
            <main className="flex-1 overflow-auto p-6">
                <Outlet />
            </main>
        </div>
    )
}

function UserBox() {
    const me = useMe()
    const navigate = useNavigate()
    const queryClient = useQueryClient()

    async function logout() {
        try {
            await apiPost('/api/auth/logout', {})
        } finally {
            await queryClient.invalidateQueries({ queryKey: ['me'] })
            navigate('/login', { replace: true })
        }
    }

    return (
        <div className="flex items-center justify-between gap-2">
            <span className="truncate text-sm" title={me.data?.username ?? ''}>
                {me.data?.username ?? '…'}
            </span>
            <button
                type="button"
                onClick={logout}
                className="rounded-md px-2 py-1 text-xs text-text-muted hover:bg-surface-hover hover:text-text"
                aria-label="Sign out"
            >
                Sign out
            </button>
        </div>
    )
}
