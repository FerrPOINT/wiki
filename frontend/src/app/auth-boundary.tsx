import type { ReactNode } from 'react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { PlatformQuerySession, platformQueryDefaults } from '@pdlc/ui/platform'
import { useAuthStore } from '@/shared/auth/store'
import { SpaceAdmissionBoundary } from './space-admission-boundary'

const authScope = {
  getSnapshot: () => useAuthStore.getState().authVersion,
  subscribe: useAuthStore.subscribe,
}
const createClient = () => new QueryClient({ defaultOptions: platformQueryDefaults })

export function AuthBoundary({ children }: { children: ReactNode }) {
  return (
    <PlatformQuerySession scope={authScope} createClient={createClient}>
      {(client) => (
        <QueryClientProvider client={client}>
          <SpaceAdmissionBoundary>{children}</SpaceAdmissionBoundary>
        </QueryClientProvider>
      )}
    </PlatformQuerySession>
  )
}
