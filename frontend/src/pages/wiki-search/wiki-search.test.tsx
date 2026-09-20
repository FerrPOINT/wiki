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
  useWikiSearch.mockImplementation((params: { result_type?: string; cursor?: string }) => {
    const matching = results.filter(
      (result) => !params.result_type || result.result_type === params.result_type,
    )
    const offset = params.cursor ? Number(params.cursor.slice(5)) : 0
    const page = matching.slice(offset, offset + 20)
    return {
      data: {
        results: page,
        next_cursor: offset + 20 < matching.length ? `page-${offset + 20}` : null,
      },
      isLoading: false,
      isError: false,
      refetch: searchRefetch,
      ...overrides,
    }
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

  it('sends result type and expanded filters to the API hook', async () => {
    setupSearch()

    expect(screen.getByRole('link', { name: /Требования Wiki/ })).toHaveAttribute(
      'href',
      '/documents/product-requirements',
    )
    expect(screen.getByRole('link', { name: /Smoke proof/ })).toHaveAttribute('href', '/evidence')

    fireEvent.click(screen.getByRole('button', { name: 'Документы' }))
    expect(screen.getByRole('link', { name: /Требования Wiki/ })).toBeInTheDocument()
    expect(screen.queryByRole('link', { name: /Smoke proof/ })).not.toBeInTheDocument()
    expect(useWikiSearch).toHaveBeenLastCalledWith(
      expect.objectContaining({ result_type: 'document' }),
    )

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
      result_type: 'document',
      cursor: undefined,
      limit: 20,
      phase_key: undefined,
      q: '',
      space: undefined,
      task_key: undefined,
    })
    fireEvent.click(screen.getByRole('button', { name: 'Применить' }))

    await waitFor(() =>
      expect(useWikiSearch).toHaveBeenLastCalledWith({
        document_type: 'test_plan',
        result_type: 'document',
        cursor: undefined,
        limit: 20,
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
    expect(screen.getByRole('button', { name: 'Все' })).toHaveAttribute('aria-pressed', 'true')
  })

  it('requests subsequent pages and displays only the current page range', () => {
    const results = Array.from({ length: 41 }, (_, index) => ({
      ...documentResult,
      id: `doc-${index + 1}`,
      title: `Документ ${String(index + 1).padStart(3, '0')}`,
    }))
    setupSearch(results)

    expect(screen.getAllByRole('link', { name: /Документ \d+/ })).toHaveLength(20)
    expect(screen.getByText('Показано 1–20')).toBeInTheDocument()
    const pagination = screen.getByRole('navigation', { name: 'Страницы результатов' })
    fireEvent.click(within(pagination).getByRole('button', { name: 'Далее' }))
    expect(useWikiSearch).toHaveBeenLastCalledWith(expect.objectContaining({ cursor: 'page-20' }))
    expect(screen.getAllByRole('link', { name: /Документ \d+/ })).toHaveLength(20)
    expect(screen.getByText('Показано 21–40')).toBeInTheDocument()
    fireEvent.click(within(pagination).getByRole('button', { name: 'Далее' }))
    expect(screen.getByText('Показано 41–41')).toBeInTheDocument()
    expect(within(pagination).getByRole('button', { name: 'Далее' })).toBeDisabled()
    fireEvent.click(within(pagination).getByRole('button', { name: 'Назад' }))
    expect(screen.getByText('Показано 21–40')).toBeInTheDocument()
  })

  it('finds materials beyond the first hundred documents and clears document type', () => {
    const documents = Array.from({ length: 101 }, (_, index) => ({
      ...documentResult,
      id: `doc-${index}`,
    }))
    setupSearch([...documents, evidenceResult])
    fireEvent.click(screen.getByRole('button', { name: 'Фильтры' }))
    fireEvent.change(screen.getByLabelText('Тип документа'), { target: { value: 'requirements' } })
    fireEvent.click(screen.getByRole('button', { name: 'Применить' }))
    expect(screen.getByRole('button', { name: 'Документы' })).toHaveAttribute(
      'aria-pressed',
      'true',
    )
    fireEvent.click(screen.getByRole('button', { name: 'Материалы' }))
    expect(screen.getByLabelText('Тип документа')).toHaveValue('all')
    expect(screen.getByRole('link', { name: /Smoke proof/ })).toBeInTheDocument()
    expect(useWikiSearch).toHaveBeenLastCalledWith(
      expect.objectContaining({
        result_type: 'evidence',
        document_type: undefined,
        cursor: undefined,
      }),
    )
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
