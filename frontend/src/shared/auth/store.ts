// Fleet-standard auth store from @sdlc/ui (token in memory only).
import { createAuthStore } from '@sdlc/ui/auth'
import { getSafeBrowserStorage } from '@/shared/lib/browser-storage'

export const useAuthStore = createAuthStore({
  storageKey: 'wiki-auth',
})
export type { AuthState } from '@sdlc/ui/auth'

// Local extra: legacy refresh-token storage helpers (cookie flow primary).
const REFRESH_KEY = 'wiki-refresh-token'

export function storeRefreshToken(token: string | null): void {
  try {
    const storage = getSafeBrowserStorage()
    if (token) storage.setItem(REFRESH_KEY, token)
    else storage.removeItem(REFRESH_KEY)
  } catch {
    // storage unavailable — cookie flow remains
  }
}

export function readRefreshToken(): string | null {
  try {
    return getSafeBrowserStorage().getItem(REFRESH_KEY)
  } catch {
    return null
  }
}
