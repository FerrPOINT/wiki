import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router'
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

function setupDocument(document: Document = baseDocument) {
  useDocument.mockReturnValue({
    data: document,
    isLoading: false,
    isError: false,
    refetch: vi.fn(),
  })
  useDocumentRevisions.mockReturnValue({
    data: { revisions: [baseRevision] },
    isLoading: false,
    isError: false,
    refetch: vi.fn(),
  })
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
    isPending: false,
    error: null,
  })
  useMoveDocument.mockReturnValue({
    mutate: moveMutate,
    isPending: false,
    error: null,
  })

  render(
    <MemoryRouter initialEntries={['/documents/product-requirements']}>
      <Routes>
        <Route path="/documents/:documentId" element={<DocumentPage />} />
      </Routes>
    </MemoryRouter>,
  )
}

describe('DocumentPage', () => {
  afterEach(() => {
    vi.clearAllMocks()
  })

  it('renders document editor, revisions, linked dossiers and evidence', () => {
    setupDocument()

    expect(screen.getByRole('heading', { name: 'Требования Wiki' })).toBeInTheDocument()
    expect(screen.getByLabelText('Название')).toHaveValue('Требования Wiki')
    expect(screen.getByLabelText('Markdown черновика')).toHaveValue('# Draft\n\nUpdated body')
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
  })

  it('opens a specific immutable revision through the revision detail hook', () => {
    setupDocument()

    fireEvent.click(screen.getByRole('button', { name: 'Открыть' }))

    expect(useDocumentRevision).toHaveBeenLastCalledWith('product-requirements', 'revision-2', true)
    expect(screen.getByRole('heading', { name: 'Снимок ревизии' })).toBeInTheDocument()
    expect(screen.getByText('Ревизия 2: Требования Wiki')).toBeInTheDocument()
    expect(screen.getAllByRole('heading', { name: 'Published' })).toHaveLength(2)
    expect(screen.getAllByText('Approved body')).toHaveLength(2)
  })

  it('sends draft and tree move mutations from visible form state', () => {
    setupDocument()

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
    expect(screen.getByText('Документ пока не связан с задачей или фазой')).toBeInTheDocument()
    expect(screen.getByText('Материалы пока не прикреплены')).toBeInTheDocument()
  })
})
