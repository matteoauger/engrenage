import { Toast } from './Toast'
import type { IdentifiedToast } from './types'

/**
 * Wrapper for the toast set to be displayed. Anchors the toasts on the bottom
 * right of the viewport. `onClose` starts the exit animation, `onExited`
 * removes the entry once that animation has played.
 **/
export function ToastList({
    toasts,
    onClose,
    onExited,
}: {
    toasts: IdentifiedToast[]
    onClose: (id: number) => void
    onExited: (id: number) => void
}) {
    return (
        <div className="fixed right-4 bottom-4 z-50 flex w-80 flex-col gap-2">
            {toasts.map((toast) => (
                <Toast
                    key={toast.id}
                    toast={toast}
                    onClose={onClose}
                    onExited={onExited}
                />
            ))}
        </div>
    )
}
