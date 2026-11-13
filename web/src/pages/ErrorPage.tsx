import { useRouteError, isRouteErrorResponse } from 'react-router'

/**
 * Full-page error boundary (root errorElement): rendered when rendering
 * itself fails or a router-level error escapes. Takes over the whole
 * viewport on purpose — the app shell cannot be trusted at that point.
 */
export function ErrorPage() {
    const error = useRouteError()

    let status = 500
    let message = 'Something went wrong'

    if (isRouteErrorResponse(error)) {
        status = error.status
        message = error.statusText || message
    } else if (error instanceof Error) {
        message = error.message
    }

    return (
        <div className="flex h-svh flex-col items-center justify-center gap-3 bg-bg p-6 text-center">
            <p className="text-6xl font-semibold text-danger/60">{status}</p>
            <h1 className="text-xl font-semibold">
                {status >= 500 ? 'Unexpected error' : 'Request error'}
            </h1>
            <p className="max-w-md text-sm break-all text-text-muted">
                {message}
            </p>
            <button
                type="button"
                onClick={() => window.location.reload()}
                className="mt-2 rounded-md bg-accent px-4 py-2 text-sm font-medium text-accent-contrast hover:bg-accent-hover"
            >
                Reload
            </button>
        </div>
    )
}
