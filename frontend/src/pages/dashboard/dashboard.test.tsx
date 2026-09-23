import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { fireEvent, render, screen, waitFor, within } from '@testing-library/react'
import { MemoryRouter } from 'react-router'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { DashboardPage } from './'

const listSpaces = vi.hoisted(() => vi.fn())
const listTaskSummaries = vi.hoisted(() => vi.fn())
const listPhaseSummaries = vi.hoisted(() => vi.fn())
const listEvidence = vi.hoisted(() => vi.fn())
const searchWiki = vi.hoisted(() => vi.fn())

vi.mock('@/api/wiki', () => ({
  listSpaces,
  listTaskSummaries,
  listPhaseSummaries,
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
    listTaskSummaries.mockResolvedValueOnce({
      tasks: [
        {
          space_key: 'BASE',
          task_key: 'BASE-42',
          title: 'Требования к Wiki',
          document_count: 1,
          evidence_count: 1,
        },
      ],
      next_cursor: 'BASE-42',
      total: 7,
    })
    listPhaseSummaries.mockResolvedValueOnce({
      phases: [
        {
          space_key: 'BASE',
          phase_key: 'implementation',
          title: 'implementation',
          document_count: 1,
          evidence_count: 1,
        },
      ],
      next_cursor: null,
      total: 3,
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
    const stats = within(await screen.findByRole('region', { name: 'Показатели Wiki' }))
    expect(stats.getByText('7')).toBeInTheDocument()
    expect(stats.getByText('3')).toBeInTheDocument()
    expect(listTaskSummaries).toHaveBeenCalledWith('BASE', { limit: 4 })
    expect(listPhaseSummaries).toHaveBeenCalledWith('BASE', { limit: 1 })
  })

  it('renders overview API errors with a retry action', async () => {
    listSpaces.mockRejectedValue(new Error('Forbidden'))
    searchWiki.mockRejectedValue(new Error('Forbidden'))
    listTaskSummaries.mockRejectedValue(new Error('Forbidden'))
    listPhaseSummaries.mockRejectedValue(new Error('Forbidden'))
    listEvidence.mockResolvedValue({ evidence: [] })

    render(wrapper(<DashboardPage />))

    const retryButton = await screen.findByRole('button', { name: /повторить/i })
    fireEvent.click(retryButton)

    await waitFor(() => {
      expect(listSpaces).toHaveBeenCalledTimes(2)
      expect(searchWiki).not.toHaveBeenCalled()
      expect(listTaskSummaries).not.toHaveBeenCalled()
      expect(listPhaseSummaries).not.toHaveBeenCalled()
    })
  })

  it('keeps the task list usable when document search fails', async () => {
    listSpaces.mockResolvedValue({
      spaces: [{ key: 'BASE', name: 'Base', document_count: 1 }],
    })
    searchWiki.mockRejectedValue(new Error('Search unavailable'))
    listTaskSummaries.mockResolvedValue({
      tasks: [{ task_key: 'BASE-42', title: 'Проверить релиз', document_count: 1 }],
      next_cursor: null,
      total: 1,
    })
    listPhaseSummaries.mockResolvedValue({ phases: [], next_cursor: null, total: 0 })

    render(wrapper(<DashboardPage />))

    expect(await screen.findByText('Проверить релиз')).toBeInTheDocument()
    expect(await screen.findByText('Search unavailable')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: /BASE-42/ })).toBeInTheDocument()
  })

  it('keeps a phase-count failure compact and retries only that metric', async () => {
    listSpaces.mockResolvedValue({
      spaces: [{ key: 'BASE', name: 'Base', document_count: 1 }],
    })
    searchWiki.mockResolvedValue({
      results: [
        {
          id: 'release-plan',
          result_type: 'document',
          title: 'План релиза',
          url: '/documents/release-plan',
          updated_at: '2026-09-19T10:00:00Z',
        },
      ],
    })
    listTaskSummaries.mockResolvedValue({
      tasks: [{ task_key: 'BASE-42', title: 'Проверить релиз', document_count: 1 }],
      next_cursor: null,
      total: 1,
    })
    listPhaseSummaries.mockRejectedValueOnce(new Error('Phases unavailable'))
    listPhaseSummaries.mockResolvedValue({ phases: [], next_cursor: null, total: 0 })

    render(wrapper(<DashboardPage />))

    const stats = within(await screen.findByRole('region', { name: 'Показатели Wiki' }))
    expect(await stats.findByRole('alert', { name: 'Phases unavailable' })).toHaveTextContent(
      'Не загружено',
    )
    expect(screen.getByText('План релиза')).toBeInTheDocument()
    expect(screen.getByText('Проверить релиз')).toBeInTheDocument()

    fireEvent.click(stats.getByRole('button', { name: 'Повторить загрузку: Фазы в пространстве' }))
    await waitFor(() => expect(listPhaseSummaries).toHaveBeenCalledTimes(2))
    expect(listSpaces).toHaveBeenCalledTimes(1)
    expect(searchWiki).toHaveBeenCalledTimes(1)
    expect(listTaskSummaries).toHaveBeenCalledTimes(1)
  })

  it('keeps documents usable when tasks fail and retries only the task section', async () => {
    listSpaces.mockResolvedValue({
      spaces: [{ key: 'BASE', name: 'Base', document_count: 1 }],
    })
    searchWiki.mockResolvedValue({
      results: [
        {
          id: 'release-plan',
          result_type: 'document',
          title: 'План релиза',
          url: '/documents/release-plan',
          updated_at: '2026-09-19T10:00:00Z',
        },
      ],
    })
    listTaskSummaries.mockRejectedValueOnce(new Error('Tasks unavailable'))
    listTaskSummaries.mockResolvedValue({ tasks: [], next_cursor: null, total: 0 })
    listPhaseSummaries.mockResolvedValue({ phases: [], next_cursor: null, total: 2 })

    render(wrapper(<DashboardPage />))

    expect(await screen.findByText('План релиза')).toBeInTheDocument()
    const stats = within(screen.getByRole('region', { name: 'Показатели Wiki' }))
    expect(stats.getByRole('alert', { name: 'Tasks unavailable' })).toHaveTextContent(
      'Не загружено',
    )
    const tasks = within(screen.getByRole('region', { name: 'Задачи в Wiki' }))
    expect(tasks.getByText('Tasks unavailable')).toBeInTheDocument()

    fireEvent.click(tasks.getByRole('button', { name: /повторить/i }))
    await waitFor(() => expect(listTaskSummaries).toHaveBeenCalledTimes(2))
    expect(listSpaces).toHaveBeenCalledTimes(1)
    expect(searchWiki).toHaveBeenCalledTimes(1)
    expect(listPhaseSummaries).toHaveBeenCalledTimes(1)
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
    listTaskSummaries.mockImplementation(async (space: string) => ({
      tasks:
        space === 'TEAM'
          ? [{ task_key: 'TEAM-1', title: 'Задача команды', document_count: 1 }]
          : [],
      next_cursor: null,
      total: space === 'TEAM' ? 1 : 0,
    }))
    listPhaseSummaries.mockResolvedValue({ phases: [], next_cursor: null, total: 0 })

    render(wrapper(<DashboardPage />))
    const spaceSelect = await screen.findByLabelText('Пространство')
    await waitFor(() => expect(screen.getByRole('option', { name: 'Team' })).toBeInTheDocument())
    fireEvent.change(spaceSelect, { target: { value: 'TEAM' } })

    expect(await screen.findByText('Документ команды')).toBeInTheDocument()
    expect(await screen.findByText('Задача команды')).toBeInTheDocument()
    expect(listTaskSummaries).toHaveBeenCalledWith('TEAM', { limit: 4 })
    expect(searchWiki).toHaveBeenCalledWith(expect.objectContaining({ space: 'TEAM' }))
    expect(
      within(screen.getByRole('region', { name: 'Задачи в Wiki' })).getByRole('link', {
        name: /Все/,
      }),
    ).toHaveAttribute('href', '/tasks?space=TEAM')
  })
})
