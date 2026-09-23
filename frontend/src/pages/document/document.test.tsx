import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react'
import { createMemoryRouter, RouterProvider } from 'react-router'
import { afterEach, describe, expect, it, vi } from 'vitest'

import type { Document, DocumentRevision } from '@/api/wiki'

import { DocumentPage } from './'

const useArchiveDocument = vi.hoisted(() => vi.fn())
const useDocument = vi.hoisted(() => vi.fn())
const useDocumentRevision = vi.hoisted(() => vi.fn())
const useDocumentRevisions = vi.hoisted(() => vi.fn())
const useMoveDocument = vi.hoisted(() => vi.fn())
const usePublishDocument = vi.hoisted(() => vi.fn())
const useUpdateDocumentDraft = vi.hoisted(() => vi.fn())

const archiveMutate = vi.hoisted(() => vi.fn())
const archiveReset = vi.hoisted(() => vi.fn())
const moveMutate = vi.hoisted(() => vi.fn())
const publishMutateAsync = vi.hoisted(() => vi.fn())
const updateDraftMutate = vi.hoisted(() => vi.fn())
const updateDraftMutateAsync = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({
  useArchiveDocument,
  useDocument,
  useDocumentRevision,
  useDocumentRevisions,
  useMoveDocument,
  usePublishDocument,
  useUpdateDocumentDraft,
}))

const baseRevision: DocumentRevision = {
  author_id: 'user-editor',
  body_markdown: '# Published',
  body_html: '<h1>Published</h1><p>Approved body</p>',
  document_id: 'product-requirements',
  id: 'revision-2',
  published_at: '2026-08-31T12:00:00Z',
  summary: 'Утверждён MVP scope',
  title: 'Требования Wiki',
  version: 2,
}

const baseDocument: Document = {
  body_markdown: '# Published',
  body_html: '<h1>Published</h1><p>Approved body</p>',
  can_edit: true,
  created_at: '2026-08-31T10:00:00Z',
  created_by: 'user-editor',
  current_revision: baseRevision,
  document_type: 'requirements',
  draft_markdown: '# Draft\n\nUpdated body',
  evidence: [
    {
      attachment_id: null,
      checksum: null,
      created_at: '2026-08-31T12:10:00Z',
      created_by: 'user-editor',
      document_id: 'product-requirements',
      evidence_type: 'external_url',
      id: 'evidence-1',
      phase_key: 'implementation',
      space_key: 'BASE',
      task_key: 'BASE-42',
      title: 'Smoke proof',
      url: 'https://ci.local/jobs/wiki-smoke',
    },
  ],
  id: 'product-requirements',
  parent_id: 'parent-doc',
  phase_keys: ['implementation'],
  slug: 'product-requirements',
  space_key: 'BASE',
  status: 'published',
  task_keys: ['BASE-42'],
  title: 'Требования Wiki',
  updated_at: '2026-08-31T12:00:00Z',
  updated_by: 'user-editor',
}

function setupDocument(document: Document = baseDocument, revisionHistory?: DocumentRevision[]) {
  useDocument.mockReturnValue({
    data: document,
    isLoading: false,
    isError: false,
    refetch: vi.fn(),
  })
  useDocumentRevisions.mockImplementation(
    (_documentId: string, params: { limit: number; offset: number }) => ({
      data: {
        revisions: (revisionHistory ?? [baseRevision]).slice(
          params.offset,
          params.offset + params.limit,
        ),
      },
      isLoading: false,
      isError: false,
      refetch: vi.fn(),
    }),
  )
  useDocumentRevision.mockImplementation(
    (_documentId: string, revisionId: string, enabled: boolean) => ({
      data: enabled && revisionId === baseRevision.id ? baseRevision : undefined,
      isLoading: false,
      isError: false,
      refetch: vi.fn(),
    }),
  )
  useUpdateDocumentDraft.mockReturnValue({
    mutate: updateDraftMutate,
    mutateAsync: updateDraftMutateAsync,
    isPending: false,
    error: null,
  })
  usePublishDocument.mockReturnValue({
    mutateAsync: publishMutateAsync,
    isPending: false,
    error: null,
  })
  useArchiveDocument.mockReturnValue({
    mutate: archiveMutate,
    reset: archiveReset,
    isPending: false,
    error: null,
  })
  useMoveDocument.mockReturnValue({
    mutate: moveMutate,
    isPending: false,
    error: null,
  })

  const router = createMemoryRouter(
    [
      { path: '/documents/:documentId', element: <DocumentPage /> },
      { path: '*', element: <div>Destination</div> },
    ],
    { initialEntries: ['/documents/product-requirements'] },
  )
  const view = render(<RouterProvider router={router} />)
  return { ...view, router }
}

describe('DocumentPage', () => {
  afterEach(() => {
    vi.clearAllMocks()
  })

  it('opens published content first, with revisions, linked dossiers and evidence', () => {
    setupDocument()

    expect(screen.getByRole('heading', { name: 'Требования Wiki' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Просмотр' })).toHaveAttribute('aria-pressed', 'true')
    expect(screen.getByRole('button', { name: 'Правка' })).toHaveAttribute('aria-pressed', 'false')
    expect(screen.queryByLabelText('Markdown черновика')).not.toBeInTheDocument()
    expect(screen.getByRole('heading', { name: 'Published' })).toBeInTheDocument()
    expect(screen.getByText('Approved body')).toBeInTheDocument()
    expect(screen.getByText('Ревизия 2')).toBeInTheDocument()
    expect(screen.getByText('Утверждён MVP scope')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'BASE-42' })).toHaveAttribute('href', '/tasks/BASE-42')
    expect(screen.getByRole('link', { name: 'implementation' })).toHaveAttribute(
      'href',
      '/phases/implementation',
    )
    expect(screen.getByRole('link', { name: /Smoke proof/ })).toHaveAttribute('href', '/evidence')

    fireEvent.click(screen.getByRole('button', { name: 'Правка' }))
    expect(screen.getByLabelText('Название')).toHaveValue('Требования Wiki')
    expect(screen.getByLabelText('Markdown черновика')).toHaveValue('# Draft\n\nUpdated body')
    expect(screen.queryByRole('heading', { name: 'Published' })).not.toBeInTheDocument()
  })

  it('makes wide Markdown tables and code blocks keyboard focusable', async () => {
    const view = setupDocument({
      ...baseDocument,
      body_html:
        '<table><tbody><tr><td>Long table value</td></tr></tbody></table><pre><code>long-command</code></pre>',
    })

    await waitFor(() => {
      expect(screen.getByRole('table')).toHaveAttribute('tabindex', '0')
      expect(view.container.querySelector('pre')).toHaveAttribute('tabindex', '0')
    })
  })

  it('starts an unpublished document in edit mode', () => {
    setupDocument({
      ...baseDocument,
      body_html: '',
      body_markdown: '',
      current_revision: null,
      status: 'draft',
    })

    expect(screen.getByRole('button', { name: 'Правка' })).toHaveAttribute('aria-pressed', 'true')
    expect(screen.getByLabelText('Markdown черновика')).toHaveValue('# Draft\n\nUpdated body')
  })

  it('preserves unsaved draft text when switching between edit and read modes', () => {
    setupDocument()

    fireEvent.click(screen.getByRole('button', { name: 'Правка' }))
    fireEvent.change(screen.getByLabelText('Markdown черновика'), {
      target: { value: '# Unsent change' },
    })
    expect(screen.getByRole('status')).toHaveTextContent('Несохранённые изменения в черновике')

    fireEvent.click(screen.getByRole('button', { name: 'Просмотр' }))
    expect(screen.getByText('Approved body')).toBeInTheDocument()
    expect(screen.queryByLabelText('Markdown черновика')).not.toBeInTheDocument()
    expect(screen.getByRole('status')).toHaveTextContent('Несохранённые изменения в черновике')

    fireEvent.click(screen.getByRole('button', { name: 'Правка' }))
    expect(screen.getByLabelText('Markdown черновика')).toHaveValue('# Unsent change')
    expect(updateDraftMutate).not.toHaveBeenCalled()
    expect(updateDraftMutateAsync).not.toHaveBeenCalled()
    expect(publishMutateAsync).not.toHaveBeenCalled()
  })

  it('blocks navigation until the editor confirms losing an unsaved draft', async () => {
    const view = setupDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Правка' }))
    fireEvent.change(screen.getByLabelText('Название'), {
      target: { value: 'Несохранённый заголовок' },
    })

    await act(async () => view.router.navigate('/evidence'))
    expect(screen.getByRole('alertdialog')).toHaveTextContent(
      'Несохранённые изменения будут потеряны',
    )
    expect(view.router.state.location.pathname).toBe('/documents/product-requirements')

    fireEvent.click(screen.getByRole('button', { name: 'Подтвердить' }))
    await waitFor(() => expect(view.router.state.location.pathname).toBe('/evidence'))
  })

  it('keeps viewer document access read-only without exposing the draft', () => {
    setupDocument({
      ...baseDocument,
      can_edit: false,
      draft_markdown: '',
    })

    expect(screen.getByRole('heading', { name: 'Требования Wiki' })).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: 'Published' })).toBeInTheDocument()
    expect(screen.getByText('Approved body')).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: 'Режим чтения' })).toBeInTheDocument()
    expect(screen.queryByLabelText('Markdown черновика')).not.toBeInTheDocument()
    expect(screen.queryByLabelText('Родительский документ')).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Архивировать' })).not.toBeInTheDocument()
    expect(screen.queryByRole('group', { name: 'Режим документа' })).not.toBeInTheDocument()
  })

  it('opens a specific immutable revision in a dialog', async () => {
    setupDocument()

    const trigger = screen.getByRole('button', { name: 'Открыть ревизию 2' })
    fireEvent.click(trigger)

    expect(useDocumentRevision).toHaveBeenLastCalledWith('product-requirements', 'revision-2', true)
    const dialog = screen.getByRole('dialog', { name: 'Снимок ревизии' })
    expect(within(dialog).getByText('Ревизия 2: Требования Wiki')).toBeInTheDocument()
    expect(within(dialog).getByRole('heading', { name: 'Published' })).toBeInTheDocument()
    expect(within(dialog).getByText('Approved body')).toBeInTheDocument()

    fireEvent.click(within(dialog).getByRole('button', { name: 'Закрыть' }))
    await waitFor(() => expect(screen.queryByRole('dialog')).not.toBeInTheDocument())
  })

  it('pages through the complete revision history without showing the sentinel item', () => {
    const history = Array.from({ length: 45 }, (_, index) => ({
      ...baseRevision,
      id: `revision-${45 - index}`,
      version: 45 - index,
    }))
    setupDocument(baseDocument, history)

    expect(useDocumentRevisions).toHaveBeenLastCalledWith('product-requirements', {
      limit: 21,
      offset: 0,
    })
    expect(screen.getAllByRole('group', { name: /^Ревизия \d+$/ })).toHaveLength(20)
    expect(screen.getByRole('group', { name: 'Ревизия 45' })).toBeInTheDocument()
    expect(screen.queryByRole('group', { name: 'Ревизия 25' })).not.toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: 'Следующая' }))
    expect(useDocumentRevisions).toHaveBeenLastCalledWith('product-requirements', {
      limit: 21,
      offset: 20,
    })
    expect(screen.getByText('Страница 2')).toBeInTheDocument()
    expect(screen.getByRole('group', { name: 'Ревизия 25' })).toBeInTheDocument()
    expect(screen.queryByRole('group', { name: 'Ревизия 5' })).not.toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: 'Следующая' }))
    expect(useDocumentRevisions).toHaveBeenLastCalledWith('product-requirements', {
      limit: 21,
      offset: 40,
    })
    expect(screen.getAllByRole('group', { name: /^Ревизия \d+$/ })).toHaveLength(5)
    expect(screen.getByRole('button', { name: 'Следующая' })).toBeDisabled()

    fireEvent.click(screen.getByRole('button', { name: 'Предыдущая' }))
    expect(screen.getByText('Страница 2')).toBeInTheDocument()
  })

  it('sends draft and tree move mutations from visible form state', () => {
    setupDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Правка' }))

    fireEvent.change(screen.getByLabelText('Название'), {
      target: { value: 'Требования Wiki v2' },
    })
    fireEvent.change(screen.getByLabelText('Markdown черновика'), {
      target: { value: '# Draft v2' },
    })
    fireEvent.submit(screen.getByLabelText('Markdown черновика').closest('form')!)
    expect(updateDraftMutate).toHaveBeenCalledWith(
      {
        documentId: 'product-requirements',
        body: {
          title: 'Требования Wiki v2',
          content_markdown: '# Draft v2',
        },
      },
      { onSuccess: expect.any(Function) },
    )

    fireEvent.change(screen.getByLabelText('Родительский документ'), {
      target: { value: 'root-doc' },
    })
    fireEvent.click(screen.getByRole('button', { name: 'Сохранить место' }))
    expect(moveMutate).toHaveBeenCalledWith(
      {
        documentId: 'product-requirements',
        body: {
          parent_id: 'root-doc',
        },
      },
      { onSuccess: expect.any(Function) },
    )
  })

  it('keeps archive confirmation open until the server confirms success', () => {
    setupDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Правка' }))

    fireEvent.click(screen.getByRole('button', { name: 'Архивировать' }))
    expect(archiveReset).toHaveBeenCalledOnce()
    const dialog = screen.getByRole('alertdialog')
    fireEvent.click(screen.getByRole('button', { name: 'Подтвердить' }))

    expect(archiveMutate).toHaveBeenCalledWith('product-requirements', {
      onSuccess: expect.any(Function),
    })
    expect(dialog).toBeVisible()

    const onSuccess = archiveMutate.mock.calls[0]![1].onSuccess as () => void
    act(() => onSuccess())
    expect(dialog).not.toBeInTheDocument()
    expect(screen.getByText('Документ архивирован')).toBeVisible()
  })

  it('shows archive errors inside the confirmation without closing it', () => {
    setupDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Правка' }))
    fireEvent.click(screen.getByRole('button', { name: 'Архивировать' }))
    fireEvent.click(screen.getByRole('button', { name: 'Подтвердить' }))

    useArchiveDocument.mockReturnValue({
      mutate: archiveMutate,
      reset: archiveReset,
      isPending: false,
      error: { code: 'FORBIDDEN' },
    })
    fireEvent.change(screen.getByLabelText('Название'), {
      target: { value: 'Требования Wiki v2' },
    })

    expect(screen.getByRole('alertdialog')).toBeVisible()
    expect(screen.getByRole('alertdialog')).toHaveTextContent('Недостаточно прав для действия')
    expect(archiveMutate).toHaveBeenCalledOnce()
  })

  it('publishes the draft with the current base revision id', async () => {
    updateDraftMutateAsync.mockResolvedValueOnce({
      ...baseDocument,
      draft_markdown: '# Draft v2',
      title: 'Требования Wiki v2',
    })
    publishMutateAsync.mockResolvedValueOnce({
      ...baseRevision,
      id: 'revision-3',
      version: 3,
    })
    setupDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Правка' }))

    fireEvent.change(screen.getByLabelText('Название'), {
      target: { value: 'Требования Wiki v2' },
    })
    fireEvent.change(screen.getByLabelText('Markdown черновика'), {
      target: { value: '# Draft v2' },
    })
    fireEvent.change(screen.getByLabelText('Комментарий к публикации'), {
      target: { value: 'Clarified scope' },
    })
    fireEvent.click(screen.getByRole('button', { name: 'Опубликовать' }))

    await waitFor(() => {
      expect(updateDraftMutateAsync).toHaveBeenCalledWith({
        documentId: 'product-requirements',
        body: {
          title: 'Требования Wiki v2',
          content_markdown: '# Draft v2',
        },
      })
    })
    expect(publishMutateAsync).toHaveBeenCalledWith({
      documentId: 'product-requirements',
      body: {
        base_revision_id: 'revision-2',
        summary: 'Clarified scope',
      },
    })
  })

  it('explains a stale publish conflict and preserves the local draft while reviewing', async () => {
    updateDraftMutateAsync.mockResolvedValueOnce({
      ...baseDocument,
      current_revision: {
        ...baseRevision,
        id: 'revision-3',
        version: 3,
      },
      draft_markdown: '# Local draft',
    })
    publishMutateAsync.mockRejectedValueOnce({
      code: 'CONFLICT',
      message: 'document draft is based on a stale revision',
    })
    setupDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Правка' }))
    fireEvent.change(screen.getByLabelText('Markdown черновика'), {
      target: { value: '# Local draft' },
    })
    fireEvent.click(screen.getByRole('button', { name: 'Опубликовать' }))

    await waitFor(() => expect(publishMutateAsync).toHaveBeenCalledOnce())
    usePublishDocument.mockReturnValue({
      mutateAsync: publishMutateAsync,
      isPending: false,
      error: {
        code: 'CONFLICT',
        message: 'document draft is based on a stale revision',
      },
    })
    fireEvent.change(screen.getByLabelText('Комментарий к публикации'), {
      target: { value: ' ' },
    })

    expect(screen.getByRole('alert')).toHaveTextContent(
      'Другой пользователь уже опубликовал новую ревизию. Ваш черновик сохранён.',
    )
    fireEvent.click(screen.getByRole('button', { name: 'Показать актуальную версию' }))
    expect(screen.getByText('Approved body')).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Правка' }))
    expect(screen.getByLabelText('Markdown черновика')).toHaveValue('# Local draft')
  })

  it('keeps archived documents read-only in the editor and tree controls', () => {
    setupDocument({
      ...baseDocument,
      current_revision: null,
      body_html: '',
      can_edit: false,
      draft_markdown: '',
      evidence: [],
      parent_id: null,
      phase_keys: [],
      status: 'archived',
      task_keys: [],
    })

    expect(screen.getByRole('heading', { name: 'Режим чтения' })).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Архивировать' })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Сохранить' })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Опубликовать' })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Сохранить место' })).not.toBeInTheDocument()
    expect(screen.queryByLabelText('Название')).not.toBeInTheDocument()
    expect(screen.queryByLabelText('Markdown черновика')).not.toBeInTheDocument()
    expect(screen.queryByLabelText('Родительский документ')).not.toBeInTheDocument()
    expect(screen.queryByRole('group', { name: 'Режим документа' })).not.toBeInTheDocument()
    expect(screen.getByText('Документ пока не связан с задачей или фазой')).toBeInTheDocument()
    expect(screen.getByText('Материалы пока не прикреплены')).toBeInTheDocument()
  })
})
