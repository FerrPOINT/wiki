import { act, renderHook, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { createElement, type ReactNode } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { apiRequest } from '@/api/client'
import { useActiveSpaces } from './active-spaces'

vi.mock('@/api/client', () => ({ apiRequest: vi.fn() }))
vi.mock('@/shared/api/hooks', () => ({
  useSpaces: () => ({
    data: {
      spaces: ['one', 'two', 'shared', 'local'].map((id) => ({
        id,
        key: id,
        name: 'Same name',
        status: 'active',
      })),
    },
    isLoading: false,
    isPending: false,
    isError: false,
    error: null,
    refetch: vi.fn(),
  }),
}))
function mount(includeArchived = false) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  const wrapper = ({ children }: { children: ReactNode }) =>
    createElement(QueryClientProvider, { client }, children)
  return renderHook(() => useActiveSpaces(includeArchived), { wrapper })
}
afterEach(() => {
  vi.unstubAllEnvs()
  vi.clearAllMocks()
})
describe('working space catalog', () => {
  it.each(['one', 'two'])(
    'filters project-owned active space %s by lifecycle, preserves shared and standalone spaces',
    async (archivedId) => {
      vi.stubEnv('VITE_NAMESPACE_ENABLED', 'true')
      let state = 'archived'
      vi.mocked(apiRequest).mockImplementation(async () => ({
        items: [
          {
            name: 'Same name',
            namespace: { registry_instance_id: 'registry', namespace_id: 'old' },
            state,
            spaces: [{ id: archivedId }, { id: 'shared' }],
          },
          {
            name: 'Same name',
            namespace: { registry_instance_id: 'registry', namespace_id: 'live' },
            state: 'active',
            spaces: [{ id: 'shared' }],
          },
        ],
        total: 2,
      }))
      const { result } = mount()
      expect(result.current.data).toBeUndefined()
      await waitFor(() => expect(result.current.data?.spaces).toHaveLength(3))
      expect(result.current.data?.spaces.map((s) => s.id)).not.toContain(archivedId)
      expect(result.current.data?.spaces.map((s) => s.id)).toEqual(
        expect.arrayContaining(['local', 'shared']),
      )
      state = 'active'
      await act(() => result.current.refetch())
      await waitFor(() => expect(result.current.data?.spaces).toHaveLength(4))
    },
  )
  it('preserves the complete management view without consulting project catalog', () => {
    vi.stubEnv('VITE_NAMESPACE_ENABLED', 'true')
    const { result } = mount(true)
    expect(result.current.data?.spaces).toHaveLength(4)
    expect(apiRequest).not.toHaveBeenCalled()
  })
  it('preserves standalone mode without a registry call', () => {
    vi.stubEnv('VITE_NAMESPACE_ENABLED', 'false')
    const { result } = mount()
    expect(result.current.data?.spaces).toHaveLength(4)
    expect(apiRequest).not.toHaveBeenCalled()
  })
  it('does not flash archived choices when the ownership catalog fails', async () => {
    vi.stubEnv('VITE_NAMESPACE_ENABLED', 'true')
    vi.mocked(apiRequest).mockRejectedValue(new Error('Catalog unavailable'))
    const { result } = mount()
    await waitFor(() => expect(result.current.isError).toBe(true))
    expect(result.current.data).toBeUndefined()
  })
})
