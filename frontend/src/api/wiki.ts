import { apiBlobRequest, apiRequest, type ApiBlobResponse } from './client'
import type {
  AttachmentResponse,
  AuditEntryResponse,
  AuditLogResponse,
  CreateTemplateRequest as GeneratedCreateTemplateRequest,
  CreateDocumentRequest as GeneratedCreateDocumentRequest,
  CreateEvidenceRequest as GeneratedCreateEvidenceRequest,
  CreateSpaceRequest as GeneratedCreateSpaceRequest,
  DocumentResponse,
  DocumentRevisionListResponse,
  DocumentRevisionResponse,
  DocumentSummaryResponse,
  EvidenceListResponse,
  EvidenceResponse,
  LinkDocumentRequest as GeneratedLinkDocumentRequest,
  MoveDocumentRequest as GeneratedMoveDocumentRequest,
  PhasePageListResponse,
  PhasePageResponse,
  PublishDocumentRequest as GeneratedPublishDocumentRequest,
  SearchResponse,
  SearchResultResponse,
  SpaceListResponse,
  SpaceMemberListResponse,
  SpaceMemberResponse,
  SpaceResponse,
  SpaceTreeNodeResponse,
  SpaceTreeResponse,
  TaskPageListResponse,
  TaskPageResponse,
  TemplateListResponse,
  TemplateResponse,
  UpdateDocumentDraftRequest as GeneratedUpdateDocumentDraftRequest,
  UpdateSpaceRequest as GeneratedUpdateSpaceRequest,
  UpsertSpaceMemberRequest as GeneratedUpsertSpaceMemberRequest,
  WikiCreateUserRequest,
  WikiSettingsResponse,
  WikiUpdateUserRequest,
  WikiUserListResponse,
  WikiUserResponse,
} from './generated-exports'

export type Space = SpaceResponse
export type SpaceTreeNode = SpaceTreeNodeResponse
export type DocumentRevision = DocumentRevisionResponse
export type Evidence = EvidenceResponse
export type Attachment = AttachmentResponse
export type AttachmentDownload = ApiBlobResponse
export type DocumentSummary = DocumentSummaryResponse
export type Document = DocumentResponse
export type CreateSpaceRequest = GeneratedCreateSpaceRequest
export type UpdateSpaceRequest = GeneratedUpdateSpaceRequest
export type SpaceMember = SpaceMemberResponse
export type UpsertSpaceMemberRequest = GeneratedUpsertSpaceMemberRequest
export type CreateDocumentRequest = GeneratedCreateDocumentRequest
export type UpdateDocumentDraftRequest = GeneratedUpdateDocumentDraftRequest
export type PublishDocumentRequest = GeneratedPublishDocumentRequest
export type MoveDocumentRequest = GeneratedMoveDocumentRequest
export type LinkDocumentRequest = GeneratedLinkDocumentRequest
export type TaskPage = TaskPageResponse
export type PhasePage = PhasePageResponse
export type Template = TemplateResponse
export type CreateTemplateRequest = GeneratedCreateTemplateRequest
export type AuditEntry = AuditEntryResponse
export type User = WikiUserResponse
export type CreateUserRequest = WikiCreateUserRequest
export type UpdateUserRequest = WikiUpdateUserRequest
export type WikiSettings = WikiSettingsResponse
export type SearchResult = SearchResultResponse
export type CreateEvidenceRequest = GeneratedCreateEvidenceRequest

export type SearchParams = {
  q?: string
  space?: string
  task_key?: string
  phase_key?: string
  document_type?: string
  include_archived?: boolean
  limit?: number
}

export type EvidenceListParams = {
  space?: string
  document_id?: string
  task_key?: string
  phase_key?: string
  limit?: number
}

export type DocumentRevisionListParams = {
  limit?: number
}

export type AuditLogParams = {
  limit?: number
}

function queryString(params: Record<string, string | number | boolean | null | undefined>): string {
  const query = new URLSearchParams()
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== null && value !== '') query.set(key, String(value))
  }
  const serialized = query.toString()
  return serialized ? `?${serialized}` : ''
}

export function listSpaces(): Promise<SpaceListResponse> {
  return apiRequest<SpaceListResponse>('/api/v1/spaces')
}

export function getSpace(spaceKey: string): Promise<SpaceResponse> {
  return apiRequest<SpaceResponse>(`/api/v1/spaces/${encodeURIComponent(spaceKey)}`)
}

export function createSpace(body: CreateSpaceRequest): Promise<SpaceResponse> {
  return apiRequest<SpaceResponse>('/api/v1/spaces', { method: 'POST', body })
}

export function updateSpace(spaceKey: string, body: UpdateSpaceRequest): Promise<SpaceResponse> {
  return apiRequest<SpaceResponse>(`/api/v1/spaces/${encodeURIComponent(spaceKey)}`, {
    method: 'PUT',
    body,
  })
}

export function archiveSpace(spaceKey: string): Promise<SpaceResponse> {
  return apiRequest<SpaceResponse>(`/api/v1/spaces/${encodeURIComponent(spaceKey)}/archive`, {
    method: 'POST',
  })
}

export function getSpaceTree(spaceKey: string): Promise<SpaceTreeResponse> {
  return apiRequest<SpaceTreeResponse>(`/api/v1/spaces/${encodeURIComponent(spaceKey)}/tree`)
}

export function listSpaceMembers(spaceKey: string): Promise<SpaceMemberListResponse> {
  return apiRequest<SpaceMemberListResponse>(
    `/api/v1/spaces/${encodeURIComponent(spaceKey)}/members`,
  )
}

export function upsertSpaceMember(
  spaceKey: string,
  userId: string,
  body: UpsertSpaceMemberRequest,
): Promise<SpaceMemberResponse> {
  return apiRequest<SpaceMemberResponse>(
    `/api/v1/spaces/${encodeURIComponent(spaceKey)}/members/${encodeURIComponent(userId)}`,
    {
      method: 'PUT',
      body,
    },
  )
}

export function deleteSpaceMember(spaceKey: string, userId: string): Promise<void> {
  return apiRequest<void>(
    `/api/v1/spaces/${encodeURIComponent(spaceKey)}/members/${encodeURIComponent(userId)}`,
    {
      method: 'DELETE',
    },
  )
}

export function createDocument(spaceKey: string, body: CreateDocumentRequest): Promise<Document> {
  return apiRequest<Document>(`/api/v1/spaces/${encodeURIComponent(spaceKey)}/documents`, {
    method: 'POST',
    body,
  })
}

export function getDocument(documentId: string): Promise<Document> {
  return apiRequest<Document>(`/api/v1/documents/${encodeURIComponent(documentId)}`)
}

export function updateDocumentDraft(
  documentId: string,
  body: UpdateDocumentDraftRequest,
): Promise<Document> {
  return apiRequest<Document>(`/api/v1/documents/${encodeURIComponent(documentId)}/draft`, {
    method: 'PUT',
    body,
  })
}

export function publishDocument(
  documentId: string,
  body: PublishDocumentRequest,
): Promise<DocumentRevision> {
  return apiRequest<DocumentRevision>(
    `/api/v1/documents/${encodeURIComponent(documentId)}/publish`,
    {
      method: 'POST',
      body,
    },
  )
}

export function archiveDocument(documentId: string): Promise<Document> {
  return apiRequest<Document>(`/api/v1/documents/${encodeURIComponent(documentId)}/archive`, {
    method: 'POST',
  })
}

export function moveDocument(documentId: string, body: MoveDocumentRequest): Promise<Document> {
  return apiRequest<Document>(`/api/v1/documents/${encodeURIComponent(documentId)}/move`, {
    method: 'POST',
    body,
  })
}

export function listDocumentRevisions(
  documentId: string,
  params: DocumentRevisionListParams = {},
): Promise<DocumentRevisionListResponse> {
  return apiRequest<DocumentRevisionListResponse>(
    `/api/v1/documents/${encodeURIComponent(documentId)}/revisions${queryString(params)}`,
  )
}

export function getDocumentRevision(
  documentId: string,
  revisionId: string,
): Promise<DocumentRevision> {
  return apiRequest<DocumentRevision>(
    `/api/v1/documents/${encodeURIComponent(documentId)}/revisions/${encodeURIComponent(revisionId)}`,
  )
}

export function listTasks(spaceKey: string): Promise<TaskPageListResponse> {
  return apiRequest<TaskPageListResponse>(`/api/v1/spaces/${encodeURIComponent(spaceKey)}/tasks`)
}

export function getTask(spaceKey: string, taskKey: string): Promise<TaskPage> {
  return apiRequest<TaskPage>(
    `/api/v1/spaces/${encodeURIComponent(spaceKey)}/tasks/${encodeURIComponent(taskKey)}`,
  )
}

export function linkTaskDocument(
  spaceKey: string,
  taskKey: string,
  body: LinkDocumentRequest,
): Promise<TaskPage> {
  return apiRequest<TaskPage>(
    `/api/v1/spaces/${encodeURIComponent(spaceKey)}/tasks/${encodeURIComponent(taskKey)}/links/documents`,
    {
      method: 'POST',
      body,
    },
  )
}

export function listPhases(spaceKey: string): Promise<PhasePageListResponse> {
  return apiRequest<PhasePageListResponse>(`/api/v1/spaces/${encodeURIComponent(spaceKey)}/phases`)
}

export function getPhase(spaceKey: string, phaseKey: string): Promise<PhasePage> {
  return apiRequest<PhasePage>(
    `/api/v1/spaces/${encodeURIComponent(spaceKey)}/phases/${encodeURIComponent(phaseKey)}`,
  )
}

export function linkPhaseDocument(
  spaceKey: string,
  phaseKey: string,
  body: LinkDocumentRequest,
): Promise<PhasePage> {
  return apiRequest<PhasePage>(
    `/api/v1/spaces/${encodeURIComponent(spaceKey)}/phases/${encodeURIComponent(phaseKey)}/links/documents`,
    {
      method: 'POST',
      body,
    },
  )
}

export function listEvidence(params: EvidenceListParams = {}): Promise<EvidenceListResponse> {
  return apiRequest<EvidenceListResponse>(`/api/v1/evidence${queryString(params)}`)
}

export function getEvidence(evidenceId: string): Promise<Evidence> {
  return apiRequest<Evidence>(`/api/v1/evidence/${encodeURIComponent(evidenceId)}`)
}

export function createEvidence(body: CreateEvidenceRequest): Promise<Evidence> {
  return apiRequest<Evidence>('/api/v1/evidence', { method: 'POST', body })
}

export function uploadAttachment(file: File): Promise<Attachment> {
  const form = new FormData()
  form.set('file', file)
  return apiRequest<Attachment>('/api/v1/attachments', { method: 'POST', body: form })
}

export function getAttachment(attachmentId: string): Promise<Attachment> {
  return apiRequest<Attachment>(`/api/v1/attachments/${encodeURIComponent(attachmentId)}`)
}

export function downloadAttachment(attachmentId: string): Promise<AttachmentDownload> {
  return apiBlobRequest(`/api/v1/attachments/${encodeURIComponent(attachmentId)}/download`)
}

export function listTemplates(): Promise<TemplateListResponse> {
  return apiRequest<TemplateListResponse>('/api/v1/templates')
}

export function createTemplate(body: CreateTemplateRequest): Promise<Template> {
  return apiRequest<Template>('/api/v1/templates', { method: 'POST', body })
}

export function listAuditLog(params: AuditLogParams = {}): Promise<AuditLogResponse> {
  return apiRequest<AuditLogResponse>(`/api/v1/audit-log${queryString(params)}`)
}

export function listUsers(): Promise<WikiUserListResponse> {
  return apiRequest<WikiUserListResponse>('/api/v1/users')
}

export function createUser(body: CreateUserRequest): Promise<User> {
  return apiRequest<User>('/api/v1/users', { method: 'POST', body })
}

export function updateUser(userId: string, body: UpdateUserRequest): Promise<User> {
  return apiRequest<User>(`/api/v1/users/${encodeURIComponent(userId)}`, {
    method: 'PUT',
    body,
  })
}

export function getWikiSettings(): Promise<WikiSettingsResponse> {
  return apiRequest<WikiSettingsResponse>('/api/v1/settings')
}

export function searchWiki(params: SearchParams): Promise<SearchResponse> {
  return apiRequest<SearchResponse>(`/api/v1/search${queryString(params)}`)
}
