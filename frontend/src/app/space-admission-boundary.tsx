import { useCallback, type ReactNode } from 'react'
import { AccessFreshnessBoundary } from '@pdlc/ui/ui'
import { apiRequest } from '@/api/client'
import { useAuthStore } from '@/shared/auth/store'

const load = (signal: AbortSignal) =>
  apiRequest<unknown>('/api/v1/space-admission', { signal, cache: 'no-store' })
export function SpaceAdmissionBoundary({ children }: { children: ReactNode }) {
  const token = useAuthStore((state) => state.token)
  const version = useAuthStore((state) => state.authVersion ?? 0)
  const retire = useCallback(() => {
    useAuthStore.setState((state) =>
      (state.authVersion ?? 0) === version ? { authVersion: version + 1 } : {},
    )
  }, [version])
  return (
    <AccessFreshnessBoundary enabled={!!token} authorityKey={version} load={load} onRetire={retire}>
      {children}
    </AccessFreshnessBoundary>
  )
}
