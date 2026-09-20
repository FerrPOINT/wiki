import { fireEvent, render, screen, within } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router'
import { afterEach, describe, expect, it, vi } from 'vitest'

import { TaskDossierPage, TaskDossiersPage } from './'

const useLinkTaskDocument = vi.hoisted(() => vi.fn())
const useSpaces = vi.hoisted(() => vi.fn())
const useTask = vi.hoisted(() => vi.fn())
const useTasks = vi.hoisted(() => vi.fn())
const linkTaskMutate = vi.hoisted(() => vi.fn())
const taskRefetch = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({
  defaultSpaceKey: 'BASE',
  useLinkTaskDocument,
  useSpaces,
  useTask,
  useTasks,
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
  useTasks.mockReturnValue({ data: { tasks: [] }, isLoading: false, isError: false })
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

function renderTaskList(tasks: Array<Record<string, unknown>>) {
  useTasks.mockReturnValue({ data: { tasks }, isLoading: false, isError: false })
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
    <MemoryRouter initialEntries={['/tasks?space=DOCS']}>
      <Routes>
        <Route path="/tasks" element={<TaskDossiersPage />} />
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
    renderTaskPage({}, '/tasks/BASE-42?space=DOCS', {
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
      '/tasks?space=DOCS',
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
  afterEach(() => vi.clearAllMocks())

  it('shows compact searchable pages without a fabricated readiness score', () => {
    renderTaskList(
      Array.from({ length: 25 }, (_, index) => ({
        task_key: `DOCS-${String(index + 1).padStart(2, '0')}`,
        title: `Задача ${index + 1}`,
        document_count: 1,
        evidence_count: 0,
      })),
    )

    expect(screen.queryByText('Заполненность')).not.toBeInTheDocument()
    expect(screen.getByText('Показано 12 из 25 задач')).toBeInTheDocument()
    expect(screen.getAllByRole('listitem')).toHaveLength(12)
    const pagination = screen.getByRole('navigation', { name: 'Страницы задач' })
    fireEvent.click(within(pagination).getByRole('button', { name: 'Далее' }))
    expect(screen.getByText('2 / 3')).toBeInTheDocument()
    fireEvent.change(screen.getByRole('searchbox', { name: 'Найти задачу' }), {
      target: { value: 'DOCS-25' },
    })
    expect(screen.getByText('Показано 1 из 1 задач')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: /DOCS-25/ })).toHaveAttribute(
      'href',
      '/tasks/DOCS-25?space=DOCS',
    )
    expect(screen.queryByRole('navigation', { name: 'Страницы задач' })).not.toBeInTheDocument()
  })
})
