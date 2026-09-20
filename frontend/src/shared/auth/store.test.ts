import { beforeEach, describe, expect, it, vi } from 'vitest'

describe('Wiki SSO storage migration', () => {
  beforeEach(() => {
    vi.resetModules()
    localStorage.clear()
  })

  it('purges the old refresh token when the auth store initializes', async () => {
    localStorage.setItem('wiki-refresh-token', 'legacy-secret')

    const { useAuthStore } = await import('./store')

    expect(useAuthStore.getState().token).toBeNull()
    expect(localStorage.getItem('wiki-refresh-token')).toBeNull()
  })
})
