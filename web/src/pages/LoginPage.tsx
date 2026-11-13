import { useState, type FormEvent } from 'react'
import { Navigate, useNavigate, useSearchParams } from 'react-router'
import { useQueryClient } from '@tanstack/react-query'
import { apiPost, ApiError } from '../api/client'
import { useMe } from '../hooks/useMe'

/// `from` is user-controllable via ?from=; only in-app paths may pass.
function safeFrom(raw: string | null): string {
    return raw?.startsWith('/') && !raw.startsWith('//') ? raw : '/'
}

export function LoginPage() {
    const me = useMe()
    const navigate = useNavigate()
    const [searchParams] = useSearchParams()
    const from = safeFrom(searchParams.get('from'))
    const queryClient = useQueryClient()

    const [username, setUsername] = useState('')
    const [password, setPassword] = useState('')
    const [error, setError] = useState<string | null>(null)
    const [pending, setPending] = useState(false)

    // Already signed in? Send the user where they belong.
    if (me.isSuccess) {
        return (
            <Navigate
                to={me.data.mustChangePassword ? '/change-password' : from}
                replace
            />
        )
    }

    async function onSubmit(event: FormEvent) {
        event.preventDefault()
        setPending(true)
        setError(null)
        try {
            await apiPost('/api/auth/login', { username, password })
            await queryClient.invalidateQueries({ queryKey: ['me'] })
            navigate(from, { replace: true })
        } catch (err) {
            setError(err instanceof ApiError ? err.message : 'Login failed')
        } finally {
            setPending(false)
        }
    }

    return (
        <div className="flex h-svh items-center justify-center bg-bg p-6">
            <form
                onSubmit={onSubmit}
                className="w-full max-w-sm rounded-lg border bg-surface p-6 shadow-sm"
            >
                <h1 className="text-xl font-semibold tracking-tight">
                    Engrenage
                </h1>
                <p className="mt-1 text-sm text-text-muted">
                    Sign in to manage the server.
                </p>

                <div className="mt-6 flex flex-col gap-4">
                    <div className="flex flex-col gap-1">
                        <label
                            htmlFor="username"
                            className="text-sm font-medium"
                        >
                            Username
                        </label>
                        <input
                            id="username"
                            autoComplete="username"
                            required
                            value={username}
                            onChange={(e) => setUsername(e.target.value)}
                            className="rounded-md border bg-bg px-3 py-2 text-sm outline-none focus:border-accent"
                        />
                    </div>
                    <div className="flex flex-col gap-1">
                        <label
                            htmlFor="password"
                            className="text-sm font-medium"
                        >
                            Password
                        </label>
                        <input
                            id="password"
                            type="password"
                            autoComplete="current-password"
                            required
                            value={password}
                            onChange={(e) => setPassword(e.target.value)}
                            className="rounded-md border bg-bg px-3 py-2 text-sm outline-none focus:border-accent"
                        />
                    </div>

                    {error && (
                        <p className="rounded-md border border-danger/40 bg-danger/10 px-3 py-2 text-sm text-danger">
                            {error}
                        </p>
                    )}

                    <button
                        type="submit"
                        disabled={pending}
                        className="rounded-md bg-accent px-4 py-2 text-sm font-medium text-accent-contrast hover:bg-accent-hover disabled:opacity-50"
                    >
                        {pending ? 'Signing in…' : 'Sign in'}
                    </button>
                </div>
            </form>
        </div>
    )
}
