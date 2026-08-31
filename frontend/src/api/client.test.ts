import { afterEach, describe, expect, it, vi } from 'vitest'

import { ApiError, apiRequest } from './client'

function mockFetchResponse(response: Response) {
  const fetchMock = vi.fn<typeof fetch>()
  fetchMock.mockResolvedValue(response)
  vi.stubGlobal('fetch', fetchMock)
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
})
