import { act, cleanup, renderHook } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const sso = vi.hoisted(() => ({ restoreSso: vi.fn(), endSso: vi.fn() }))
vi.mock('@sdlc/ui/sso', async (importOriginal) => ({
  ...(await importOriginal<typeof import('@sdlc/ui/sso')>()),
  ...sso,
}))

beforeEach(() => {
  vi.resetModules()
  vi.clearAllMocks()
  vi.stubEnv('VITE_AUTH_ISSUER', 'http://localhost:8701')
  sso.restoreSso.mockRejectedValue(new Error('Fixture login required'))
})

afterEach(() => {
  cleanup()
  vi.unstubAllEnvs()
})

describe('deployment-owned SSO client', () => {
  it.each([
    { configured: undefined, expected: 'wiki' },
    { configured: 'wiki-dev', expected: 'wiki-dev' },
  ])('restores Wiki with $expected', async ({ configured, expected }) => {
    vi.stubEnv('VITE_AUTH_CLIENT_ID', configured)
    const { restoreWikiSession } = await import('./session')
    await expect(restoreWikiSession()).rejects.toThrow('Fixture login required')
    expect(sso.restoreSso).toHaveBeenCalledWith({
      issuer: 'http://localhost:8701',
      clientId: expected,
    })
  })

  it('uses the same configured client for logout', async () => {
    vi.stubEnv('VITE_AUTH_CLIENT_ID', 'wiki-dev')
    const { QueryClient, QueryClientProvider } = await import('@tanstack/react-query')
    const { useLogout } = await import('@/shared/api/hooks')
    const client = new QueryClient({ defaultOptions: { mutations: { gcTime: 0 } } })
    try {
      const { result } = renderHook(() => useLogout(), {
        wrapper: ({ children }) => (
          <QueryClientProvider client={client}>{children}</QueryClientProvider>
        ),
      })
      await act(async () => {
        await result.current.mutateAsync()
      })
      expect(sso.endSso).toHaveBeenCalledWith({
        issuer: 'http://localhost:8701',
        clientId: 'wiki-dev',
      })
    } finally {
      cleanup()
      client.clear()
    }
  })
})
