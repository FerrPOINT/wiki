import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { SearchResultLink } from './search-result-link'
import { apiRequest, ApiError } from '@/api/client'

vi.mock('@/api/client', async (importOriginal) => ({
  ...(await importOriginal<typeof import('@/api/client')>()),
  apiRequest: vi.fn(),
}))
afterEach(() => {
  vi.unstubAllEnvs()
  vi.clearAllMocks()
})
const registry = '11111111-1111-4111-8111-111111111111'
const source = '22222222-2222-4222-8222-222222222222'
const destination = '33333333-3333-4333-8333-333333333333'
const result = {
  id: '44444444-4444-4444-8444-444444444444',
  result_type: 'document',
  title: 'Result',
  snippet: '',
  space_key: 'SECOND',
  updated_at: '2026-10-12T00:00:00Z',
  url: '/documents/44444444-4444-4444-8444-444444444444#section',
}
function mount() {
  vi.stubEnv('VITE_NAMESPACE_ENABLED', 'true')
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  render(
    <QueryClientProvider client={client}>
      <MemoryRouter
        initialEntries={[
          `/search?search_scope=all&project_scope=all&registry_instance_id=${registry}&namespace_id=${source}`,
        ]}
      >
        <SearchResultLink result={result} allSpaces>
          <span>Result</span>
        </SearchResultLink>
      </MemoryRouter>
    </QueryClientProvider>,
  )
}
describe('global search result context', () => {
  it('uses the confirmed destination instead of the selected source project', async () => {
    vi.mocked(apiRequest).mockResolvedValue({
      binding: { namespace: { registry_instance_id: registry, namespace_id: destination } },
    })
    mount()
    const link = await screen.findByRole('link', { name: 'Result' })
    const target = new URL(link.getAttribute('href')!, 'https://test.invalid')
    expect(target.searchParams.get('namespace_id')).toBe(destination)
    expect(target.searchParams.get('project_scope')).toBe('all')
    expect(target.hash).toBe('#section')
    expect(apiRequest).toHaveBeenCalledWith(
      `/api/v1/documents/${result.id}/namespace-context`,
      expect.anything(),
    )
  })
  it('clears a forbidden result instead of falling back to a neighboring project', async () => {
    vi.mocked(apiRequest).mockRejectedValue(new ApiError({ status: 403, statusText: 'Forbidden' }))
    mount()
    expect(await screen.findByRole('alert')).toHaveTextContent('недоступен')
    expect(screen.queryByRole('link')).toBeNull()
    expect(screen.queryByText('Result')).toBeNull()
  })
})
