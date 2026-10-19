import { afterEach, expect, it, vi } from 'vitest'

vi.mock('@/shared/auth/store', () => ({
  useAuthStore: { getState: () => ({ token: null, logout: vi.fn() }) },
}))

afterEach(() => {
  vi.unstubAllEnvs()
  vi.unstubAllGlobals()
  vi.resetModules()
})

it.each([
  { name: 'root', base: '/', api: undefined, expected: '/api/v1/health' },
  { name: 'mounted', base: '/wiki/', api: undefined, expected: '/wiki/api/v1/health' },
  {
    name: 'explicit origin',
    base: '/wiki/',
    api: 'https://api.example.test/api/v1',
    expected: 'https://api.example.test/api/v1/health',
  },
  { name: 'explicit root', base: '/wiki/', api: '', expected: '/api/v1/health' },
])('keeps requests inside the $name deployment', async ({ base, api, expected }) => {
  vi.resetModules()
  vi.stubEnv('BASE_URL', base)
  vi.stubEnv('VITE_API_BASE_URL', api)
  const fetchMock = vi
    .fn()
    .mockResolvedValue(
      new Response('{}', { status: 200, headers: { 'Content-Type': 'application/json' } }),
    )
  vi.stubGlobal('fetch', fetchMock)
  const client = await import('./client')
  await client.apiRequest('/api/v1/health')
  const input = fetchMock.mock.calls[0]![0] as string | Request
  const url = new URL(typeof input === 'string' ? input : input.url, window.location.origin)
  expect(url.href).toBe(new URL(expected, window.location.origin).href)
})
