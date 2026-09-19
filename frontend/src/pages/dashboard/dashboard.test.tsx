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
          key: 'BASE',
          name: 'База знаний Base',
          description: 'Документы платформы Base',
          owner_id: 'user-1',
          status: 'active',
          document_count: 1,
          member_count: 1,
          created_at: '2026-08-31T10:00:00Z',
          updated_at: '2026-08-31T10:00:00Z',
        },
      ],
    })
    searchWiki.mockResolvedValue({
      results: [
        {
          id: 'product-requirements',
          result_type: 'document',
          title: 'Требования к Wiki',
          space_key: 'BASE',
          url: '/documents/product-requirements',
          snippet: 'Базовый документ',
          updated_at: '2026-08-31T10:00:00Z',
        },
      ],
    })
    listTasks.mockResolvedValueOnce({
      tasks: [
        {
          space_key: 'BASE',
          task_key: 'BASE-42',
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
          space_key: 'BASE',
          phase_key: 'implementation',
          title: 'implementation',
          document_count: 1,
          evidence_count: 1,
          documents: [],
          evidence: [],
        },
      ],
    })
    listEvidence.mockResolvedValue({ evidence: [] })

    render(wrapper(<DashboardPage />))

    expect(screen.getByRole('heading', { name: 'Wiki' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: /новый документ/i })).toHaveAttribute(
      'href',
      '/documents/new',
    )
    expect(await screen.findAllByText('Требования к Wiki')).toHaveLength(2)
    expect(screen.getByRole('link', { name: /BASE-42/ })).toHaveAttribute(
      'href',
      '/tasks/BASE-42?space=BASE',
    )
  })

  it('renders overview API errors with a retry action', async () => {
    listSpaces.mockRejectedValue(new Error('Forbidden'))
    searchWiki.mockRejectedValue(new Error('Forbidden'))
    listTasks.mockRejectedValue(new Error('Forbidden'))
    listPhases.mockRejectedValue(new Error('Forbidden'))
    listEvidence.mockResolvedValue({ evidence: [] })

    render(wrapper(<DashboardPage />))

    const retryButton = await screen.findByRole('button', { name: /повторить/i })
    fireEvent.click(retryButton)

    await waitFor(() => {
      expect(listSpaces).toHaveBeenCalledTimes(2)
      expect(searchWiki).not.toHaveBeenCalled()
      expect(listTasks).not.toHaveBeenCalled()
      expect(listPhases).not.toHaveBeenCalled()
    })
  })

  it('keeps the task list usable when document search fails', async () => {
    listSpaces.mockResolvedValue({
      spaces: [{ key: 'BASE', name: 'Base', document_count: 1 }],
    })
    searchWiki.mockRejectedValue(new Error('Search unavailable'))
    listTasks.mockResolvedValue({
      tasks: [{ task_key: 'BASE-42', title: 'Проверить релиз', document_count: 1 }],
    })
    listPhases.mockResolvedValue({ phases: [] })

    render(wrapper(<DashboardPage />))

    expect(await screen.findByText('Проверить релиз')).toBeInTheDocument()
    expect(await screen.findByText('Search unavailable')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: /BASE-42/ })).toBeInTheDocument()
  })

  it('shows a separate retry for the phase count', async () => {
    listSpaces.mockResolvedValue({
      spaces: [{ key: 'BASE', name: 'Base', document_count: 1 }],
    })
    searchWiki.mockResolvedValue({ results: [] })
    listTasks.mockResolvedValue({ tasks: [] })
    listPhases.mockRejectedValueOnce(new Error('Phases unavailable'))
    listPhases.mockResolvedValue({ phases: [] })

    render(wrapper(<DashboardPage />))

    expect(await screen.findByText('Phases unavailable')).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: /повторить/i }))
    await waitFor(() => expect(listPhases).toHaveBeenCalledTimes(2))
    expect(listSpaces).toHaveBeenCalledTimes(1)
    expect(searchWiki).toHaveBeenCalledTimes(1)
    expect(listTasks).toHaveBeenCalledTimes(1)
  })

  it('switches the scoped overview between spaces', async () => {
    listSpaces.mockResolvedValue({
      spaces: [
        { key: 'BASE', name: 'Base', document_count: 1 },
        { key: 'TEAM', name: 'Team', document_count: 2 },
      ],
    })
    searchWiki.mockImplementation(async ({ space }: { space?: string }) => ({
      results:
        space === 'TEAM'
          ? [
              {
                id: 'team-doc',
                result_type: 'document',
                title: 'Документ команды',
                url: '/documents/team-doc',
                updated_at: '2026-09-19T10:00:00Z',
              },
            ]
          : [],
    }))
    listTasks.mockImplementation(async (space: string) => ({
      tasks:
        space === 'TEAM'
          ? [{ task_key: 'TEAM-1', title: 'Задача команды', document_count: 1 }]
          : [],
    }))
    listPhases.mockResolvedValue({ phases: [] })

    render(wrapper(<DashboardPage />))
    const spaceSelect = await screen.findByLabelText('Пространство')
    await waitFor(() => expect(screen.getByRole('option', { name: 'Team' })).toBeInTheDocument())
    fireEvent.change(spaceSelect, { target: { value: 'TEAM' } })

    expect(await screen.findByText('Документ команды')).toBeInTheDocument()
    expect(await screen.findByText('Задача команды')).toBeInTheDocument()
    expect(listTasks).toHaveBeenCalledWith('TEAM')
    expect(searchWiki).toHaveBeenCalledWith(expect.objectContaining({ space: 'TEAM' }))
  })
})
