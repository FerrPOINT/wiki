import { readRefreshToken, storeRefreshToken, useAuthStore } from '@/shared/auth/store'

const rawBaseUrl = import.meta.env.VITE_API_BASE_URL ?? ''
export const apiBaseUrl = rawBaseUrl.replace(/\/api\/v1\/?$/, '')

type ApiRequestOptions = Omit<RequestInit, 'body'> & {
  body?: unknown
  skipAuth?: boolean
}

let refreshPromise: Promise<boolean> | null = null

function buildUrl(path: string): string {
  if (path.startsWith('http://') || path.startsWith('https://')) return path
  return `${apiBaseUrl}${path.startsWith('/') ? path : `/${path}`}`
}

async function readErrorMessage(response: Response): Promise<string> {
  try {
    const body = (await response.json()) as { error?: string; message?: string }
    return body.error ?? body.message ?? response.statusText
  } catch {
    return response.statusText
  }
}

async function refreshAccessToken(): Promise<boolean> {
  if (refreshPromise) return refreshPromise

  refreshPromise = (async () => {
    try {
      const refreshToken = readRefreshToken()
      const response = await fetch(buildUrl('/api/v1/auth/refresh'), {
        method: 'POST',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(refreshToken ? { refresh_token: refreshToken } : {}),
      })

      if (!response.ok) {
        useAuthStore.getState().logout()
        window.location.href = '/login'
        return false
      }

      const data = (await response.json()) as {
        access_token?: string
        refresh_token?: string | null
      }
      if (data.refresh_token !== undefined) storeRefreshToken(data.refresh_token)
      if (data.access_token) useAuthStore.setState({ token: data.access_token })
      return Boolean(data.access_token)
    } catch {
      useAuthStore.getState().logout()
      window.location.href = '/login'
      return false
    } finally {
      refreshPromise = null
    }
  })()

  return refreshPromise
}

function shouldRefresh(path: string, response: Response): boolean {
  return response.status === 401 && !path.includes('/api/v1/auth/')
}

async function send(path: string, options: ApiRequestOptions): Promise<Response> {
  const { body, headers, skipAuth, ...init } = options
  const requestHeaders = new Headers(headers)
  const token = useAuthStore.getState().token

  if (body !== undefined && !requestHeaders.has('Content-Type')) {
    if (!(body instanceof FormData)) requestHeaders.set('Content-Type', 'application/json')
  }
  if (token && !skipAuth) {
    requestHeaders.set('Authorization', `Bearer ${token}`)
  }

  return fetch(buildUrl(path), {
    ...init,
    credentials: init.credentials ?? 'include',
    headers: requestHeaders,
    body: body === undefined || body instanceof FormData ? body : JSON.stringify(body),
  })
}

export async function apiRequest<T>(path: string, options: ApiRequestOptions = {}): Promise<T> {
  let response = await send(path, options)
  if (shouldRefresh(path, response)) {
    const refreshed = await refreshAccessToken()
    if (refreshed) response = await send(path, options)
  }

  if (!response.ok) {
    throw new Error(await readErrorMessage(response))
  }
  if (response.status === 204) return undefined as T
  return (await response.json()) as T
}
