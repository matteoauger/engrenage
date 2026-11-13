import type { IdentifiedToast } from './types'
import { faX } from '@fortawesome/free-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { cn } from '../../lib/cn'

const VARIANT_STYLES: Record<IdentifiedToast['variant'], string> = {
    info: 'border-info',
    success: 'border-success',
    error: 'border-danger',
}

// Toast component : displays notifications.
export function Toast({
    toast,
    onClose,
    onExited,
}: {
    toast: IdentifiedToast
    onClose: (id: number) => void
    onExited: (id: number) => void
}) {
    return (
        <div
            role="status"
            className={cn(
                'rounded-lg border-l-4 bg-surface p-3 shadow-md',
                VARIANT_STYLES[toast.variant],
                toast.isClosing ? 'animate-toast-out' : 'animate-toast-in'
            )}
            onAnimationEnd={(event) => {
                if (toast.isClosing && event.animationName === 'toast-out') {
                    onExited(toast.id)
                }
            }}
        >
            <div className="flex items-start justify-between gap-2">
                <p className="text-sm font-medium">{toast.title}</p>
                <button
                    type="button"
                    onClick={() => onClose(toast.id)}
                    className="flex text-text-muted w-4 h-4 hover:text-text hover:cursor-pointer"
                    aria-label="Dismiss notification"
                >
                    <FontAwesomeIcon icon={faX} />
                </button>
            </div>
            {toast.detail && (
                <p className="mt-1 text-xs text-text-muted">{toast.detail}</p>
            )}
        </div>
    )
}
