import type { ReactNode } from 'react'
import { Navigate, useLocation } from 'react-router'
import { useMe } from '../../hooks/useMe'
import { ApiError } from '../../api/client'
import { FullScreenLoading, ServerUnreachable } from './MeFallbacks'

/**
 * Auth guard: renders `children` only for authenticated users whose
 * password change is not pending; otherwise redirects to /login (remembering
 * the intended location) or to the forced change-password interstitial.
 */
export function RequireAuth({ children }: { children: ReactNode }) {
    const me = useMe()
    const location = useLocation()

    if (me.isPending) {
        return <FullScreenLoading />
    }

    if (me.isError) {
        // Only an explicit 401 means the session is gone; anything else
        // (network blip, backend restart, 5xx) must not bounce an
        // authenticated user to the login form.
        if (me.error instanceof ApiError && me.error.status === 401) {
            const from = encodeURIComponent(location.pathname)
            return <Navigate to={`/login?from=${from}`} replace />
        }
        return <ServerUnreachable onRetry={() => me.refetch()} />
    }

    if (me.data.mustChangePassword) {
        return <Navigate to="/change-password" replace />
    }

    return <>{children}</>
}
