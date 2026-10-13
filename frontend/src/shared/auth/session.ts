import { applySsoSession, restoreSso, type SsoSession } from '@sdlc/ui/sso'
import { apiBaseUrl } from '@/api/client'
import { clearLegacyRefreshToken, ssoConfig, useAuthStore } from './store'

export async function loadSsoSession(session: SsoSession, canApply = () => true): Promise<boolean> {
  const response = await fetch(`${apiBaseUrl}/api/v1/users/me`, {
    headers: { Authorization: `Bearer ${session.accessToken}` },
  })
  if (!response.ok) throw new Error(`Не удалось открыть профиль Wiki (HTTP ${response.status}).`)
  const user = (await response.json()) as {
    id: string
    email: string
    username: string
    display_name: string
  }
  if (!canApply()) return false
  applySsoSession(session, () => {
    clearLegacyRefreshToken()
    useAuthStore.getState().setAuth({
      token: session.accessToken,
      userId: user.id,
      email: user.email,
      username: user.username,
      displayName: user.display_name,
    })
  })
  return true
}

let restoring: Promise<void> | null = null
export function restoreWikiSession(): Promise<void> {
  if (!restoring) {
    const version = useAuthStore.getState().authVersion
    restoring = restoreSso(ssoConfig).then(async (session) => {
      await loadSsoSession(session, () => useAuthStore.getState().authVersion === version)
    })
    void restoring
      .finally(() => {
        restoring = null
      })
      .catch(() => undefined)
  }
  return restoring
}
