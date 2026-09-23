import { act, fireEvent, render, screen, within } from '@testing-library/react'
import { MemoryRouter, Route, Routes, useLocation, useNavigate } from 'react-router'
import { afterEach, describe, expect, it, vi } from 'vitest'

import { TaskDossierPage, TaskDossiersPage } from './'

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

const useLinkTaskDocument = vi.hoisted(() => vi.fn())
const useSpaces = vi.hoisted(() => vi.fn())
const useTask = vi.hoisted(() => vi.fn())
const useTaskSummaries = vi.hoisted(() => vi.fn())
const linkTaskMutate = vi.hoisted(() => vi.fn())
const taskRefetch = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({
  defaultSpaceKey: 'BASE',
  useLinkTaskDocument,
  useSpaces,
  useTask,
  useTaskSummaries,
}))

const taskPage = {
  space_key: 'BASE',
  task_key: 'BASE-42',
  title: 'Требования к Wiki MVP',
  document_count: 0,
  evidence_count: 0,
  documents: [],
  evidence: [],
}

function renderTaskPage(
  linkState: Record<string, unknown> = {},
  initialEntry = '/tasks/BASE-42',
  taskData: Record<string, unknown> = taskPage,
  spaceState: Record<string, unknown> = {},
) {
  useTask.mockReturnValue({
    data: taskData,
    isLoading: false,
    isError: false,
    refetch: taskRefetch,
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
  useLinkTaskDocument.mockReturnValue({
    mutate: linkTaskMutate,
    isPending: false,
    isError: false,
    error: null,
    ...linkState,
  })

  render(
    <MemoryRouter initialEntries={[initialEntry]}>
      <Routes>
        <Route path="/tasks/:taskKey" element={<TaskDossierPage />} />
      </Routes>
    </MemoryRouter>,
  )
}

function renderTaskList(
  tasks: Array<{ task_key: string; title: string; document_count: number; evidence_count: number }>,
  nextPageState: 'normal' | 'error' | 'empty' = 'normal',
  initialEntry = '/tasks?space=DOCS',
) {
  useTaskSummaries.mockImplementation(
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
          data: { tasks: [], next_cursor: null, total: tasks.length },
          isLoading: false,
          isFetching: false,
          isError: false,
          refetch: vi.fn(),
        }
      }
      const needle = params.q?.toLocaleLowerCase('ru') ?? ''
      const matching = tasks.filter((task) =>
        `${task.task_key} ${task.title}`.toLocaleLowerCase('ru').includes(needle),
      )
      const remaining = params.cursor
        ? matching.filter((task) => task.task_key > params.cursor!)
        : matching
      const page = remaining.slice(0, params.limit)
      return {
        data: {
          tasks: page,
          next_cursor: remaining.length > params.limit ? page.at(-1)?.task_key : null,
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
          path="/tasks"
          element={
            <>
              <TaskDossiersPage />
              <RouterState />
            </>
          }
        />
      </Routes>
    </MemoryRouter>,
  )
}

describe('TaskDossierPage', () => {
  afterEach(() => {
    vi.clearAllMocks()
  })

  it('links an existing document to the task dossier', () => {
    linkTaskMutate.mockImplementation((_variables, options) => options?.onSuccess?.())
    renderTaskPage()

    fireEvent.change(screen.getByLabelText('Документ для задачи'), {
      target: { value: ' product-requirements ' },
    })
    fireEvent.click(screen.getByRole('button', { name: 'Привязать' }))

    expect(linkTaskMutate).toHaveBeenCalledWith(
      {
        spaceKey: 'BASE',
        taskKey: 'BASE-42',
        body: { document_id: 'product-requirements' },
      },
      expect.objectContaining({ onSuccess: expect.any(Function) }),
    )
    expect(screen.getByText('Документ привязан к задаче')).toBeInTheDocument()
    expect(screen.getByLabelText('Документ для задачи')).toHaveValue('')
  })

  it('renders task link API errors', () => {
    renderTaskPage({
      isError: true,
      error: { code: 'FORBIDDEN', message: 'Forbidden' },
    })

    expect(screen.getByRole('alert')).toHaveTextContent('Недостаточно прав для действия')
  })

  it('uses the selected space from the route query', () => {
    renderTaskPage({}, '/tasks/BASE-42?space=DOCS')

    expect(useTask).toHaveBeenCalledWith('BASE-42', 'DOCS')

    fireEvent.change(screen.getByLabelText('Документ для задачи'), {
      target: { value: 'docs-requirements' },
    })
    fireEvent.click(screen.getByRole('button', { name: 'Привязать' }))

    expect(linkTaskMutate).toHaveBeenCalledWith(
      {
        spaceKey: 'DOCS',
        taskKey: 'BASE-42',
        body: { document_id: 'docs-requirements' },
      },
      expect.objectContaining({ onSuccess: expect.any(Function) }),
    )
  })

  it('opens exact document and evidence records in the selected space', () => {
    renderTaskPage({}, '/tasks/BASE-42?space=DOCS&q=proof&cursor=DOCS-12&keep=1', {
      ...taskPage,
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
          phase_key: 'testing',
        },
      ],
    })

    expect(screen.getByRole('link', { name: /План/ })).toHaveAttribute(
      'href',
      '/documents/doc-uuid',
    )
    expect(screen.getByRole('link', { name: /CI proof/ })).toHaveAttribute(
      'href',
      '/evidence?space=DOCS&task_key=BASE-42&id=proof-uuid',
    )
    expect(screen.getByRole('link', { name: 'К задачам' })).toHaveAttribute(
      'href',
      '/tasks?space=DOCS&q=proof&cursor=DOCS-12&keep=1',
    )
  })

  it('does not submit the link form again while saving', () => {
    renderTaskPage({ isPending: true })
    fireEvent.change(screen.getByLabelText('Документ для задачи'), { target: { value: 'plan' } })
    fireEvent.submit(screen.getByRole('form', { name: 'Привязать документ к задаче' }))
    expect(linkTaskMutate).not.toHaveBeenCalled()
    expect(screen.getByLabelText('Пространство')).toBeDisabled()
  })

  it('keeps the dossier visible and offers retry when the space catalog fails', () => {
    const refetch = vi.fn()
    renderTaskPage({}, '/tasks/BASE-42?space=DOCS', taskPage, {
      data: undefined,
      isError: true,
      error: { code: 'INTERNAL_ERROR' },
      refetch,
    })
    expect(screen.getByRole('heading', { name: 'BASE-42' })).toBeInTheDocument()
    expect(screen.getByRole('alert')).toHaveTextContent('Не удалось загрузить пространства')
    fireEvent.click(screen.getByRole('button', { name: 'Повторить' }))
    expect(refetch).toHaveBeenCalledTimes(1)
  })
})

describe('TaskDossiersPage', () => {
  afterEach(() => {
    vi.useRealTimers()
    vi.clearAllMocks()
  })

  it('searches the whole space and pages bounded summaries without a readiness score', () => {
    vi.useFakeTimers()
    renderTaskList(
      Array.from({ length: 25 }, (_, index) => ({
        task_key: `DOCS-${String(index + 1).padStart(2, '0')}`,
        title: `Задача ${index + 1}`,
        document_count: 1,
        evidence_count: 0,
      })),
    )

    expect(screen.queryByText('Заполненность')).not.toBeInTheDocument()
    expect(useTaskSummaries).toHaveBeenCalledWith('DOCS', {
      limit: 12,
      cursor: undefined,
      q: undefined,
    })
    expect(screen.getByText('Задачи: 12 из 25')).toBeInTheDocument()
    expect(screen.getAllByRole('listitem')).toHaveLength(12)
    const pagination = screen.getByRole('navigation', { name: 'Страницы задач' })
    const previousButton = within(pagination).getByRole('button', { name: 'Назад' })
    const nextButton = within(pagination).getByRole('button', { name: 'Далее' })
    expect(previousButton).toHaveClass('min-h-10', 'sm:min-h-10')
    expect(nextButton).toHaveClass('min-h-10', 'sm:min-h-10')
    fireEvent.click(nextButton)
    expect(screen.getByText('Страница 2')).toBeInTheDocument()
    expect(useTaskSummaries).toHaveBeenLastCalledWith('DOCS', {
      limit: 12,
      cursor: 'DOCS-12',
      q: undefined,
    })
    fireEvent.change(screen.getByRole('searchbox', { name: 'Найти задачу' }), {
      target: { value: 'DOCS-25' },
    })
    act(() => vi.advanceTimersByTime(300))
    expect(useTaskSummaries).toHaveBeenLastCalledWith('DOCS', {
      limit: 12,
      cursor: undefined,
      q: 'DOCS-25',
    })
    expect(screen.getByText('Задачи: 1 из 1')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: /DOCS-25/ })).toHaveAttribute(
      'href',
      '/tasks/DOCS-25?space=DOCS&q=DOCS-25',
    )
    expect(screen.queryByRole('navigation', { name: 'Страницы задач' })).not.toBeInTheDocument()

    fireEvent.change(screen.getByRole('searchbox', { name: 'Найти задачу' }), {
      target: { value: 'missing' },
    })
    act(() => vi.advanceTimersByTime(300))
    expect(screen.getByText('По запросу задачи не найдены')).toBeInTheDocument()
    expect(screen.getByRole('searchbox', { name: 'Найти задачу' })).toHaveValue('missing')
  })

  it('keeps previous-page navigation and search after a cursor request fails', () => {
    renderTaskList(
      Array.from({ length: 13 }, (_, index) => ({
        task_key: `DOCS-${String(index + 1).padStart(2, '0')}`,
        title: `Задача ${index + 1}`,
        document_count: 0,
        evidence_count: 0,
      })),
      'error',
    )

    fireEvent.click(screen.getByRole('button', { name: 'Далее' }))
    expect(screen.getByRole('alert')).toBeInTheDocument()
    expect(screen.getByRole('searchbox', { name: 'Найти задачу' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Назад' })).toBeEnabled()
    fireEvent.click(screen.getByRole('button', { name: 'Назад' }))
    expect(screen.queryByRole('alert')).not.toBeInTheDocument()
    expect(screen.getByText('Задачи: 12 из 13')).toBeInTheDocument()
  })

  it('restores search and cursor history from the URL and browser navigation', () => {
    renderTaskList(
      Array.from({ length: 37 }, (_, index) => ({
        task_key: `DOCS-${String(index + 1).padStart(2, '0')}`,
        title: `Задача ${index + 1}`,
        document_count: 1,
        evidence_count: 1,
      })),
      'normal',
      '/tasks?space=DOCS&q=DOCS&keep=1',
    )

    expect(screen.getByRole('searchbox', { name: 'Найти задачу' })).toHaveValue('DOCS')
    fireEvent.click(screen.getByRole('button', { name: 'Далее' }))
    expect(screen.getByTestId('router-location')).toHaveTextContent(
      '/tasks?space=DOCS&q=DOCS&keep=1&cursor=DOCS-12',
    )
    expect(screen.getByText('Страница 2')).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: 'История назад' }))
    expect(screen.getByTestId('router-location')).toHaveTextContent(
      '/tasks?space=DOCS&q=DOCS&keep=1',
    )
    expect(screen.queryByText('Страница 2')).not.toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'История вперёд' }))
    expect(screen.getByText('Страница 2')).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: 'Далее' }))
    expect(screen.getByTestId('router-location')).toHaveTextContent(
      '/tasks?space=DOCS&q=DOCS&keep=1&cursor=DOCS-24&previous_cursor=DOCS-12',
    )
    expect(screen.getByText('Страница 3')).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Назад' }))
    expect(screen.getByTestId('router-location')).toHaveTextContent(
      '/tasks?space=DOCS&q=DOCS&keep=1&cursor=DOCS-12',
    )
    expect(screen.getByText('Страница 2')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: /DOCS-13/ })).toHaveAttribute(
      'href',
      '/tasks/DOCS-13?space=DOCS&q=DOCS&keep=1&cursor=DOCS-12',
    )
  })

  it('keeps back navigation when a later page becomes empty', () => {
    renderTaskList(
      Array.from({ length: 13 }, (_, index) => ({
        task_key: `DOCS-${String(index + 1).padStart(2, '0')}`,
        title: `Задача ${index + 1}`,
        document_count: 0,
        evidence_count: 0,
      })),
      'empty',
    )

    fireEvent.click(screen.getByRole('button', { name: 'Далее' }))
    expect(screen.getByText('На этой странице задач больше нет')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Назад' })).toBeEnabled()
  })

  it('resets the catalog query when the selected space changes', () => {
    vi.useFakeTimers()
    renderTaskList([{ task_key: 'DOCS-01', title: 'Задача', document_count: 0, evidence_count: 0 }])
    fireEvent.change(screen.getByRole('searchbox', { name: 'Найти задачу' }), {
      target: { value: 'DOCS-01' },
    })
    act(() => vi.advanceTimersByTime(300))

    fireEvent.change(screen.getByLabelText('Пространство'), { target: { value: 'BASE' } })

    expect(screen.getByRole('searchbox', { name: 'Найти задачу' })).toHaveValue('')
    expect(useTaskSummaries).toHaveBeenLastCalledWith('BASE', {
      limit: 12,
      cursor: undefined,
      q: undefined,
    })
  })
})
