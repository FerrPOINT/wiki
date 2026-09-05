import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, renderHook, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'

import {
  useArchiveDocument,
  useCreateDocument,
  useCreateFileEvidence,
  useCreateEvidence,
  useDocumentRevisions,
  useEvidence,
  useEvidenceItem,
  useLinkPhaseDocument,
  useLinkTaskDocument,
  useMoveDocument,
  usePublishDocument,
  useUpdateDocumentDraft,
} from './hooks'

const archiveDocument = vi.hoisted(() => vi.fn())
const createDocument = vi.hoisted(() => vi.fn())
const createEvidence = vi.hoisted(() => vi.fn())
const getEvidence = vi.hoisted(() => vi.fn())
const linkPhaseDocument = vi.hoisted(() => vi.fn())
const linkTaskDocument = vi.hoisted(() => vi.fn())
const listDocumentRevisions = vi.hoisted(() => vi.fn())
const listEvidence = vi.hoisted(() => vi.fn())
const moveDocument = vi.hoisted(() => vi.fn())
const publishDocument = vi.hoisted(() => vi.fn())
const updateDocumentDraft = vi.hoisted(() => vi.fn())
const uploadAttachment = vi.hoisted(() => vi.fn())

vi.mock('@/api/auth', () => ({
  getCurrentUser: vi.fn(),
  login: vi.fn(),
  logout: vi.fn(),
  register: vi.fn(),
}))

vi.mock('@/api/wiki', () => ({
  archiveSpace: vi.fn(),
  archiveDocument,
  createSpace: vi.fn(),
  createDocument,
  createEvidence,
  createTemplate: vi.fn(),
  createUser: vi.fn(),
  deleteSpaceMember: vi.fn(),
  downloadAttachment: vi.fn(),
  getAttachment: vi.fn(),
  getDocument: vi.fn(),
  getDocumentRevision: vi.fn(),
  getEvidence,
  getPhase: vi.fn(),
  getSpaceTree: vi.fn(),
  getTask: vi.fn(),
  getWikiSettings: vi.fn(),
  linkPhaseDocument,
  linkTaskDocument,
  listSpaceMembers: vi.fn(),
  listAuditLog: vi.fn(),
  listDocumentRevisions,
  listEvidence,
  listPhases: vi.fn(),
  listSpaces: vi.fn(),
  listTasks: vi.fn(),
  listTemplates: vi.fn(),
  listUsers: vi.fn(),
  moveDocument,
  publishDocument,
  searchWiki: vi.fn(),
  updateDocumentDraft,
  updateSpace: vi.fn(),
  updateUser: vi.fn(),
  uploadAttachment,
  upsertSpaceMember: vi.fn(),
}))

function wrapper({ children }: { children: React.ReactNode }) {
  const queryClient = new QueryClient({
    defaultOptions: { mutations: { retry: false }, queries: { retry: false } },
  })
  return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
}

function wrapperFor(queryClient: QueryClient) {
  return function TestWrapper({ children }: { children: React.ReactNode }) {
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  }
}

function documentResponse(overrides: Record<string, unknown> = {}) {
  return {
    id: 'product-requirements',
    slug: 'product-requirements',
    space_key: 'BASE',
    title: 'Product requirements',
    document_type: 'requirements',
    status: 'draft',
    can_edit: true,
    parent_id: null,
    task_keys: [],
    phase_keys: [],
    evidence: [],
    created_at: '2026-08-31T12:00:00Z',
    updated_at: '2026-08-31T12:00:00Z',
    body_markdown: '# Requirements',
    body_html: '<h1>Requirements</h1>',
    draft_markdown: '# Requirements',
    current_revision: null,
    ...overrides,
  }
}

describe('wiki API hooks', () => {
  afterEach(() => {
    vi.clearAllMocks()
  })

  it('creates file evidence by claiming staged attachment without sending checksum', async () => {
    uploadAttachment.mockResolvedValueOnce({
      id: 'attachment-1',
      checksum: 'sha256-test',
    })
    createEvidence.mockResolvedValueOnce({
      id: 'evidence-1',
      space_key: 'BASE',
      evidence_type: 'uploaded_file',
      attachment_id: 'attachment-1',
      checksum: 'sha256-test',
    })

    const { result } = renderHook(() => useCreateFileEvidence(), { wrapper })

    await act(async () => {
      await result.current.mutateAsync({
        file: new File(['file body'], 'evidence.txt', { type: 'text/plain' }),
        evidence: {
          space: 'BASE',
          document_id: 'product-requirements',
          task_key: 'BASE-42',
          phase_key: 'testing',
          title: 'File evidence',
        },
      })
    })

    expect(uploadAttachment).toHaveBeenCalledWith(expect.any(File))
    expect(createEvidence).toHaveBeenCalledWith({
      space: 'BASE',
      document_id: 'product-requirements',
      task_key: 'BASE-42',
      phase_key: 'testing',
      title: 'File evidence',
      evidence_type: 'uploaded_file',
      attachment_id: 'attachment-1',
    })
  })

  it('uses bounded defaults for revision and evidence list queries', async () => {
    listDocumentRevisions.mockResolvedValueOnce({ revisions: [] })
    listEvidence.mockResolvedValueOnce({ evidence: [] })

    const revisionsHook = renderHook(() => useDocumentRevisions('product-requirements'), {
      wrapper,
    })
    const evidenceHook = renderHook(() => useEvidence(), { wrapper })

    await waitFor(() => expect(revisionsHook.result.current.isSuccess).toBe(true))
    await waitFor(() => expect(evidenceHook.result.current.isSuccess).toBe(true))

    expect(listDocumentRevisions).toHaveBeenCalledWith('product-requirements', { limit: 20 })
    expect(listEvidence).toHaveBeenCalledWith({ limit: 30 })
  })

  it('invalidates audit log after document and evidence write mutations', async () => {
    createDocument.mockResolvedValueOnce(documentResponse())
    updateDocumentDraft.mockResolvedValueOnce(documentResponse({ title: 'Updated requirements' }))
    publishDocument.mockResolvedValueOnce({
      id: 'revision-1',
      document_id: 'product-requirements',
      version: 1,
      title: 'Updated requirements',
      summary: 'Ready',
      author_id: 'admin',
      published_at: '2026-08-31T12:05:00Z',
      body_markdown: '# Requirements',
      body_html: '<h1>Requirements</h1>',
      checksum: 'sha256-test',
    })
    moveDocument.mockResolvedValueOnce(documentResponse({ parent_id: 'parent-document' }))
    archiveDocument.mockResolvedValueOnce(documentResponse({ status: 'archived' }))
    createEvidence.mockResolvedValueOnce({
      id: 'evidence-1',
      space_key: 'BASE',
      evidence_type: 'external_url',
      title: 'Smoke proof',
      url: 'https://ci.local/jobs/wiki-smoke',
      created_at: '2026-08-31T12:10:00Z',
    })
    uploadAttachment.mockResolvedValueOnce({
      id: 'attachment-1',
      checksum: 'sha256-test',
    })
    createEvidence.mockResolvedValueOnce({
      id: 'evidence-2',
      space_key: 'BASE',
      evidence_type: 'uploaded_file',
      attachment_id: 'attachment-1',
      checksum: 'sha256-test',
      created_at: '2026-08-31T12:11:00Z',
    })
    const queryClient = new QueryClient({
      defaultOptions: { mutations: { retry: false }, queries: { retry: false } },
    })
    const invalidateQueries = vi.spyOn(queryClient, 'invalidateQueries')

    const wrapper = wrapperFor(queryClient)
    const createDocumentHook = renderHook(() => useCreateDocument(), { wrapper })
    const updateDraftHook = renderHook(() => useUpdateDocumentDraft(), { wrapper })
    const publishHook = renderHook(() => usePublishDocument(), { wrapper })
    const moveHook = renderHook(() => useMoveDocument(), { wrapper })
    const archiveHook = renderHook(() => useArchiveDocument(), { wrapper })
    const createLinkHook = renderHook(() => useCreateEvidence(), { wrapper })
    const createFileHook = renderHook(() => useCreateFileEvidence(), { wrapper })

    await act(async () => {
      await createDocumentHook.result.current.mutateAsync({
        spaceKey: 'SDLC',
        body: {
          title: 'Product requirements',
          document_type: 'requirements',
          content_markdown: '# Requirements',
        },
      })
      await updateDraftHook.result.current.mutateAsync({
        documentId: 'product-requirements',
        body: { title: 'Updated requirements', content_markdown: '# Requirements' },
      })
      await publishHook.result.current.mutateAsync({
        documentId: 'product-requirements',
        body: { summary: 'Ready' },
      })
      await moveHook.result.current.mutateAsync({
        documentId: 'product-requirements',
        body: { parent_id: 'parent-document' },
      })
      await archiveHook.result.current.mutateAsync('product-requirements')
      await createLinkHook.result.current.mutateAsync({
        space: 'BASE',
        document_id: 'product-requirements',
        title: 'Smoke proof',
        evidence_type: 'external_url',
        url: 'https://ci.local/jobs/wiki-smoke',
      })
      await createFileHook.result.current.mutateAsync({
        file: new File(['file body'], 'evidence.txt', { type: 'text/plain' }),
        evidence: {
          space: 'BASE',
          document_id: 'product-requirements',
          title: 'File evidence',
        },
      })
    })

    const auditInvalidations = invalidateQueries.mock.calls.filter(
      ([call]) => call?.queryKey?.join('/') === 'wiki/audit-log',
    )
    expect(auditInvalidations).toHaveLength(7)
  })

  it('links task documents through the public API wrapper', async () => {
    linkTaskDocument.mockResolvedValueOnce({
      space_key: 'BASE',
      task_key: 'BASE-42',
      document_count: 1,
      evidence_count: 0,
      documents: [],
      evidence: [],
    })

    const { result } = renderHook(() => useLinkTaskDocument(), { wrapper })

    await act(async () => {
      await result.current.mutateAsync({
        spaceKey: 'SDLC',
        taskKey: 'SDLC-42',
        body: { document_id: 'product-requirements' },
      })
    })

    expect(linkTaskDocument).toHaveBeenCalledWith('SDLC', 'SDLC-42', {
      document_id: 'product-requirements',
    })
  })

  it('links phase documents through the public API wrapper', async () => {
    linkPhaseDocument.mockResolvedValueOnce({
      space_key: 'BASE',
      phase_key: 'implementation',
      document_count: 1,
      evidence_count: 0,
      documents: [],
      evidence: [],
    })

    const { result } = renderHook(() => useLinkPhaseDocument(), { wrapper })

    await act(async () => {
      await result.current.mutateAsync({
        spaceKey: 'SDLC',
        phaseKey: 'implementation',
        body: { document_id: 'product-requirements' },
      })
    })

    expect(linkPhaseDocument).toHaveBeenCalledWith('SDLC', 'implementation', {
      document_id: 'product-requirements',
    })
  })

  it('reads a selected evidence item through the public API wrapper', async () => {
    getEvidence.mockResolvedValueOnce({
      id: 'evidence-1',
      space_key: 'BASE',
      evidence_type: 'external_url',
      title: 'Smoke proof',
      url: 'https://ci.local/jobs/wiki-smoke',
      created_at: '2026-08-31T12:10:00Z',
    })

    const { result } = renderHook(() => useEvidenceItem('evidence-1'), { wrapper })

    await waitFor(() => expect(result.current.isSuccess).toBe(true))

    expect(getEvidence).toHaveBeenCalledWith('evidence-1')
  })
})
