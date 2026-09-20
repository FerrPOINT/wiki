import { fireEvent, render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { EvidencePage } from './'

const useCreateEvidence = vi.hoisted(() => vi.fn())
const useCreateFileEvidence = vi.hoisted(() => vi.fn())
const useAttachment = vi.hoisted(() => vi.fn())
const useDownloadAttachment = vi.hoisted(() => vi.fn())
const useEvidence = vi.hoisted(() => vi.fn())
const useEvidenceItem = vi.hoisted(() => vi.fn())
const useSpaces = vi.hoisted(() => vi.fn())
const createFileMutate = vi.hoisted(() => vi.fn())
const createLinkMutate = vi.hoisted(() => vi.fn())
const downloadAttachmentMutate = vi.hoisted(() => vi.fn())
const refetchEvidence = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({
  useAttachment,
  useCreateEvidence,
  useCreateFileEvidence,
  useDownloadAttachment,
  useEvidence,
  useEvidenceItem,
  useSpaces,
}))

function evidenceItem(id: string, overrides: Record<string, unknown> = {}) {
  return {
    attachment_id: null,
    checksum: null,
    created_at: '2026-08-31T12:10:00Z',
    created_by: 'user-editor',
    document_id: 'product-requirements',
    evidence_type: 'external_url',
    id,
    phase_key: 'implementation',
    space_key: 'BASE',
    task_key: 'BASE-42',
    title: `Материал ${id}`,
    url: `https://ci.local/jobs/${id}`,
    ...overrides,
  }
}

const defaultItems = [
  evidenceItem('link', { title: 'Сборка прошла' }),
  evidenceItem('file', {
    attachment_id: 'attachment-1',
    checksum: 'sha256:abc123',
    created_at: '2026-08-31T12:15:00Z',
    document_id: 'test-plan',
    evidence_type: 'uploaded_file',
    phase_key: 'testing',
    task_key: 'BASE-43',
    title: 'Лог сборки',
    url: null,
  }),
]

function setupEvidence(initialRoute = '/evidence', items = defaultItems) {
  useSpaces.mockReturnValue({ data: { spaces: [{ key: 'BASE' }] } })
  useAttachment.mockImplementation((attachmentId: string | null | undefined) => ({
    data:
      attachmentId === 'attachment-1'
        ? {
            checksum: 'sha256:abc123',
            content_type: 'text/plain',
            file_name: 'build.log',
            id: 'attachment-1',
            size_bytes: 2048,
            uploaded_at: '2026-08-31T12:14:00Z',
            uploaded_by: 'user-editor',
          }
        : undefined,
    isLoading: false,
    isError: false,
    error: null,
  }))
  useEvidence.mockReturnValue({
    data: { evidence: items, next_cursor: null },
    isLoading: false,
    isFetching: false,
    isError: false,
    refetch: refetchEvidence,
  })
  useEvidenceItem.mockImplementation((id: string | null | undefined) => ({
    data: items.find((item) => item.id === id) ?? defaultItems.find((item) => item.id === id),
    isLoading: false,
    isError: false,
    error: null,
    refetch: vi.fn(),
  }))
  useCreateEvidence.mockReturnValue({ mutate: createLinkMutate, isPending: false, error: null })
  useCreateFileEvidence.mockReturnValue({ mutate: createFileMutate, isPending: false, error: null })
  useDownloadAttachment.mockReturnValue({
    mutate: downloadAttachmentMutate,
    isPending: false,
    isError: false,
    error: null,
  })
  render(
    <MemoryRouter initialEntries={[initialRoute]}>
      <EvidencePage />
    </MemoryRouter>,
  )
}

function openCreateForm() {
  fireEvent.click(screen.getByRole('button', { name: 'Добавить материал' }))
}

function fillEvidenceForm() {
  fireEvent.change(screen.getByLabelText('Документ'), { target: { value: 'product-requirements' } })
  fireEvent.change(screen.getByLabelText('Название'), { target: { value: 'Проверка сборки' } })
  fireEvent.change(screen.getByLabelText('Задача'), { target: { value: 'BASE-42' } })
  fireEvent.change(screen.getByLabelText('Фаза'), { target: { value: 'implementation' } })
}

describe('EvidencePage', () => {
  afterEach(() => vi.resetAllMocks())

  it('shows a compact registry and loads file metadata only after selection', () => {
    setupEvidence()
    const table = screen.getByRole('table')
    expect(screen.getByRole('heading', { name: 'Материалы' })).toBeInTheDocument()
    expect(within(table).getByRole('link', { name: 'Сборка прошла' })).toHaveAttribute(
      'href',
      'https://ci.local/jobs/link',
    )
    expect(
      within(table).getByRole('link', { name: 'документ product-requirements' }),
    ).toHaveAttribute('href', '/documents/product-requirements')
    expect(within(table).getByRole('link', { name: 'задача BASE-42' })).toHaveAttribute(
      'href',
      '/tasks/BASE-42',
    )
    expect(within(table).getByRole('link', { name: 'фаза implementation' })).toHaveAttribute(
      'href',
      '/phases/implementation',
    )
    expect(screen.getByLabelText('Сводка текущей страницы')).toHaveTextContent(
      'Показано 2 (лимит 20)',
    )
    expect(useAttachment).not.toHaveBeenCalled()

    fireEvent.click(within(table).getByRole('button', { name: 'Открыть материал Лог сборки' }))
    expect(screen.getByRole('heading', { name: 'Выбранный материал' })).toBeInTheDocument()
    expect(screen.getByText('build.log')).toBeInTheDocument()
    expect(screen.getByText('2.0 КБ · text/plain')).toBeInTheDocument()
    expect(screen.getByText('sha256:abc123')).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Скачать Лог сборки' }))
    expect(downloadAttachmentMutate).toHaveBeenCalledWith('attachment-1', {
      onSuccess: expect.any(Function),
    })
  })

  it('applies server search only on submit and resets the cursor with filters', () => {
    setupEvidence()
    fireEvent.change(screen.getByLabelText('Поиск материалов'), { target: { value: 'лог' } })
    expect(useEvidence).toHaveBeenLastCalledWith({
      q: undefined,
      space: undefined,
      document_id: undefined,
      task_key: undefined,
      phase_key: undefined,
      cursor: undefined,
      limit: 20,
    })
    fireEvent.submit(screen.getByRole('form', { name: 'Фильтры материалов' }))
    expect(useEvidence).toHaveBeenLastCalledWith(
      expect.objectContaining({ q: 'лог', cursor: undefined, limit: 20 }),
    )
    fireEvent.click(screen.getByRole('button', { name: 'Сбросить' }))
    expect(useEvidence).toHaveBeenLastCalledWith(
      expect.objectContaining({ q: undefined, cursor: undefined }),
    )
  })

  it('does not request an invalid one-character space while it is typed', () => {
    setupEvidence()
    fireEvent.change(screen.getByLabelText('Фильтр пространства'), { target: { value: 'D' } })
    expect(useEvidence).not.toHaveBeenCalledWith(expect.objectContaining({ space: 'D' }))
    fireEvent.submit(screen.getByRole('form', { name: 'Фильтры материалов' }))
    expect(screen.getByRole('alert')).toHaveTextContent('минимум 2 символа')
    expect(useEvidence).not.toHaveBeenCalledWith(expect.objectContaining({ space: 'D' }))
    fireEvent.change(screen.getByLabelText('Фильтр пространства'), { target: { value: 'DOCS' } })
    fireEvent.submit(screen.getByRole('form', { name: 'Фильтры материалов' }))
    expect(useEvidence).toHaveBeenLastCalledWith(expect.objectContaining({ space: 'DOCS' }))
  })

  it('reads owner filters from URL and preserves non-default space in dossier links', () => {
    const items = [
      evidenceItem('docs', { space_key: 'DOCS', task_key: 'DOCS-7', phase_key: 'testing' }),
    ]
    setupEvidence('/evidence?space=DOCS&task_key=DOCS-7&phase_key=testing', items)
    expect(useEvidence).toHaveBeenLastCalledWith(
      expect.objectContaining({ space: 'DOCS', task_key: 'DOCS-7', phase_key: 'testing' }),
    )
    expect(screen.getByLabelText('Фильтр пространства')).toHaveValue('DOCS')
    const table = screen.getByRole('table')
    expect(within(table).getByRole('link', { name: 'задача DOCS-7' })).toHaveAttribute(
      'href',
      '/tasks/DOCS-7?space=DOCS',
    )
    expect(within(table).getByRole('link', { name: 'фаза testing' })).toHaveAttribute(
      'href',
      '/phases/testing?space=DOCS',
    )
    fireEvent.click(within(table).getByRole('button', { name: 'Открыть материал Материал docs' }))
    expect(screen.getByRole('heading', { name: 'Выбранный материал' })).toBeInTheDocument()
    expect(screen.getAllByRole('link', { name: 'задача DOCS-7' })[0]).toHaveAttribute(
      'href',
      '/tasks/DOCS-7?space=DOCS',
    )
  })

  it('navigates past 30 records in pages of 20', () => {
    const items = Array.from({ length: 45 }, (_, index) => evidenceItem(String(index + 1)))
    setupEvidence('/evidence', items)
    useEvidence.mockImplementation(({ cursor }: { cursor?: string }) => ({
      data: {
        evidence:
          cursor === 'page-2'
            ? items.slice(40)
            : cursor === 'page-1'
              ? items.slice(20, 40)
              : items.slice(0, 20),
        next_cursor: cursor === 'page-2' ? null : cursor === 'page-1' ? 'page-2' : 'page-1',
      },
      isLoading: false,
      isFetching: false,
      isError: false,
      refetch: refetchEvidence,
    }))
    fireEvent.change(screen.getByLabelText('Поиск материалов'), { target: { value: 'anything' } })
    fireEvent.submit(screen.getByRole('form', { name: 'Фильтры материалов' }))
    expect(screen.getByRole('table')).toHaveTextContent('Материал 20')
    fireEvent.click(screen.getByRole('button', { name: 'Далее' }))
    expect(useEvidence).toHaveBeenLastCalledWith(
      expect.objectContaining({ cursor: 'page-1', limit: 20 }),
    )
    fireEvent.click(screen.getByRole('button', { name: 'Далее' }))
    expect(useEvidence).toHaveBeenLastCalledWith(
      expect.objectContaining({ cursor: 'page-2', limit: 20 }),
    )
    expect(screen.getByRole('table')).toHaveTextContent('Материал 45')
    expect(screen.getByLabelText('Сводка текущей страницы')).toHaveTextContent(
      'Показано 5 (лимит 20)',
    )
    fireEvent.click(screen.getByRole('button', { name: 'Назад' }))
    expect(useEvidence).toHaveBeenLastCalledWith(expect.objectContaining({ cursor: 'page-1' }))
  })

  it('submits URL evidence through the shared API hook', () => {
    setupEvidence()
    openCreateForm()
    fillEvidenceForm()
    fireEvent.change(screen.getByLabelText('URL материала'), {
      target: { value: 'https://ci.local/jobs/wiki-smoke' },
    })
    fireEvent.submit(screen.getByRole('button', { name: 'Сохранить материал' }).closest('form')!)
    expect(createLinkMutate).toHaveBeenCalledWith(
      {
        document_id: 'product-requirements',
        evidence_type: 'external_url',
        phase_key: 'implementation',
        space: 'BASE',
        task_key: 'BASE-42',
        title: 'Проверка сборки',
        url: 'https://ci.local/jobs/wiki-smoke',
      },
      { onSuccess: expect.any(Function) },
    )
  })

  it('requires an owner target before creating evidence', () => {
    setupEvidence()
    openCreateForm()
    fireEvent.change(screen.getByLabelText('Название'), { target: { value: 'Без связи' } })
    fireEvent.change(screen.getByLabelText('URL материала'), {
      target: { value: 'https://ci.local/no-target' },
    })
    fireEvent.submit(screen.getByRole('button', { name: 'Сохранить материал' }).closest('form')!)
    expect(screen.getByRole('alert')).toHaveTextContent('Укажите документ, задачу или фазу')
    expect(createLinkMutate).not.toHaveBeenCalled()
  })

  it('opens a selected file from its URL even when the list page does not contain it', () => {
    setupEvidence('/evidence?id=file', [])
    expect(screen.getByRole('heading', { name: 'Выбранный материал' })).toBeInTheDocument()
    expect(screen.getByText('build.log')).toBeInTheDocument()
  })

  it('clears the native file input after success and permits choosing the same file again', async () => {
    const user = userEvent.setup()
    createFileMutate.mockImplementation((_payload, options) => options.onSuccess())
    setupEvidence()
    openCreateForm()
    await user.click(screen.getByRole('button', { name: 'Файл' }))
    fillEvidenceForm()
    const file = new File(['build ok'], 'build.log', { type: 'text/plain' })
    await user.upload(screen.getByLabelText('Файл материала'), file)
    fireEvent.submit(screen.getByLabelText('Файл материала').closest('form')!)
    expect(createFileMutate).toHaveBeenCalledTimes(1)
    expect(screen.queryByLabelText('Файл материала')).not.toBeInTheDocument()
    openCreateForm()
    expect(screen.getByLabelText('Файл материала')).toHaveValue('')
    fillEvidenceForm()
    await user.upload(screen.getByLabelText('Файл материала'), file)
    fireEvent.submit(screen.getByLabelText('Файл материала').closest('form')!)
    expect(createFileMutate).toHaveBeenCalledTimes(2)
  })

  it('keeps file selection on failed mutation for retry', async () => {
    const user = userEvent.setup()
    setupEvidence()
    openCreateForm()
    await user.click(screen.getByRole('button', { name: 'Файл' }))
    fillEvidenceForm()
    const file = new File(['build ok'], 'build.log', { type: 'text/plain' })
    await user.upload(screen.getByLabelText('Файл материала'), file)
    fireEvent.submit(screen.getByLabelText('Файл материала').closest('form')!)
    expect(screen.getByLabelText('Файл материала')).toHaveValue('C:\\fakepath\\build.log')
    expect(screen.getByLabelText('Название')).toHaveValue('Проверка сборки')
  })
})
