import {
    useCallback,
    useEffect,
    useMemo,
    useRef,
    useState,
    type ReactNode,
} from 'react'
import { ToastContext } from './ToastContext'
import { ToastList } from './ToastList'
import type { IdentifiedToast, ToastInput, ToastVariant } from './types'

const AUTO_DISMISS_MS: Record<ToastVariant, number> = {
    info: 4000,
    success: 4000,
    error: 8000,
}

/**
 * Owns the toast queue and mounts the viewport (children first, so the
 * viewport stays outside the router and toasts survive route changes).
 *
 * Closing is two-phase: close(id) flags the entry so its exit animation plays,
 * then <Toast>'s onAnimationEnd calls remove(id) to actually drop it.
 */
export function ToastProvider({ children }: { children: ReactNode }) {
    const [toasts, setToasts] = useState<IdentifiedToast[]>([])
    const nextId = useRef(0)
    const timers = useRef(new Map<number, number>())

    /** Phase 2: drop the entry, canceling any pending auto-dismiss timer. */
    const remove = useCallback((id: number) => {
        const timer = timers.current.get(id)
        if (timer !== undefined) {
            window.clearTimeout(timer)
            timers.current.delete(id)
        }
        // Updater form: reads the latest queue inside the setter, no stale
        // closure — this is what lets toasts stack up.
        setToasts((current) => current.filter((toast) => toast.id !== id))
    }, [])

    /**
     * Phase 1: flag the toast as closing so its exit animation plays.
     *
     * Also arms a fallback removal timer: if animations are disabled in the
     * browser (accessibility tooling, UA settings), `animationend` never
     * fires and the entry would otherwise stay in the DOM forever. The
     * normal path remains `animationend` → remove (see Toast.tsx).
     */
    const close = useCallback(
        (id: number) => {
            // Closing is terminal: cancel any pending auto-dismiss timer.
            const pending = timers.current.get(id)
            if (pending !== undefined) window.clearTimeout(pending)

            setToasts((current) =>
                current.map((toast) =>
                    toast.id === id ? { ...toast, isClosing: true } : toast
                )
            )

            // Exit animation is 180ms; 400ms gives a comfortable margin.
            const fallback = window.setTimeout(() => {
                timers.current.delete(id)
                remove(id)
            }, 400)
            timers.current.set(id, fallback)
        },
        [remove]
    )

    const push = useCallback(
        (toast: ToastInput) => {
            const id = nextId.current++
            // Updater form: append to the latest state, not to the `toasts`
            // captured by this memoized callback. Identical toasts stack up
            // on purpose (e.g. a button clicked multiple times).
            setToasts((current) => [...current, { ...toast, id }])

            // auto dismiss (phase 1 only; the animation triggers removal)
            const timeout = AUTO_DISMISS_MS[toast.variant]
            const timer = window.setTimeout(() => close(id), timeout)
            timers.current.set(id, timer)
        },
        [close]
    )

    const clear = useCallback(() => {
        for (const timer of timers.current.values()) window.clearTimeout(timer)
        timers.current.clear()
        setToasts([])
    }, [])

    // Sweep remaining timers if the provider unmounts.
    useEffect(() => {
        const pending = timers.current
        return () => {
            for (const timer of pending.values()) window.clearTimeout(timer)
            pending.clear()
        }
    }, [])

    const value = useMemo(() => ({ push, clear }), [push, clear])

    return (
        <ToastContext.Provider value={value}>
            {children}
            <ToastList toasts={toasts} onClose={close} onExited={remove} />
        </ToastContext.Provider>
    )
}
