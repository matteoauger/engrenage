/** Typed error carrying the HTTP status so callers can branch on 4xx vs 5xx. */
export class ApiError extends Error {
    status: number

    constructor(status: number, message: string) {
        super(message)
        this.name = 'ApiError'
        this.status = status
    }
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
    let response: Response
    try {
        response = await fetch(path, {
            ...init,
            headers: {
                // Content negotiation only makes sense for bodied requests;
                // bodiless GETs stay headerless.
                ...(init?.body ? { 'Content-Type': 'application/json' } : {}),
                ...init?.headers,
            },
        })
    } catch {
        // Network-level failure (backend down, DNS, CORS...).
        throw new ApiError(0, 'network error — backend unreachable')
    }

    if (!response.ok) {
        // Backend error shape: {"error": "message"} (see crates/engrenage/src/error.rs).
        let message = `HTTP ${response.status}`
        try {
            const body = (await response.json()) as { error?: string }
            if (body.error) message = body.error
        } catch {
            // Non-JSON body (proxy, dev server...) — keep the generic message.
        }
        throw new ApiError(response.status, message)
    }

    // The backend always speaks JSON today, but a future 204-style endpoint or
    // a proxy hiccup must not surface to callers as a raw SyntaxError.
    try {
        return (await response.json()) as T
    } catch {
        throw new ApiError(response.status, 'invalid response body')
    }
}

export const apiGet = <T>(path: string): Promise<T> => request<T>(path)

export const apiPost = <T>(path: string, body: unknown): Promise<T> =>
    request<T>(path, { method: 'POST', body: JSON.stringify(body) })
