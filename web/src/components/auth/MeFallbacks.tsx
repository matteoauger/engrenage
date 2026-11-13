/**
 * Full-screen fallback states shared by every guard that resolves the
 * session (`/me`) before rendering: spinner while pending, and a retry
 * prompt when the server is unreachable.
 *
 * Error-handling contract: only an explicit 401 means the session is gone
 * and may bounce an authenticated user to the login form; anything else
 * (network blip, backend restart, 5xx) shows the unreachable state below
 * instead. Both guards in RequireAuth and ChangePasswordPage follow it.
 */
export function FullScreenLoading() {
    return (
        <div className="flex h-svh items-center justify-center bg-bg">
            <p className="text-sm text-text-muted">loading…</p>
        </div>
    )
}

export function ServerUnreachable({ onRetry }: { onRetry: () => void }) {
    return (
        <div className="flex h-svh flex-col items-center justify-center gap-3 bg-bg">
            <p className="text-sm text-text-muted">couldn’t reach the server</p>
            <button
                onClick={onRetry}
                className="rounded-md px-3 py-1.5 text-sm text-text hover:bg-surface-hover"
            >
                retry
            </button>
        </div>
    )
}
