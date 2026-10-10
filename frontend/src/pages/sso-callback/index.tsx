import { useEffect, useState } from 'react'
import { useNamespaceNavigate as useNavigate } from '@sdlc/ui/ui'

import { completeSso } from '@sdlc/ui/sso'
import { Button } from '@sdlc/ui/ui'
import { ssoConfig, useAuthStore } from '@/shared/auth/store'
import { loadSsoSession } from '@/shared/auth/session'
import { finishPlatformLogin } from '@sdlc/ui/platform'

let pending: ReturnType<typeof completeSso> | null = null
function completion() {
  if (!pending) {
    pending = completeSso(ssoConfig)
    void pending
      .finally(() => {
        pending = null
      })
      .catch(() => undefined)
  }
  return pending
}

export function SsoCallbackPage() {
  const navigate = useNavigate()
  const [error, setError] = useState<string | null>(null)
  useEffect(() => {
    let active = true
    const version = useAuthStore.getState().authVersion
    void completion()
      .then(async (session) => {
        if (!active) return
        const applied = await loadSsoSession(session, () => active && useAuthStore.getState().authVersion === version)
        if (!active || !applied) return
        if (!finishPlatformLogin(import.meta.env.VITE_PLATFORM_ORIGIN)) navigate(session.returnTo, { replace: true })
      })
      .catch((caught) => {
        if (active) setError(caught instanceof Error ? caught.message : 'Не удалось завершить вход')
      })
    return () => {
      active = false
    }
  }, [navigate])
  return (
    <main className="grid min-h-screen place-items-center bg-background p-4">
      {error ? (
        <div className="space-y-4 text-center">
          <p role="alert">{error}</p>
          <Button onClick={() => navigate('/login', { replace: true })}>Повторить вход</Button>
        </div>
      ) : (
        <p role="status">Завершаем вход...</p>
      )}
    </main>
  )
}
