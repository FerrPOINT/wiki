import { create } from 'zustand'
import { createJSONStorage, persist } from 'zustand/middleware'

import { getSafeBrowserStorage } from '@/shared/lib/browser-storage'

function readStoredAuth(): {
  token: string | null
  userId: string | null
  email: string | null
  username: string | null
  displayName: string | null
} {
  try {
    const raw = getSafeBrowserStorage().getItem('wiki-auth')
    if (!raw)
      return {
        token: null,
        userId: null,
        email: null,
        username: null,
        displayName: null,
      }
    const parsed = JSON.parse(raw)
    const state = parsed.state ?? parsed
    return {
      token: state.token ?? null,
      userId: state.userId ?? state.user_id ?? null,
      email: state.email ?? null,
      username: state.username ?? null,
      displayName: state.displayName ?? state.display_name ?? null,
    }
  } catch {
    return {
      token: null,
      userId: null,
      email: null,
      username: null,
      displayName: null,
    }
  }
}

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

interface AuthState {
  token: string | null
  userId: string | null
  email: string | null
  username: string | null
  displayName: string | null
  setAuth: (payload: {
    token: string
    userId: string
    email: string
    username?: string
    displayName?: string
  }) => void
  setUser: (payload: {
    userId?: string
    email?: string
    username?: string
    displayName?: string
  }) => void
  logout: () => void
}

const initial = readStoredAuth()

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      token: initial.token,
      userId: initial.userId,
      email: initial.email,
      username: initial.username,
      displayName: initial.displayName,
      setAuth: (payload) =>
        set({
          token: payload.token,
          userId: payload.userId,
          email: payload.email,
          username: payload.username ?? null,
          displayName: payload.displayName ?? null,
        }),
      setUser: (payload) =>
        set((state) => ({
          userId: payload.userId ?? state.userId,
          email: payload.email ?? state.email,
          username: payload.username ?? state.username,
          displayName: payload.displayName ?? state.displayName,
        })),
      logout: () => {
        storeRefreshToken(null)
        set({
          token: null,
          userId: null,
          email: null,
          username: null,
          displayName: null,
        })
      },
    }),
    {
      name: 'wiki-auth',
      storage: createJSONStorage(getSafeBrowserStorage),
    },
  ),
)
