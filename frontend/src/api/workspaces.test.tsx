import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, cleanup, renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { afterEach, expect, it, vi } from 'vitest'
import { apiRequest } from './client'
import { archiveSpace } from './wiki'
import { useArchiveSpace } from '@/shared/api/hooks'
import { useSpaceProjects } from './workspaces'

vi.mock('./client', async (original) => ({
  ...(await original<typeof import('./client')>()),
  apiRequest: vi.fn(),
}))
vi.mock('./wiki', async (original) => ({
  ...(await original<typeof import('./wiki')>()),
  archiveSpace: vi.fn(),
}))
afterEach(() => {
  cleanup()
  vi.resetAllMocks()
})
it('refreshes the own-space state after archive without requiring a page reload', async () => {
  let status = 'active'
  vi.mocked(apiRequest).mockImplementation(async () => ({
    space: { id: 'space-id', key: 'SHARED', name: 'Shared', status },
    projects: [],
  }))
  vi.mocked(archiveSpace).mockImplementation(async () => {
    status = 'archived'
    return {
      id: 'space-id',
      key: 'SHARED',
      name: 'Shared',
      description: '',
      status,
      owner_id: 'owner-id',
      created_at: '',
      updated_at: '',
      document_count: 1,
      member_count: 1,
    }
  })
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  const { result } = renderHook(
    () => ({ projects: useSpaceProjects('SHARED'), archive: useArchiveSpace() }),
    {
      wrapper: ({ children }: { children: ReactNode }) => (
        <QueryClientProvider client={client}>{children}</QueryClientProvider>
      ),
    },
  )
  await waitFor(() => expect(result.current.projects.data?.space.status).toBe('active'))
  await act(async () => {
    await result.current.archive.mutateAsync('SHARED')
  })
  await waitFor(() => expect(result.current.projects.data?.space.status).toBe('archived'))
})
