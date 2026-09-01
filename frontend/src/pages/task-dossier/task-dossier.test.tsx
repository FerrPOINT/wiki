import { fireEvent, render, screen } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router'
import { afterEach, describe, expect, it, vi } from 'vitest'

import { TaskDossierPage } from './'

const useLinkTaskDocument = vi.hoisted(() => vi.fn())
const useSpaces = vi.hoisted(() => vi.fn())
const useTask = vi.hoisted(() => vi.fn())
const useTasks = vi.hoisted(() => vi.fn())
const linkTaskMutate = vi.hoisted(() => vi.fn())
const taskRefetch = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({
  defaultSpaceKey: 'SDLC',
  useLinkTaskDocument,
  useSpaces,
  useTask,
  useTasks,
}))

const taskPage = {
  space_key: 'SDLC',
  task_key: 'SDLC-42',
  title: 'Требования к Wiki MVP',
  document_count: 0,
  evidence_count: 0,
  documents: [],
  evidence: [],
}

function renderTaskPage(linkState: Record<string, unknown> = {}, initialEntry = '/tasks/SDLC-42') {
  useTask.mockReturnValue({
    data: taskPage,
    isLoading: false,
    isError: false,
    refetch: taskRefetch,
  })
  useTasks.mockReturnValue({ data: { tasks: [] }, isLoading: false, isError: false })
  useSpaces.mockReturnValue({
    data: {
      spaces: [
        { key: 'SDLC', name: 'SDLC' },
        { key: 'DOCS', name: 'Документы' },
      ],
    },
    isLoading: false,
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
        spaceKey: 'SDLC',
        taskKey: 'SDLC-42',
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
    renderTaskPage({}, '/tasks/SDLC-42?space=DOCS')

    expect(useTask).toHaveBeenCalledWith('SDLC-42', 'DOCS')

    fireEvent.change(screen.getByLabelText('Документ для задачи'), {
      target: { value: 'docs-requirements' },
    })
    fireEvent.click(screen.getByRole('button', { name: 'Привязать' }))

    expect(linkTaskMutate).toHaveBeenCalledWith(
      {
        spaceKey: 'DOCS',
        taskKey: 'SDLC-42',
        body: { document_id: 'docs-requirements' },
      },
      expect.objectContaining({ onSuccess: expect.any(Function) }),
    )
  })
})
