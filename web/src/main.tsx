import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { QueryClientProvider } from '@tanstack/react-query'

import { ToastProvider } from './components/toast'
import { queryClient } from './lib/queryClient'
import { initSystemTheme } from './lib/theme'
import { Router } from './router'
import './index.css'

initSystemTheme()

createRoot(document.getElementById('root')!).render(
    <StrictMode>
        <QueryClientProvider client={queryClient}>
            <ToastProvider>
                <Router />
            </ToastProvider>
        </QueryClientProvider>
    </StrictMode>
)
