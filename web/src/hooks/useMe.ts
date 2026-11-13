import { useQuery } from '@tanstack/react-query'
import { apiGet } from '../api/client'

export interface MeResponse {
    username: string
    mustChangePassword: boolean
}

/**
 * Query the authenticated account (`/api/auth/me`).
 * 401 when no valid session exists; `mustChangePassword` drives the forced
 * password-change flow.
 */
export function useMe() {
    return useQuery({
        queryKey: ['me'],
        queryFn: () => apiGet<MeResponse>('/api/auth/me'),
        retry: false,
    })
}
