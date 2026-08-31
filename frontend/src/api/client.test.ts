import { afterEach, describe, expect, it, vi } from 'vitest'

import { apiRequest } from './client'

function mockFetchResponse(response: Response) {
  const fetchMock = vi.fn<typeof fetch>()
  fetchMock.mockResolvedValue(response)
  vi.stubGlobal('fetch', fetchMock)
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

    await expect(apiRequest('/api/v1/spaces')).rejects.toThrow('forbidden')
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

    await expect(apiRequest('/api/v1/documents')).rejects.toThrow(
      'Request validation failed; details=summary: required; requestId=req-1',
    )
  })
})
