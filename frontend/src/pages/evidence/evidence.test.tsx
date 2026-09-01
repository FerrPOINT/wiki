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

const createFileMutate = vi.hoisted(() => vi.fn())
const createLinkMutate = vi.hoisted(() => vi.fn())
const downloadAttachmentMutate = vi.hoisted(() => vi.fn())
const refetchEvidence = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({
  defaultSpaceKey: 'SDLC',
  useAttachment,
  useCreateEvidence,
  useCreateFileEvidence,
  useDownloadAttachment,
  useEvidence,
  useEvidenceItem,
}))

function setupEvidence(initialRoute = '/evidence') {
  const evidenceItems = [
    {
      attachment_id: null,
      checksum: null,
      created_at: '2026-08-31T12:10:00Z',
      created_by: 'user-editor',
      document_id: 'product-requirements',
      evidence_type: 'external_url',
      id: 'evidence-link',
      phase_key: 'implementation',
      space_key: 'SDLC',
      task_key: 'SDLC-42',
      title: 'Сборка прошла',
      url: 'https://ci.local/jobs/wiki-smoke',
    },
    {
      attachment_id: 'attachment-1',
      checksum: 'sha256:abc123',
      created_at: '2026-08-31T12:15:00Z',
      created_by: 'user-editor',
      document_id: 'test-plan',
      evidence_type: 'uploaded_file',
      id: 'evidence-file',
      phase_key: 'testing',
      space_key: 'SDLC',
      task_key: 'SDLC-43',
      title: 'Лог сборки',
      url: null,
    },
  ]
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
    data: {
      evidence: evidenceItems,
    },
    isLoading: false,
    isError: false,
    refetch: refetchEvidence,
  })
  useEvidenceItem.mockImplementation((evidenceId: string | null | undefined) => ({
    data: evidenceItems.find((item) => item.id === evidenceId),
    isLoading: false,
    isError: false,
    error: null,
    refetch: vi.fn(),
  }))
  useCreateEvidence.mockReturnValue({
    mutate: createLinkMutate,
    isPending: false,
    error: null,
  })
  useCreateFileEvidence.mockReturnValue({
    mutate: createFileMutate,
    isPending: false,
    error: null,
  })
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

function fillEvidenceForm() {
  fireEvent.change(screen.getByLabelText('Документ'), {
    target: { value: 'product-requirements' },
  })
  fireEvent.change(screen.getByLabelText('Название'), {
    target: { value: 'Проверка сборки' },
  })
  fireEvent.change(screen.getByLabelText('Задача'), {
    target: { value: 'SDLC-42' },
  })
  fireEvent.change(screen.getByLabelText('Фаза'), {
    target: { value: 'implementation' },
  })
}

describe('EvidencePage', () => {
  afterEach(() => {
    vi.clearAllMocks()
  })

  it('renders evidence registry with document, task and phase links', () => {
    setupEvidence()

    expect(screen.getByRole('heading', { name: 'Материалы' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'Сборка прошла' })).toHaveAttribute(
      'href',
      'https://ci.local/jobs/wiki-smoke',
    )
    expect(screen.getByRole('link', { name: 'product-requirements' })).toHaveAttribute(
      'href',
      '/documents/product-requirements',
    )
    expect(screen.getByRole('link', { name: 'SDLC-42' })).toHaveAttribute('href', '/tasks/SDLC-42')
    expect(screen.getByRole('link', { name: 'implementation' })).toHaveAttribute(
      'href',
      '/phases/implementation',
    )
    expect(screen.getByText('ссылка')).toBeInTheDocument()
    expect(screen.getByText('файл')).toBeInTheDocument()
    expect(screen.getByText('build.log')).toBeInTheDocument()
    expect(screen.getByText('2.0 КБ · text/plain')).toBeInTheDocument()
    expect(screen.getByText('sha256:abc123')).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Скачать Лог сборки' }))
    expect(downloadAttachmentMutate).toHaveBeenCalledWith('attachment-1', {
      onSuccess: expect.any(Function),
    })
  })

  it('filters the visible registry without changing API-level filters', () => {
    setupEvidence()

    fireEvent.change(screen.getByLabelText('Поиск материалов'), {
      target: { value: 'лог' },
    })

    expect(screen.queryByText('Сборка прошла')).not.toBeInTheDocument()
    expect(screen.getByText('Лог сборки')).toBeInTheDocument()
    expect(useEvidence).toHaveBeenCalledWith({
      space: 'SDLC',
      document_id: undefined,
      task_key: undefined,
      phase_key: undefined,
      limit: 30,
    })
  })

  it('opens selected evidence from URL query id', () => {
    setupEvidence('/evidence?id=evidence-file')

    expect(useEvidenceItem).toHaveBeenCalledWith('evidence-file')
    expect(screen.getByRole('heading', { name: 'Выбранный материал' })).toBeInTheDocument()
    expect(screen.getByText('документ test-plan')).toHaveAttribute('href', '/documents/test-plan')
    expect(screen.getByText('задача SDLC-43')).toHaveAttribute('href', '/tasks/SDLC-43')
    expect(screen.getByText('фаза testing')).toHaveAttribute('href', '/phases/testing')

    fireEvent.click(screen.getByRole('button', { name: 'Снять выделение' }))
    expect(screen.queryByRole('heading', { name: 'Выбранный материал' })).not.toBeInTheDocument()
  })

  it('initializes owner filters and create form from URL query params', () => {
    setupEvidence('/evidence?space=DOCS&task_key=DOCS-7&phase_key=testing')

    expect(useEvidence).toHaveBeenCalledWith({
      space: 'DOCS',
      document_id: undefined,
      task_key: 'DOCS-7',
      phase_key: 'testing',
      limit: 30,
    })
    expect(screen.getByLabelText('Пространство')).toHaveValue('DOCS')
    expect(screen.getByLabelText('Задача')).toHaveValue('DOCS-7')
    expect(screen.getByLabelText('Фаза')).toHaveValue('testing')
    expect(screen.getByLabelText('Фильтр пространства')).toHaveValue('DOCS')
    expect(screen.getByLabelText('Фильтр задачи')).toHaveValue('DOCS-7')
    expect(screen.getByLabelText('Фильтр фазы')).toHaveValue('testing')
  })

  it('submits URL evidence through the shared API hook', () => {
    setupEvidence()

    fillEvidenceForm()
    fireEvent.change(screen.getByLabelText('URL материала'), {
      target: { value: 'https://ci.local/jobs/wiki-smoke' },
    })
    fireEvent.submit(screen.getByRole('button', { name: 'Добавить материал' }).closest('form')!)
    expect(createLinkMutate).toHaveBeenCalledWith(
      {
        document_id: 'product-requirements',
        evidence_type: 'external_url',
        phase_key: 'implementation',
        space: 'SDLC',
        task_key: 'SDLC-42',
        title: 'Проверка сборки',
        url: 'https://ci.local/jobs/wiki-smoke',
      },
      { onSuccess: expect.any(Function) },
    )
  })

  it('submits file evidence through the shared API hook', async () => {
    const user = userEvent.setup()
    setupEvidence()

    await user.click(screen.getByRole('button', { name: 'Файл' }))
    fillEvidenceForm()
    const fileInput = screen.getByLabelText('Файл материала')
    const file = new File(['build ok'], 'build.log', { type: 'text/plain' })
    await user.upload(fileInput, file)
    fireEvent.submit(fileInput.closest('form')!)

    expect(createFileMutate).toHaveBeenCalledWith(
      {
        file,
        evidence: {
          document_id: 'product-requirements',
          phase_key: 'implementation',
          space: 'SDLC',
          task_key: 'SDLC-42',
          title: 'Проверка сборки',
        },
      },
      { onSuccess: expect.any(Function) },
    )
    expect(within(screen.getByRole('table')).getByText('Лог сборки')).toBeInTheDocument()
  })
})
