import { QueryClient } from '@tanstack/react-query'

/** Singleton query client: config for all dashboard widgets lives here. */
export const queryClient = new QueryClient({
    defaultOptions: {
        queries: {
            // Dashboard widgets: refetching on window focus is noise for a control panel.
            refetchOnWindowFocus: false,
        },
    },
})
