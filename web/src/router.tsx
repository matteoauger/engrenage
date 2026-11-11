import { createBrowserRouter, RouterProvider } from 'react-router'

import { AppLayout } from './components/layout/AppLayout'
import { Dashboard } from './pages/Dashboard'
import { ErrorPage } from './pages/ErrorPage'
import { NotFound } from './pages/NotFound'

/**
 * Route table.
 *
 * Error semantics:
 *   - unknown URL        → catch-all `*` (inside the shell): sidebar stays usable
 *   - render/router error → root errorElement (full page): shell untrustworthy
 */
const router = createBrowserRouter([
    {
        path: '/',
        element: <AppLayout />,
        errorElement: <ErrorPage />,
        children: [
            { index: true, element: <Dashboard /> },
            { path: '*', element: <NotFound /> },
        ],
    },
])

export function Router() {
    return <RouterProvider router={router} />
}
