/**
 * Password policy rules (NIST 800-63B-leaning): length is the main driver,
 * no composition rules; a small denylist catches the worst offenders.
 * The strength score is a UX hint, not a security oracle.
 *
 * NOTE: keep `COMMON_PASSWORDS` in sync with the backend list in
 * crates/engrenage/src/auth/password_policy.rs — duplicated by design
 * (shortlist, both sides fail closed); see docs/DECISIONS.md.
 */
export const MIN_PASSWORD_LENGTH = 8
export const MAX_PASSWORD_LENGTH = 128

const COMMON_PASSWORDS = [
    'password',
    'password1',
    'password123',
    'passw0rd',
    'p@ssw0rd',
    '12345678',
    '123456789',
    '1234567890',
    '1234qwer',
    'qwer1234',
    '1q2w3e4r5t',
    'qwerty123',
    'abc123456',
    'admin1234',
    'adminadmin',
    'administrator1',
    'superadmin1',
    'root1234',
    'toor1234',
    'letmein1',
    'welcome1',
    'iloveyou1',
    'changeme1',
    'test1234',
    'monkey123',
    'dragon123',
    'master123',
    'sunshine1',
    'princess1',
    'football1',
    'baseball1',
    'starwars1',
    'whatever1',
    'trustno1',
    'engrenage1',
    'engrenage123',
]

export type PasswordStrength = 'weak' | 'fair' | 'good'

export interface PasswordPolicyState {
    minLengthMet: boolean
    maxLengthMet: boolean
    notCommon: boolean
    strength: PasswordStrength
    valid: boolean
}

export function isCommonPassword(candidate: string): boolean {
    return COMMON_PASSWORDS.includes(candidate.toLowerCase())
}

export function evaluatePassword(candidate: string): PasswordPolicyState {
    const minLengthMet = candidate.length >= MIN_PASSWORD_LENGTH
    const maxLengthMet = candidate.length <= MAX_PASSWORD_LENGTH
    const notCommon = !isCommonPassword(candidate)
    return {
        minLengthMet,
        maxLengthMet,
        notCommon,
        strength: strengthOf(candidate),
        valid: minLengthMet && maxLengthMet && notCommon,
    }
}

/**
 * Heuristic strength hint: length tiers + character-class variety, minus a
 * penalty for obvious repetition (aaa, ....). Deliberately simple.
 */
function strengthOf(candidate: string): PasswordStrength {
    let score = 0
    if (candidate.length >= 12) score++
    if (candidate.length >= 16) score++
    const classes = [/[a-z]/, /[A-Z]/, /[0-9]/, /[^a-zA-Z0-9]/].filter((re) =>
        re.test(candidate)
    ).length
    if (classes >= 3) score++
    if (/(.)\1{2,}/.test(candidate)) score--
    return score >= 2 ? 'good' : score === 1 ? 'fair' : 'weak'
}
