import { Link } from 'react-router'

/**
 * In-shell 404: rendered for the catch-all route, so the sidebar and
 * navigation remain usable ("this page doesn't exist", not "app exploded").
 */
export function NotFound() {
    return (
        <div className="flex flex-col items-center justify-center gap-3 py-24 text-center">
            <p className="text-6xl font-semibold text-text-muted/40">404</p>
            <h1 className="text-xl font-semibold">Page not found</h1>
            <p className="max-w-md text-sm text-text-muted">
                The page you are looking for does not exist — it may belong to a
                feature that has not been built yet.
            </p>
            <Link
                to="/"
                className="mt-2 rounded-md bg-accent px-4 py-2 text-sm font-medium text-accent-contrast hover:bg-accent-hover"
            >
                Back to dashboard
            </Link>
        </div>
    )
}
