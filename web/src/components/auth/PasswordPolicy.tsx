import {
    evaluatePassword,
    MAX_PASSWORD_LENGTH,
    type PasswordStrength,
} from '../../lib/passwordPolicy'
import { cn } from '../../lib/cn'

const STRENGTH_HINTS: Record<
    PasswordStrength,
    { text: string; className: string }
> = {
    weak: {
        text: 'Weak password — consider a longer one',
        className: 'text-danger',
    },
    fair: {
        text: 'Fair password',
        className: 'text-warning',
    },
    good: {
        text: 'Good password',
        className: 'text-success',
    },
}

function CheckItem({ met, label }: { met: boolean; label: string }) {
    return (
        <li
            className={cn(
                'flex items-center gap-1.5 text-xs',
                met ? 'text-success' : 'text-text-muted'
            )}
        >
            <span aria-hidden="true">{met ? '✓' : '✗'}</span>
            {label}
        </li>
    )
}

/**
 * New-password fields with the live interactive policy: requirement
 * checklist, strength hint and confirm-field match/mismatch.
 *
 * Reusable for any credential-creation flow (change-password here, admin
 * user creation in M4). The parent owns the state; submit gating is done by
 * the parent via `evaluatePassword(password).valid && password === confirmation`.
 */
export function PasswordPolicy({
    password,
    confirmation,
    onPasswordChange,
    onConfirmationChange,
}: {
    password: string
    confirmation: string
    onPasswordChange: (value: string) => void
    onConfirmationChange: (value: string) => void
}) {
    const policy = evaluatePassword(password)
    const mismatch = confirmation.length > 0 && confirmation !== password
    const strength = STRENGTH_HINTS[policy.strength]

    return (
        <div className="flex flex-col gap-4">
            <div className="flex flex-col gap-1">
                <label htmlFor="new-password" className="text-sm font-medium">
                    New password
                </label>
                <input
                    id="new-password"
                    type="password"
                    autoComplete="new-password"
                    required
                    value={password}
                    onChange={(e) => onPasswordChange(e.target.value)}
                    className="rounded-md border bg-bg px-3 py-2 text-sm outline-none focus:border-accent"
                />
            </div>
            <div className="flex flex-col gap-1">
                <label
                    htmlFor="confirm-password"
                    className="text-sm font-medium"
                >
                    Confirm new password
                </label>
                <input
                    id="confirm-password"
                    type="password"
                    autoComplete="new-password"
                    required
                    value={confirmation}
                    onChange={(e) => onConfirmationChange(e.target.value)}
                    className={cn(
                        'rounded-md border bg-bg px-3 py-2 text-sm outline-none focus:border-accent',
                        mismatch && 'border-danger focus:border-danger'
                    )}
                />
                {mismatch && (
                    <p className="text-xs text-danger">
                        Passwords do not match
                    </p>
                )}
            </div>

            <ul
                className="flex flex-col gap-1"
                aria-label="Password requirements"
            >
                <CheckItem
                    met={policy.minLengthMet}
                    label="At least 8 characters"
                />
                <CheckItem
                    met={policy.maxLengthMet}
                    label={`At most ${MAX_PASSWORD_LENGTH} characters`}
                />
                <CheckItem
                    met={policy.notCommon}
                    label="Not a common password"
                />
            </ul>
            {password.length > 0 && (
                <p className={cn('text-xs', strength.className)}>
                    {strength.text}
                </p>
            )}
        </div>
    )
}
