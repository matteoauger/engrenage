import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

/**
 * Compose class names: conditional values, falsy-tolerant, with Tailwind
 * conflict resolution (later classes win: cn('p-3', 'p-4') → 'p-4').
 *
 * Usage: cn('base-class', isActive && 'bg-accent', className)
 */
export function cn(...inputs: ClassValue[]): string {
    return twMerge(clsx(inputs))
}
