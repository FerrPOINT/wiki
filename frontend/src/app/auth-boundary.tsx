import { useEffect, useState, type ReactNode } from 'react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { useAuthStore } from '@/shared/auth/store'
import { SpaceAdmissionBoundary } from './space-admission-boundary'
import { setPlatformQueryClient } from '@pdlc/ui/platform'

function AuthSession({ version, children }: { version: number | undefined; children: ReactNode }) {
  const [client] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            retry: (failureCount, error) => {
              if (failureCount >= 4) return false
              const status = (error as { status?: number })?.status
              return status === undefined || status === 429 || status >= 500
            },
            retryDelay: (attempt) => Math.min(1000 * 2 ** attempt, 10_000),
          },
          mutations: { retry: false },
        },
      }),
  )
  setPlatformQueryClient(client)
  useEffect(() => {
    const clearPrevious = () => {
      if (useAuthStore.getState().authVersion !== version) client.clear()
    }
    clearPrevious()
    return useAuthStore.subscribe(clearPrevious)
  }, [client, version])
  return <QueryClientProvider client={client}><SpaceAdmissionBoundary>{children}</SpaceAdmissionBoundary></QueryClientProvider>
}

export function AuthBoundary({ children }: { children: ReactNode }) {
  const version = useAuthStore((state) => state.authVersion)
  // Remount forms as well as caches; clearing queries alone retains private drafts.
  return (
    <AuthSession key={version} version={version}>
      {children}
    </AuthSession>
  )
}
