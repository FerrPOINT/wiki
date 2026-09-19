import { fireEvent, render, screen, waitFor, within } from '@testing-library/react'
import { MemoryRouter } from 'react-router'
import { afterEach, describe, expect, it, vi } from 'vitest'

import { WikiSearchPage } from './'

const useWikiSearch = vi.hoisted(() => vi.fn())
const searchRefetch = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({
  useWikiSearch,
}))

const documentResult = {
  id: 'doc-1',
  result_type: 'document',
  title: 'Требования Wiki',
  snippet: 'Базовое приложение',
  space_key: 'BASE',
  updated_at: '2026-08-31T12:00:00Z',
  url: '/documents/product-requirements',
}
const evidenceResult = {
  id: 'evidence-1',
  result_type: 'evidence',
  title: 'Smoke proof',
  snippet: 'Сборка прошла',
  space_key: 'BASE',
  updated_at: '2026-08-31T12:10:00Z',
  url: '/evidence',
}

function setupSearch(
  results = [documentResult, evidenceResult],
  overrides: Record<string, unknown> = {},
) {
  useWikiSearch.mockReturnValue({
    data: { results },
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

  it('filters visible result types and sends expanded filters to the API hook', async () => {
    setupSearch()

    expect(screen.getByRole('link', { name: /Требования Wiki/ })).toHaveAttribute(
      'href',
      '/documents/product-requirements',
    )
    expect(screen.getByRole('link', { name: /Smoke proof/ })).toHaveAttribute('href', '/evidence')

    fireEvent.click(screen.getByRole('button', { name: /Документы 1/ }))
    expect(screen.getByRole('link', { name: /Требования Wiki/ })).toBeInTheDocument()
    expect(screen.queryByRole('link', { name: /Smoke proof/ })).not.toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: 'Фильтры' }))
    fireEvent.change(screen.getByRole('searchbox', { name: 'Поисковый запрос' }), {
      target: { value: 'релиз' },
    })
    fireEvent.change(screen.getByLabelText('Пространство'), { target: { value: 'eng' } })
    fireEvent.change(screen.getByLabelText('Задача'), { target: { value: 'BASE-42' } })
    fireEvent.change(screen.getByLabelText('Фаза'), { target: { value: 'testing' } })
    fireEvent.change(screen.getByLabelText('Тип документа'), { target: { value: 'test_plan' } })
    expect(useWikiSearch).toHaveBeenLastCalledWith({
      document_type: undefined,
      limit: 100,
      phase_key: undefined,
      q: '',
      space: undefined,
      task_key: undefined,
    })
    fireEvent.click(screen.getByRole('button', { name: 'Применить' }))

    await waitFor(() =>
      expect(useWikiSearch).toHaveBeenLastCalledWith({
        document_type: 'test_plan',
        limit: 100,
        phase_key: 'testing',
        q: 'релиз',
        space: 'ENG',
        task_key: 'BASE-42',
      }),
    )
    expect(screen.getByRole('button', { name: /Фильтры \(4\)/ })).toHaveAttribute(
      'aria-expanded',
      'true',
    )
    fireEvent.click(screen.getByRole('button', { name: 'Сбросить фильтры' }))
    expect(screen.getByLabelText('Пространство')).toHaveValue('')
    expect(screen.getByLabelText('Тип документа')).toHaveValue('all')
    expect(screen.getByRole('button', { name: /Все 2/ })).toHaveAttribute('aria-pressed', 'true')
  })

  it('limits visible results by page and labels the API cap honestly', () => {
    const results = Array.from({ length: 100 }, (_, index) => ({
      ...documentResult,
      id: `doc-${index + 1}`,
      title: `Документ ${String(index + 1).padStart(3, '0')}`,
    }))
    setupSearch(results)

    expect(screen.getAllByRole('link', { name: /Документ \d+/ })).toHaveLength(12)
    expect(screen.getByText('1–12 из 100')).toBeInTheDocument()
    expect(screen.getByText(/Показаны первые 100 результатов/)).toBeInTheDocument()
    const pagination = screen.getByRole('navigation', { name: 'Страницы результатов' })
    fireEvent.click(within(pagination).getByRole('button', { name: 'Далее' }))
    expect(screen.getAllByRole('link', { name: /Документ \d+/ })).toHaveLength(12)
    expect(screen.getByText('13–24 из 100')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: /Документ 013/ })).toBeInTheDocument()
  })

  it('clears document type when the evidence view is selected', () => {
    setupSearch()
    fireEvent.click(screen.getByRole('button', { name: 'Фильтры' }))
    fireEvent.change(screen.getByLabelText('Тип документа'), { target: { value: 'requirements' } })
    fireEvent.click(screen.getByRole('button', { name: 'Применить' }))
    expect(screen.getByRole('button', { name: /Документы 1/ })).toHaveAttribute(
      'aria-pressed',
      'true',
    )
    fireEvent.click(screen.getByRole('button', { name: /Материалы 1/ }))
    expect(screen.getByLabelText('Тип документа')).toHaveValue('all')
    expect(screen.getByRole('link', { name: /Smoke proof/ })).toBeInTheDocument()
  })

  it('renders permission denied search errors with retry', () => {
    setupSearch([], {
      data: undefined,
      isError: true,
      error: { code: 'FORBIDDEN', message: 'Forbidden' },
    })

    expect(screen.getByRole('alert')).toHaveTextContent('Недостаточно прав для действия')
    fireEvent.click(screen.getByRole('button', { name: 'Повторить' }))
    expect(searchRefetch).toHaveBeenCalled()
  })
})
