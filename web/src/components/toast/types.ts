export type ToastVariant = 'info' | 'success' | 'error'

export interface ToastInput {
    variant: ToastVariant
    title: string
    detail?: string
}

/** Internal shape shared provider → list. `isClosing` is provider-internal:
 *  flags the exit-animation phase of the toast. */
export interface IdentifiedToast extends ToastInput {
    id: number
    isClosing?: boolean
}
