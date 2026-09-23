import { act, fireEvent, render, screen, within } from '@testing-library/react'
import { MemoryRouter, Route, Routes, useLocation, useNavigate } from 'react-router'
import { afterEach, describe, expect, it, vi } from 'vitest'

import { PhaseDossierPage, PhaseDossiersPage } from './'

function RouterState() {
  const location = useLocation()
  const navigate = useNavigate()
  return (
    <>
      <output data-testid="router-location">{`${location.pathname}${location.search}`}</output>
      <button type="button" onClick={() => navigate(-1)}>
        История назад
      </button>
      <button type="button" onClick={() => navigate(1)}>
        История вперёд
      </button>
    </>
  )
}

const useLinkPhaseDocument = vi.hoisted(() => vi.fn())
const usePhase = vi.hoisted(() => vi.fn())
const usePhaseSummaries = vi.hoisted(() => vi.fn())
const useSpaces = vi.hoisted(() => vi.fn())
const linkPhaseMutate = vi.hoisted(() => vi.fn())
const phaseRefetch = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({
  defaultSpaceKey: 'BASE',
  useLinkPhaseDocument,
  usePhase,
  usePhaseSummaries,
  useSpaces,
}))

const phasePage = {
  space_key: 'BASE',
  phase_key: 'implementation',
  title: 'implementation',
  document_count: 0,
  evidence_count: 0,
  documents: [],
  evidence: [],
}

function renderPhasePage(
  linkState: Record<string, unknown> = {},
  initialEntry = '/phases/implementation',
  phaseData: Record<string, unknown> = phasePage,
  spaceState: Record<string, unknown> = {},
) {
  usePhase.mockReturnValue({
    data: phaseData,
    isLoading: false,
    isError: false,
    refetch: phaseRefetch,
  })
  useSpaces.mockReturnValue({
    data: {
      spaces: [
        { key: 'BASE', name: 'BASE' },
        { key: 'DOCS', name: 'Документы' },
      ],
    },
    isLoading: false,
    isError: false,
    refetch: vi.fn(),
    ...spaceState,
  })
  useLinkPhaseDocument.mockReturnValue({
    mutate: linkPhaseMutate,
    isPending: false,
    isError: false,
    error: null,
    ...linkState,
  })

  render(
    <MemoryRouter initialEntries={[initialEntry]}>
      <Routes>
        <Route path="/phases/:phaseId" element={<PhaseDossierPage />} />
      </Routes>
    </MemoryRouter>,
  )
}

function renderPhaseList(
  phases: Array<{
    phase_key: string
    title: string
    document_count: number
    evidence_count: number
  }>,
  nextPageState: 'normal' | 'error' | 'empty' = 'normal',
  initialEntry = '/phases?space=DOCS',
) {
  usePhaseSummaries.mockImplementation(
    (_spaceKey, params: { limit: number; cursor?: string; q?: string }) => {
      if (nextPageState === 'error' && params.cursor) {
        return {
          data: undefined,
          isLoading: false,
          isFetching: false,
          isError: true,
          error: new Error('network error'),
          refetch: vi.fn(),
        }
      }
      if (nextPageState === 'empty' && params.cursor) {
        return {
          data: { phases: [], next_cursor: null, total: phases.length },
          isLoading: false,
          isFetching: false,
          isError: false,
          refetch: vi.fn(),
        }
      }
      const needle = params.q?.toLocaleLowerCase('ru') ?? ''
      const matching = phases.filter((phase) =>
        `${phase.phase_key} ${phase.title}`.toLocaleLowerCase('ru').includes(needle),
      )
      const remaining = params.cursor
        ? matching.filter((phase) => phase.phase_key > params.cursor!)
        : matching
      const page = remaining.slice(0, params.limit)
      return {
        data: {
          phases: page,
          next_cursor: remaining.length > params.limit ? page.at(-1)?.phase_key : null,
          total: matching.length,
        },
        isLoading: false,
        isFetching: false,
        isError: false,
        refetch: vi.fn(),
      }
    },
  )
  useSpaces.mockReturnValue({
    data: {
      spaces: [
        { key: 'BASE', name: 'BASE' },
        { key: 'DOCS', name: 'Документы' },
      ],
    },
    isLoading: false,
  })
  render(
    <MemoryRouter initialEntries={[initialEntry]}>
      <Routes>
        <Route
          path="/phases"
          element={
            <>
              <PhaseDossiersPage />
              <RouterState />
            </>
          }
        />
      </Routes>
    </MemoryRouter>,
  )
}

describe('PhaseDossierPage', () => {
  afterEach(() => {
    vi.clearAllMocks()
  })

  it('links an existing document to the phase dossier', () => {
    linkPhaseMutate.mockImplementation((_variables, options) => options?.onSuccess?.())
    renderPhasePage()

    fireEvent.change(screen.getByLabelText('Документ для фазы'), {
      target: { value: ' product-requirements ' },
    })
    fireEvent.click(screen.getByRole('button', { name: 'Привязать' }))

    expect(linkPhaseMutate).toHaveBeenCalledWith(
      {
        spaceKey: 'BASE',
        phaseKey: 'implementation',
        body: { document_id: 'product-requirements' },
      },
      expect.objectContaining({ onSuccess: expect.any(Function) }),
    )
    expect(screen.getByText('Документ привязан к фазе')).toBeInTheDocument()
    expect(screen.getByLabelText('Документ для фазы')).toHaveValue('')
  })

  it('renders phase link API errors', () => {
    renderPhasePage({
      isError: true,
      error: { code: 'FORBIDDEN', message: 'Forbidden' },
    })

    expect(screen.getByRole('alert')).toHaveTextContent('Недостаточно прав для действия')
  })

  it('uses the selected space from the route query', () => {
    renderPhasePage({}, '/phases/implementation?space=DOCS')

    expect(usePhase).toHaveBeenCalledWith('implementation', 'DOCS')

    fireEvent.change(screen.getByLabelText('Документ для фазы'), {
      target: { value: 'docs-test-plan' },
    })
    fireEvent.click(screen.getByRole('button', { name: 'Привязать' }))

    expect(linkPhaseMutate).toHaveBeenCalledWith(
      {
        spaceKey: 'DOCS',
        phaseKey: 'implementation',
        body: { document_id: 'docs-test-plan' },
      },
      expect.objectContaining({ onSuccess: expect.any(Function) }),
    )
  })

  it('opens exact document and evidence records in the selected space', () => {
    renderPhasePage({}, '/phases/testing?space=DOCS&q=proof&cursor=phase-12&keep=1', {
      ...phasePage,
      phase_key: 'testing',
      document_count: 1,
      evidence_count: 1,
      documents: [
        {
          id: 'doc-uuid',
          slug: 'shared-slug',
          title: 'План',
          document_type: 'test_plan',
          status: 'published',
        },
      ],
      evidence: [
        {
          id: 'proof-uuid',
          title: 'CI proof',
          evidence_type: 'external_url',
          created_at: '2026-09-20T10:00:00Z',
        },
      ],
    })

    expect(screen.getByRole('link', { name: /План/ })).toHaveAttribute(
      'href',
      '/documents/doc-uuid',
    )
    expect(screen.getByRole('link', { name: /CI proof/ })).toHaveAttribute(
      'href',
      '/evidence?space=DOCS&phase_key=testing&id=proof-uuid',
    )
    expect(screen.getByRole('link', { name: 'К фазам' })).toHaveAttribute(
      'href',
      '/phases?space=DOCS&q=proof&cursor=phase-12&keep=1',
    )
    expect(screen.queryByText('Заполненность')).not.toBeInTheDocument()
  })

  it('does not submit the link form again while saving', () => {
    renderPhasePage({ isPending: true })
    fireEvent.change(screen.getByLabelText('Документ для фазы'), { target: { value: 'plan' } })
    fireEvent.submit(screen.getByRole('form', { name: 'Привязать документ к фазе' }))
    expect(linkPhaseMutate).not.toHaveBeenCalled()
    expect(screen.getByLabelText('Пространство')).toBeDisabled()
  })

  it('keeps the dossier visible and offers retry when the space catalog fails', () => {
    const refetch = vi.fn()
    renderPhasePage({}, '/phases/implementation?space=DOCS', phasePage, {
      data: undefined,
      isError: true,
      error: { code: 'INTERNAL_ERROR' },
      refetch,
    })
    expect(screen.getByRole('heading', { name: 'implementation' })).toBeInTheDocument()
    expect(screen.getByRole('alert')).toHaveTextContent('Не удалось загрузить пространства')
    fireEvent.click(screen.getByRole('button', { name: 'Повторить' }))
    expect(refetch).toHaveBeenCalledTimes(1)
  })
})

describe('PhaseDossiersPage', () => {
  afterEach(() => {
    vi.useRealTimers()
    vi.clearAllMocks()
  })

  it('searches the whole space and pages bounded summaries without completion state', () => {
    vi.useFakeTimers()
    renderPhaseList(
      Array.from({ length: 25 }, (_, index) => ({
        phase_key: `phase-${String(index + 1).padStart(2, '0')}`,
        title: `Фаза ${index + 1}`,
        document_count: 1,
        evidence_count: 1,
      })),
    )

    expect(screen.queryByText('Заполненность')).not.toBeInTheDocument()
    expect(usePhaseSummaries).toHaveBeenCalledWith('DOCS', {
      limit: 12,
      cursor: undefined,
      q: undefined,
    })
    expect(screen.getByText('Фазы: 12 из 25')).toBeInTheDocument()
    expect(screen.getAllByRole('listitem')).toHaveLength(12)
    const pagination = screen.getByRole('navigation', { name: 'Страницы фаз' })
    const previousButton = within(pagination).getByRole('button', { name: 'Назад' })
    const nextButton = within(pagination).getByRole('button', { name: 'Далее' })
    expect(previousButton).toHaveClass('min-h-10', 'sm:min-h-10')
    expect(nextButton).toHaveClass('min-h-10', 'sm:min-h-10')
    fireEvent.click(nextButton)
    expect(screen.getByText('Страница 2')).toBeInTheDocument()
    expect(usePhaseSummaries).toHaveBeenLastCalledWith('DOCS', {
      limit: 12,
      cursor: 'phase-12',
      q: undefined,
    })
    fireEvent.change(screen.getByRole('searchbox', { name: 'Найти фазу' }), {
      target: { value: 'phase-25' },
    })
    act(() => vi.advanceTimersByTime(300))
    expect(usePhaseSummaries).toHaveBeenLastCalledWith('DOCS', {
      limit: 12,
      cursor: undefined,
      q: 'phase-25',
    })
    expect(screen.getByText('Фазы: 1 из 1')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: /phase-25/ })).toHaveAttribute(
      'href',
      '/phases/phase-25?space=DOCS&q=phase-25',
    )
    expect(screen.queryByRole('navigation', { name: 'Страницы фаз' })).not.toBeInTheDocument()

    fireEvent.change(screen.getByRole('searchbox', { name: 'Найти фазу' }), {
      target: { value: 'missing' },
    })
    act(() => vi.advanceTimersByTime(300))
    expect(screen.getByText('По запросу фазы не найдены')).toBeInTheDocument()
    expect(screen.getByRole('searchbox', { name: 'Найти фазу' })).toHaveValue('missing')
  })

  it('keeps previous-page navigation after a cursor request fails', () => {
    renderPhaseList(
      Array.from({ length: 13 }, (_, index) => ({
        phase_key: `phase-${String(index + 1).padStart(2, '0')}`,
        title: `Фаза ${index + 1}`,
        document_count: 0,
        evidence_count: 0,
      })),
      'error',
    )

    fireEvent.click(screen.getByRole('button', { name: 'Далее' }))
    expect(screen.getByRole('alert')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Назад' })).toBeEnabled()
    fireEvent.click(screen.getByRole('button', { name: 'Назад' }))
    expect(screen.queryByRole('alert')).not.toBeInTheDocument()
    expect(screen.getByText('Фазы: 12 из 13')).toBeInTheDocument()
  })

  it('restores search and cursor history from the URL and browser navigation', () => {
    renderPhaseList(
      Array.from({ length: 37 }, (_, index) => ({
        phase_key: `phase-${String(index + 1).padStart(2, '0')}`,
        title: `Фаза ${index + 1}`,
        document_count: 1,
        evidence_count: 1,
      })),
      'normal',
      '/phases?space=DOCS&q=phase&keep=1',
    )

    expect(screen.getByRole('searchbox', { name: 'Найти фазу' })).toHaveValue('phase')
    fireEvent.click(screen.getByRole('button', { name: 'Далее' }))
    expect(screen.getByTestId('router-location')).toHaveTextContent(
      '/phases?space=DOCS&q=phase&keep=1&cursor=phase-12',
    )
    expect(screen.getByText('Страница 2')).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: 'История назад' }))
    expect(screen.getByTestId('router-location')).toHaveTextContent(
      '/phases?space=DOCS&q=phase&keep=1',
    )
    expect(screen.queryByText('Страница 2')).not.toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'История вперёд' }))
    expect(screen.getByText('Страница 2')).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: 'Далее' }))
    expect(screen.getByTestId('router-location')).toHaveTextContent(
      '/phases?space=DOCS&q=phase&keep=1&cursor=phase-24&previous_cursor=phase-12',
    )
    expect(screen.getByText('Страница 3')).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Назад' }))
    expect(screen.getByTestId('router-location')).toHaveTextContent(
      '/phases?space=DOCS&q=phase&keep=1&cursor=phase-12',
    )
    expect(screen.getByText('Страница 2')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: /phase-13/ })).toHaveAttribute(
      'href',
      '/phases/phase-13?space=DOCS&q=phase&keep=1&cursor=phase-12',
    )
  })

  it('keeps back navigation when a later page becomes empty', () => {
    renderPhaseList(
      Array.from({ length: 13 }, (_, index) => ({
        phase_key: `phase-${String(index + 1).padStart(2, '0')}`,
        title: `Фаза ${index + 1}`,
        document_count: 0,
        evidence_count: 0,
      })),
      'empty',
    )

    fireEvent.click(screen.getByRole('button', { name: 'Далее' }))
    expect(screen.getByText('На этой странице фаз больше нет')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Назад' })).toBeEnabled()
  })
})
