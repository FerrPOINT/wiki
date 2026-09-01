import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, renderHook } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'

import { useCreateFileEvidence, useLinkPhaseDocument, useLinkTaskDocument } from './hooks'

const createEvidence = vi.hoisted(() => vi.fn())
const linkPhaseDocument = vi.hoisted(() => vi.fn())
const linkTaskDocument = vi.hoisted(() => vi.fn())
const uploadAttachment = vi.hoisted(() => vi.fn())

vi.mock('@/api/auth', () => ({
  getCurrentUser: vi.fn(),
  login: vi.fn(),
  logout: vi.fn(),
  register: vi.fn(),
}))

vi.mock('@/api/wiki', () => ({
  archiveSpace: vi.fn(),
  archiveDocument: vi.fn(),
  createSpace: vi.fn(),
  createDocument: vi.fn(),
  createEvidence,
  createTemplate: vi.fn(),
  createUser: vi.fn(),
  deleteSpaceMember: vi.fn(),
  downloadAttachment: vi.fn(),
  getAttachment: vi.fn(),
  getDocument: vi.fn(),
  getDocumentRevision: vi.fn(),
  getPhase: vi.fn(),
  getSpaceTree: vi.fn(),
  getTask: vi.fn(),
  getWikiSettings: vi.fn(),
  linkPhaseDocument,
  linkTaskDocument,
  listSpaceMembers: vi.fn(),
  listAuditLog: vi.fn(),
  listDocumentRevisions: vi.fn(),
  listEvidence: vi.fn(),
  listPhases: vi.fn(),
  listSpaces: vi.fn(),
  listTasks: vi.fn(),
  listTemplates: vi.fn(),
  listUsers: vi.fn(),
  moveDocument: vi.fn(),
  publishDocument: vi.fn(),
  searchWiki: vi.fn(),
  updateDocumentDraft: vi.fn(),
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
      space_key: 'SDLC',
      evidence_type: 'uploaded_file',
      attachment_id: 'attachment-1',
      checksum: 'sha256-test',
    })

    const { result } = renderHook(() => useCreateFileEvidence(), { wrapper })

    await act(async () => {
      await result.current.mutateAsync({
        file: new File(['file body'], 'evidence.txt', { type: 'text/plain' }),
        evidence: {
          space: 'SDLC',
          document_id: 'product-requirements',
          task_key: 'SDLC-42',
          phase_key: 'testing',
          title: 'File evidence',
        },
      })
    })

    expect(uploadAttachment).toHaveBeenCalledWith(expect.any(File))
    expect(createEvidence).toHaveBeenCalledWith({
      space: 'SDLC',
      document_id: 'product-requirements',
      task_key: 'SDLC-42',
      phase_key: 'testing',
      title: 'File evidence',
      evidence_type: 'uploaded_file',
      attachment_id: 'attachment-1',
    })
  })

  it('links task documents through the public API wrapper', async () => {
    linkTaskDocument.mockResolvedValueOnce({
      space_key: 'SDLC',
      task_key: 'SDLC-42',
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
      space_key: 'SDLC',
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
})
