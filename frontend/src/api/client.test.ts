import { afterEach, describe, expect, it, vi } from 'vitest'

import { ApiError, apiBlobRequest, apiRequest } from './client'
import { storeRefreshToken, useAuthStore } from '@/shared/auth/store'

function mockFetchResponse(response: Response) {
  const fetchMock = vi.fn<typeof fetch>()
  fetchMock.mockResolvedValue(response)
  vi.stubGlobal('fetch', fetchMock)
  return fetchMock
}

function requestHeaders(fetchMock: ReturnType<typeof mockFetchResponse>): Headers {
  const headers = fetchMock.mock.calls[0]?.[1]?.headers
  expect(headers).toBeInstanceOf(Headers)
  return headers as Headers
}

async function expectApiError(request: Promise<unknown>): Promise<ApiError> {
  try {
    await request
  } catch (error) {
    expect(error).toBeInstanceOf(ApiError)
    return error as ApiError
  }
  throw new Error('Expected request to reject')
}

describe('apiRequest error handling', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
    storeRefreshToken(null)
    useAuthStore.setState({
      token: null,
      userId: null,
      email: null,
      username: null,
      displayName: null,
    })
  })

  it('renders legacy string error envelopes', async () => {
    mockFetchResponse(
      new Response(JSON.stringify({ error: 'forbidden' }), {
        status: 403,
        statusText: 'Forbidden',
      }),
    )

    await expect(apiRequest('/api/v1/spaces')).rejects.toMatchObject({
      name: 'ApiError',
      status: 403,
      code: 'FORBIDDEN',
      message: 'forbidden',
      details: [],
    })
  })

  it('renders structured API error envelopes', async () => {
    mockFetchResponse(
      new Response(
        JSON.stringify({
          error: {
            code: 'VALIDATION_ERROR',
            message: 'Request validation failed',
            requestId: 'req-1',
            details: [{ field: 'summary', message: 'required' }],
          },
        }),
        {
          status: 400,
          statusText: 'Bad Request',
        },
      ),
    )

    const error = await expectApiError(apiRequest('/api/v1/documents'))

    expect(error).toMatchObject({
      status: 400,
      code: 'VALIDATION_ERROR',
      requestId: 'req-1',
      details: [{ field: 'summary', message: 'required' }],
      message: 'Request validation failed; details=summary: required; requestId=req-1',
    })
  })

  it('uses response request id headers when error body omits request id', async () => {
    mockFetchResponse(
      new Response(
        JSON.stringify({
          error: {
            code: 'FORBIDDEN',
            message: 'forbidden',
          },
        }),
        {
          status: 403,
          statusText: 'Forbidden',
          headers: { 'X-Request-ID': 'req-header-1' },
        },
      ),
    )

    const error = await expectApiError(apiRequest('/api/v1/spaces'))

    expect(error).toMatchObject({
      status: 403,
      code: 'FORBIDDEN',
      requestId: 'req-header-1',
      message: 'forbidden; requestId=req-header-1',
    })
  })

  it('returns blob responses with download metadata', async () => {
    mockFetchResponse(
      new Response('downloaded bytes', {
        status: 200,
        headers: {
          'Content-Disposition': 'attachment; filename="build.log"',
          'Content-Type': 'text/plain',
        },
      }),
    )

    const download = await apiBlobRequest('/api/v1/attachments/attachment-1/download')

    expect(download.contentType).toBe('text/plain')
    expect(download.fileName).toBe('build.log')
    expect(download.sizeBytes).toBe('downloaded bytes'.length)
    expect(download.blob).toMatchObject({ size: 'downloaded bytes'.length, type: 'text/plain' })
  })

  it('adds request id headers to API requests', async () => {
    const fetchMock = mockFetchResponse(
      new Response(JSON.stringify({ spaces: [] }), { status: 200 }),
    )

    await apiRequest('/api/v1/spaces')

    expect(requestHeaders(fetchMock).get('X-Request-ID')).toMatch(/^wiki-ui-/)
  })

  it('preserves caller-provided request id headers', async () => {
    const fetchMock = mockFetchResponse(
      new Response(JSON.stringify({ spaces: [] }), { status: 200 }),
    )

    await apiRequest('/api/v1/spaces', {
      headers: {
        'X-Request-ID': 'req-caller-1',
      },
    })

    expect(requestHeaders(fetchMock).get('X-Request-ID')).toBe('req-caller-1')
  })

  it('adds request id headers to token refresh requests', async () => {
    useAuthStore.setState({ token: 'expired-token' })
    storeRefreshToken('refresh-token')
    const fetchMock = vi.fn<typeof fetch>()
    fetchMock
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ error: { code: 'UNAUTHORIZED', message: 'unauthorized' } }), {
          status: 401,
        }),
      )
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({ access_token: 'fresh-token', refresh_token: 'next-refresh' }),
          {
            status: 200,
          },
        ),
      )
      .mockResolvedValueOnce(new Response(JSON.stringify({ spaces: [] }), { status: 200 }))
    vi.stubGlobal('fetch', fetchMock)

    await apiRequest('/api/v1/spaces')

    const refreshHeaders = fetchMock.mock.calls[1]?.[1]?.headers
    expect(fetchMock.mock.calls[1]?.[0]).toBe('/api/v1/auth/refresh')
    expect(refreshHeaders).toBeInstanceOf(Headers)
    expect((refreshHeaders as Headers).get('X-Request-ID')).toMatch(/^wiki-ui-/)
  })
})
