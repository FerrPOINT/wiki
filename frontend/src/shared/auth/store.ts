// Fleet-standard auth store from @sdlc/ui (token in memory only).
import { createAuthStore } from '@sdlc/ui/auth'
import { getSafeBrowserStorage } from '@/shared/lib/browser-storage'

export const ssoConfig = {
  issuer: import.meta.env.VITE_AUTH_ISSUER ?? 'http://localhost:7701',
  clientId: 'wiki',
}

export const useAuthStore = createAuthStore({
  storageKey: 'wiki-auth',
  legacyKeys: ['wiki-refresh-token'],
})
export type { AuthState } from '@sdlc/ui/auth'

const REFRESH_KEY = 'wiki-refresh-token'

export function clearLegacyRefreshToken(): void {
  try {
    getSafeBrowserStorage().removeItem(REFRESH_KEY)
  } catch {
    // Storage can be unavailable in private or opaque browser contexts.
  }
}
