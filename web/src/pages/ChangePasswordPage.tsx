import { useState, type FormEvent } from 'react'
import { Navigate, useNavigate } from 'react-router'
import { useQueryClient } from '@tanstack/react-query'
import { apiPost, ApiError } from '../api/client'
import {
    FullScreenLoading,
    ServerUnreachable,
} from '../components/auth/MeFallbacks'
import { PasswordPolicy } from '../components/auth/PasswordPolicy'
import { useMe } from '../hooks/useMe'
import { evaluatePassword } from '../lib/passwordPolicy'

/**
 * Forced password-change interstitial: reachable only while
 * `mustChangePassword` is set (guard below redirects otherwise), and the
 * user cannot leave until the change succeeds (the app shell is not
 * rendered here).
 */
export function ChangePasswordPage() {
    const me = useMe()
    const navigate = useNavigate()
    const queryClient = useQueryClient()

    const [currentPassword, setCurrentPassword] = useState('')
    const [newPassword, setNewPassword] = useState('')
    const [confirmation, setConfirmation] = useState('')
    const [error, setError] = useState<string | null>(null)
    const [pending, setPending] = useState(false)

    const policy = evaluatePassword(newPassword)
    const confirmed = confirmation.length > 0 && confirmation === newPassword
    const canSubmit =
        policy.valid && confirmed && currentPassword.length > 0 && !pending

    if (me.isPending) {
        return <FullScreenLoading />
    }

    if (me.isError) {
        // Same contract as RequireAuth: only an explicit 401 means the
        // session is gone; anything else keeps the user on this page (they
        // still hold a valid cookie) with a retry prompt.
        if (me.error instanceof ApiError && me.error.status === 401) {
            return <Navigate to="/login" replace />
        }
        return <ServerUnreachable onRetry={() => me.refetch()} />
    }

    if (!me.data.mustChangePassword) {
        return <Navigate to="/" replace />
    }

    async function onSubmit(event: FormEvent) {
        event.preventDefault()
        setPending(true)
        setError(null)
        try {
            await apiPost('/api/auth/change-password', {
                currentPassword,
                newPassword,
            })
            await queryClient.invalidateQueries({ queryKey: ['me'] })
            navigate('/', { replace: true })
        } catch (err) {
            setError(
                err instanceof ApiError ? err.message : 'Password change failed'
            )
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
                    Change your password
                </h1>
                <p className="mt-1 text-sm text-text-muted">
                    You must set a new password before using the dashboard.
                </p>

                <div className="mt-6 flex flex-col gap-4">
                    <div className="flex flex-col gap-1">
                        <label
                            htmlFor="current"
                            className="text-sm font-medium"
                        >
                            Current password
                        </label>
                        <input
                            id="current"
                            type="password"
                            autoComplete="current-password"
                            required
                            value={currentPassword}
                            onChange={(e) => setCurrentPassword(e.target.value)}
                            className="rounded-md border bg-bg px-3 py-2 text-sm outline-none focus:border-accent"
                        />
                    </div>

                    <PasswordPolicy
                        password={newPassword}
                        confirmation={confirmation}
                        onPasswordChange={setNewPassword}
                        onConfirmationChange={setConfirmation}
                    />

                    {error && (
                        <p className="rounded-md border border-danger/40 bg-danger/10 px-3 py-2 text-sm text-danger">
                            {error}
                        </p>
                    )}

                    <button
                        type="submit"
                        disabled={!canSubmit}
                        className="rounded-md bg-accent px-4 py-2 text-sm font-medium text-accent-contrast hover:bg-accent-hover disabled:opacity-50"
                    >
                        {pending ? 'Saving…' : 'Save new password'}
                    </button>
                </div>
            </form>
        </div>
    )
}
