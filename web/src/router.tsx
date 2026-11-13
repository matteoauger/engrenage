import { createBrowserRouter, RouterProvider } from 'react-router'

import { RequireAuth } from './components/auth/RequireAuth'
import { AppLayout } from './components/layout/AppLayout'
import { ChangePasswordPage } from './pages/ChangePasswordPage'
import { Dashboard } from './pages/Dashboard'
import { ErrorPage } from './pages/ErrorPage'
import { LoginPage } from './pages/LoginPage'
import { NotFound } from './pages/NotFound'

/**
 * Route table.
 *
 * Auth semantics:
 *   - /login and /change-password are standalone full-page routes
 *   - everything under / sits behind <RequireAuth> (which redirects to
 *     /login and enforces the forced password change)
 *   - render/router error → root errorElement (full page)
 */
const router = createBrowserRouter([
    { path: '/login', element: <LoginPage /> },
    { path: '/change-password', element: <ChangePasswordPage /> },
    {
        path: '/',
        element: (
            <RequireAuth>
                <AppLayout />
            </RequireAuth>
        ),
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
