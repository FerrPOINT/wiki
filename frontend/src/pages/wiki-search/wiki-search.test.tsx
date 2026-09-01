import { fireEvent, render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router'
import { afterEach, describe, expect, it, vi } from 'vitest'

import { WikiSearchPage } from './'

const useWikiSearch = vi.hoisted(() => vi.fn())
const searchRefetch = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({
  defaultSpaceKey: 'SDLC',
  useWikiSearch,
}))

function setupSearch(overrides: Record<string, unknown> = {}) {
  useWikiSearch.mockReturnValue({
    data: {
      results: [
        {
          id: 'doc-1',
          result_type: 'document',
          title: 'Требования Wiki',
          snippet: 'Базовое приложение',
          space_key: 'SDLC',
          updated_at: '2026-08-31T12:00:00Z',
          url: '/documents/product-requirements',
        },
        {
          id: 'evidence-1',
          result_type: 'evidence',
          title: 'Smoke proof',
          snippet: 'Сборка прошла',
          space_key: 'SDLC',
          updated_at: '2026-08-31T12:10:00Z',
          url: '/evidence',
        },
      ],
    },
    isLoading: false,
    isError: false,
    refetch: searchRefetch,
    ...overrides,
  })

  render(
    <MemoryRouter>
      <WikiSearchPage />
    </MemoryRouter>,
  )
}

describe('WikiSearchPage', () => {
  afterEach(() => {
    vi.clearAllMocks()
  })

  it('filters visible result types and sends selected search filters to the API hook', () => {
    setupSearch()

    expect(screen.getByRole('link', { name: /Требования Wiki/ })).toHaveAttribute(
      'href',
      '/documents/product-requirements',
    )
    expect(screen.getByRole('link', { name: /Smoke proof/ })).toHaveAttribute('href', '/evidence')

    fireEvent.click(screen.getByRole('button', { name: 'Документы' }))
    expect(screen.getByRole('link', { name: /Требования Wiki/ })).toBeInTheDocument()
    expect(screen.queryByRole('link', { name: /Smoke proof/ })).not.toBeInTheDocument()

    fireEvent.change(screen.getByLabelText('Поисковый запрос'), {
      target: { value: 'релиз' },
    })
    fireEvent.change(screen.getByLabelText('Пространство поиска'), {
      target: { value: 'eng' },
    })
    fireEvent.change(screen.getByLabelText('Задача'), { target: { value: 'SDLC-42' } })
    fireEvent.change(screen.getByLabelText('Фаза'), { target: { value: 'testing' } })
    fireEvent.click(screen.getByRole('button', { name: 'План проверки' }))
    expect(useWikiSearch).toHaveBeenLastCalledWith({
      document_type: 'test_plan',
      limit: 25,
      phase_key: 'testing',
      q: 'релиз',
      space: 'ENG',
      task_key: 'SDLC-42',
    })
  })

  it('renders permission denied search errors with retry', () => {
    setupSearch({
      data: undefined,
      isError: true,
      error: { code: 'FORBIDDEN', message: 'Forbidden' },
    })

    expect(screen.getByRole('alert')).toHaveTextContent('Недостаточно прав для действия')
    fireEvent.click(screen.getByRole('button', { name: 'Повторить' }))
    expect(searchRefetch).toHaveBeenCalled()
  })
})
