import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { DashboardPage } from './'

const listSpaces = vi.hoisted(() => vi.fn())
const listTasks = vi.hoisted(() => vi.fn())
const listPhases = vi.hoisted(() => vi.fn())
const listEvidence = vi.hoisted(() => vi.fn())
const searchWiki = vi.hoisted(() => vi.fn())

vi.mock('@/api/wiki', () => ({
  listSpaces,
  listTasks,
  listPhases,
  listEvidence,
  searchWiki,
}))

beforeEach(() => {
  vi.clearAllMocks()
})

function wrapper(children: React.ReactNode) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return (
    <QueryClientProvider client={queryClient}>
      <MemoryRouter>{children}</MemoryRouter>
    </QueryClientProvider>
  )
}

describe('DashboardPage', () => {
  it('renders the wiki overview and API-backed document actions', async () => {
    listSpaces.mockResolvedValueOnce({
      spaces: [
        {
          id: 'space-sdlc',
          key: 'SDLC',
          name: 'База знаний SDLC',
          description: 'Документы SDLC',
          owner_id: 'user-1',
          status: 'active',
          document_count: 1,
          member_count: 1,
          created_at: '2026-08-31T10:00:00Z',
          updated_at: '2026-08-31T10:00:00Z',
        },
      ],
    })
    searchWiki.mockResolvedValueOnce({
      results: [
        {
          id: 'product-requirements',
          result_type: 'document',
          title: 'Требования к Wiki',
          space_key: 'SDLC',
          url: '/documents/product-requirements',
          snippet: 'Базовый документ',
          updated_at: '2026-08-31T10:00:00Z',
        },
      ],
    })
    listTasks.mockResolvedValueOnce({
      tasks: [
        {
          space_key: 'SDLC',
          task_key: 'SDLC-42',
          title: 'Требования к Wiki',
          document_count: 1,
          evidence_count: 1,
          documents: [],
          evidence: [],
        },
      ],
    })
    listPhases.mockResolvedValueOnce({
      phases: [
        {
          space_key: 'SDLC',
          phase_key: 'implementation',
          title: 'implementation',
          document_count: 1,
          evidence_count: 1,
          documents: [],
          evidence: [],
        },
      ],
    })
    listEvidence.mockResolvedValueOnce({ evidence: [] })

    render(wrapper(<DashboardPage />))

    expect(screen.getByRole('heading', { name: 'Wiki' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: /новый документ/i })).toHaveAttribute(
      'href',
      '/documents/new',
    )
    expect(await screen.findByText('Требования к Wiki')).toBeInTheDocument()
    expect(screen.getByText('SDLC-42')).toBeInTheDocument()
  })

  it('renders overview API errors with a retry action', async () => {
    listSpaces.mockRejectedValue(new Error('Forbidden'))
    searchWiki.mockRejectedValue(new Error('Forbidden'))
    listTasks.mockRejectedValue(new Error('Forbidden'))
    listPhases.mockRejectedValue(new Error('Forbidden'))
    listEvidence.mockResolvedValue({ evidence: [] })

    render(wrapper(<DashboardPage />))

    const retryButtons = await screen.findAllByRole('button', { name: /повторить/i })
    fireEvent.click(retryButtons[0]!)

    await waitFor(() => {
      expect(listSpaces).toHaveBeenCalledTimes(2)
      expect(searchWiki).toHaveBeenCalledTimes(2)
      expect(listTasks).toHaveBeenCalledTimes(2)
      expect(listPhases).toHaveBeenCalledTimes(2)
    })
  })
})
