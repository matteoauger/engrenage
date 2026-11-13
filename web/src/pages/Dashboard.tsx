import { useEffect } from 'react'
import { useQuery } from '@tanstack/react-query'
import { apiGet, ApiError } from '../api/client'
import { useToast } from '../components/toast'
import { cn } from '../lib/cn'

interface HealthResponse {
    status: string
}

function StatusCard({
    title,
    children,
}: {
    title: string
    children: React.ReactNode
}) {
    return (
        <div className="rounded-lg border bg-surface p-4">
            <h3 className="text-sm font-medium text-text-muted">{title}</h3>
            <div className="mt-2">{children}</div>
        </div>
    )
}

export function Dashboard() {
    const { push } = useToast()

    const health = useQuery({
        queryKey: ['health'],
        queryFn: () => apiGet<HealthResponse>('/health'),
        retry: 1,
    })

    useEffect(() => {
        if (!health.isError) return
        const error = health.error as ApiError
        push({
            variant: 'error',
            title: 'Backend unreachable',
            detail: error instanceof ApiError ? error.message : 'Unknown error',
        })
    }, [health.isError, health.error, push])

    const backendUp = health.isSuccess

    return (
        <div className="flex flex-col gap-6">
            <div>
                <h1 className="text-2xl font-semibold tracking-tight">
                    Dashboard
                </h1>
                <p className="mt-1 text-sm text-text-muted">
                    Server status and activity at a glance.
                </p>
            </div>

            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                <StatusCard title="Backend">
                    {health.isPending ? (
                        <span className="text-sm text-text-muted">
                            checking…
                        </span>
                    ) : (
                        <span className="inline-flex items-center gap-2 text-sm">
                            <span
                                className={cn(
                                    'size-2 rounded-full',
                                    backendUp ? 'bg-success' : 'bg-danger'
                                )}
                            />
                            {backendUp ? 'reachable' : 'unreachable'}
                        </span>
                    )}
                </StatusCard>

                <StatusCard title="Server">
                    <span className="text-sm text-text-muted">
                        server lifecycle lands in v0.2.0
                    </span>
                </StatusCard>

                <StatusCard title="Players">
                    <span className="text-sm text-text-muted">
                        player management lands in v0.5.0
                    </span>
                </StatusCard>
            </div>

            <div className="rounded-lg border border-dashed bg-surface/50 p-10 text-center">
                <p className="font-medium">Nothing else here yet</p>
                <p className="mt-1 text-sm text-text-muted">
                    The dashboard fills up as milestones land — see the roadmap.
                </p>
            </div>
        </div>
    )
}
