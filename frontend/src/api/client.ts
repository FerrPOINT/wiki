import { readRefreshToken, storeRefreshToken, useAuthStore } from '@/shared/auth/store'

const rawBaseUrl = import.meta.env.VITE_API_BASE_URL ?? ''
export const apiBaseUrl = rawBaseUrl.replace(/\/api\/v1\/?$/, '')
const requestIdHeader = 'X-Request-ID'

type ApiRequestOptions = Omit<RequestInit, 'body'> & {
  body?: unknown
  skipAuth?: boolean
}

export type ApiErrorDetail = {
  field?: string
  message?: string
}

type ApiErrorBody = {
  error?: unknown
  message?: unknown
}

export type ApiBlobResponse = {
  blob: Blob
  contentType: string
  fileName?: string
  sizeBytes: number
}

type ApiErrorInit = {
  status: number
  statusText: string
  code?: string
  message?: string
  requestId?: string
  details?: ApiErrorDetail[]
}

export class ApiError extends Error {
  readonly status: number
  readonly code: string
  readonly requestId?: string
  readonly details: ApiErrorDetail[]

  constructor({ status, statusText, code, message, requestId, details = [] }: ApiErrorInit) {
    const safeCode = code ?? defaultCodeForStatus(status)
    const safeMessage = message ?? statusText
    super(formatApiErrorMessage(safeMessage, requestId, details))
    this.name = 'ApiError'
    this.status = status
    this.code = safeCode
    this.requestId = requestId
    this.details = details
  }
}

let refreshPromise: Promise<boolean> | null = null

function buildUrl(path: string): string {
  if (path.startsWith('http://') || path.startsWith('https://')) return path
  return `${apiBaseUrl}${path.startsWith('/') ? path : `/${path}`}`
}

function createRequestId(): string {
  const randomUUID = globalThis.crypto?.randomUUID?.bind(globalThis.crypto)
  if (randomUUID) return `wiki-ui-${randomUUID()}`
  return `wiki-ui-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`
}

function ensureRequestId(headers: Headers): void {
  if (!headers.has(requestIdHeader)) headers.set(requestIdHeader, createRequestId())
}

function responseRequestId(response: Response): string | undefined {
  return response.headers.get(requestIdHeader) ?? undefined
}

async function readApiError(response: Response): Promise<ApiError> {
  try {
    const body = (await response.json()) as ApiErrorBody

    if (typeof body.error === 'string') {
      return new ApiError({
        status: response.status,
        statusText: response.statusText,
        message: body.error,
        requestId: responseRequestId(response),
      })
    }
    if (body.error && typeof body.error === 'object') {
      return readStructuredError(body.error as Record<string, unknown>, response)
    }
    if (typeof body.message === 'string') {
      return new ApiError({
        status: response.status,
        statusText: response.statusText,
        message: body.message,
        requestId: responseRequestId(response),
      })
    }
    return new ApiError({
      status: response.status,
      statusText: response.statusText,
      requestId: responseRequestId(response),
    })
  } catch {
    return new ApiError({
      status: response.status,
      statusText: response.statusText,
      requestId: responseRequestId(response),
    })
  }
}

function readStructuredError(error: Record<string, unknown>, response: Response): ApiError {
  const code = typeof error.code === 'string' ? error.code : undefined
  const message = typeof error.message === 'string' ? error.message : undefined
  const requestId =
    typeof error.requestId === 'string'
      ? error.requestId
      : typeof error.request_id === 'string'
        ? error.request_id
        : responseRequestId(response)
  const details = readErrorDetails(error.details)

  return new ApiError({
    status: response.status,
    statusText: response.statusText,
    code,
    message,
    requestId,
    details,
  })
}

function readErrorDetails(details: unknown): ApiErrorDetail[] {
  if (!Array.isArray(details)) return []

  return details.flatMap((detail): ApiErrorDetail[] => {
    if (!detail || typeof detail !== 'object') return []
    const record = detail as Record<string, unknown>
    const field = typeof record.field === 'string' ? record.field : undefined
    const message = typeof record.message === 'string' ? record.message : undefined

    if (!field && !message) return []
    return [
      {
        ...(field ? { field } : {}),
        ...(message ? { message } : {}),
      },
    ]
  })
}

function formatApiErrorMessage(
  message: string,
  requestId?: string,
  details: ApiErrorDetail[] = [],
): string {
  const formattedDetails = formatErrorDetails(details)
  const parts = [
    message,
    formattedDetails ? `details=${formattedDetails}` : undefined,
    requestId ? `requestId=${requestId}` : undefined,
  ].filter((part): part is string => Boolean(part))

  return parts.join('; ')
}

function formatErrorDetails(details: ApiErrorDetail[]): string | undefined {
  const messages = details
    .map(({ field, message }) => {
      if (field && message) return `${field}: ${message}`
      return field ?? message
    })
    .filter((detail): detail is string => Boolean(detail))

  return messages.length ? messages.join(', ') : undefined
}

function defaultCodeForStatus(status: number): string {
  switch (status) {
    case 400:
      return 'VALIDATION_ERROR'
    case 401:
      return 'UNAUTHORIZED'
    case 403:
      return 'FORBIDDEN'
    case 404:
      return 'NOT_FOUND'
    case 409:
      return 'CONFLICT'
    default:
      return status >= 500 ? 'INTERNAL_ERROR' : 'UNKNOWN'
  }
}

export async function refreshAccessToken(): Promise<boolean> {
  if (refreshPromise) return refreshPromise

  refreshPromise = (async () => {
    try {
      const refreshToken = readRefreshToken()
      const headers = new Headers({ 'Content-Type': 'application/json' })
      ensureRequestId(headers)
      const response = await fetch(buildUrl('/api/v1/auth/refresh'), {
        method: 'POST',
        credentials: 'include',
        headers,
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
  ensureRequestId(requestHeaders)

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
    const error = await readApiError(response)
    throw error
  }
  if (response.status === 204) return undefined as T
  return (await response.json()) as T
}

export async function apiBlobRequest(
  path: string,
  options: ApiRequestOptions = {},
): Promise<ApiBlobResponse> {
  let response = await send(path, options)
  if (shouldRefresh(path, response)) {
    const refreshed = await refreshAccessToken()
    if (refreshed) response = await send(path, options)
  }

  if (!response.ok) {
    const error = await readApiError(response)
    throw error
  }

  const blob = await response.blob()
  const contentType = response.headers.get('Content-Type') ?? blob.type
  const fileName = filenameFromContentDisposition(response.headers.get('Content-Disposition'))

  return {
    blob,
    contentType,
    fileName,
    sizeBytes: blob.size,
  }
}

function filenameFromContentDisposition(value: string | null): string | undefined {
  if (!value) return undefined
  const encoded = value.match(/filename\*=UTF-8''([^;]+)/i)?.[1]
  if (encoded) {
    try {
      return decodeURIComponent(encoded)
    } catch {
      return encoded
    }
  }

  const quoted = value.match(/filename="([^"]+)"/i)?.[1]
  if (quoted) return quoted

  return value.match(/filename=([^;]+)/i)?.[1]?.trim()
}
