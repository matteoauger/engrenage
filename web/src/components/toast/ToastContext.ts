import { createContext, useContext } from 'react'
import type { ToastInput } from './types'

export interface ToastManager {
    /**
     * Pushes a new notification in the toast system.
     **/
    push: (toast: ToastInput) => void

    /**
     * Clears the notifications. Intended for an user action.
     **/
    clear: () => void
}

export const ToastContext = createContext<ToastManager | null>(null)

export function useToast(): ToastManager {
    const ctx = useContext(ToastContext)
    if (!ctx) throw new Error('useToast must be used within <ToastProvider>')
    return ctx
}
