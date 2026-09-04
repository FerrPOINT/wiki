// Route guard from @sdlc/ui/auth: one silent refresh, then /login.
// Wiki keeps the access token in memory only; on a hard navigation the guard
// performs a single silent refresh (cookie/localStorage refresh token) before
// redirecting, so full page loads of protected routes stay authenticated.
import { RequireAuth as Guard } from '@sdlc/ui/auth'
import { useAuthStore } from '@/shared/auth/store'
import { refreshAccessToken } from '@/api/client'

export function RequireAuth() {
  return <Guard store={useAuthStore} refresh={refreshAccessToken} />
}
