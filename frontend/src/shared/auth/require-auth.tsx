// Route guard from @sdlc/ui/auth. Wiki redirects immediately; its API client
// owns the 401 refresh flow.
import { RequireAuth as Guard } from '@sdlc/ui/auth'
import { useAuthStore } from '@/shared/auth/store'

export function RequireAuth() {
  return <Guard store={useAuthStore} />
}
