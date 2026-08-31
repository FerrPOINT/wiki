import { apiRequest } from './client'

export type Space = {
  id: string
  key: string
  name: string
  description?: string | null
  owner_id: string
  status: string
  document_count: number
  member_count: number
  created_at: string
  updated_at: string
}

export type SpaceTreeNode = {
  id: string
  slug: string
  title: string
  document_type: string
  status: string
  children: SpaceTreeNode[]
}

export type DocumentRevision = {
  id: string
  document_id: string
  version: number
  title: string
  body_markdown: string
  summary?: string | null
  author_id: string
  published_at: string
}

export type Evidence = {
  id: string
  space_key: string
  document_id?: string | null
  task_key?: string | null
  phase_key?: string | null
  title: string
  evidence_type: string
  url?: string | null
  attachment_id?: string | null
  checksum?: string | null
  created_by: string
  created_at: string
}

export type Attachment = {
  id: string
  file_name: string
  content_type: string
  size_bytes: number
  checksum: string
  uploaded_by: string
  uploaded_at: string
}

export type DocumentSummary = {
  id: string
  slug: string
  title: string
  document_type: string
  status: string
  updated_at: string
}

export type Document = DocumentSummary & {
  space_key: string
  parent_id?: string | null
  body_markdown: string
  draft_markdown: string
  current_revision?: DocumentRevision | null
  task_keys: string[]
  phase_keys: string[]
  evidence: Evidence[]
  created_by: string
  updated_by: string
  created_at: string
}

export type CreateDocumentRequest = {
  title: string
  slug?: string | null
  document_type: string
  parent_id?: string | null
  content_markdown: string
  task_key?: string | null
  phase_key?: string | null
}

export type TaskPage = {
  space_key: string
  task_key: string
  title?: string | null
  document_count: number
  evidence_count: number
  documents: DocumentSummary[]
  evidence: Evidence[]
}

export type PhasePage = {
  space_key: string
  phase_key: string
  title?: string | null
  document_count: number
  evidence_count: number
  documents: DocumentSummary[]
  evidence: Evidence[]
}

export type Template = {
  id: string
  name: string
  document_type: string
  body_markdown: string
}

export type AuditEntry = {
  id: string
  actor_id: string
  action: string
  entity_type: string
  entity_id: string
  created_at: string
}

export type User = {
  id: string
  email: string
  username?: string | null
  display_name?: string | null
  role?: string | null
  is_system_admin?: boolean
  active?: boolean
}

export type CreateUserRequest = {
  email: string
  username: string
  password: string
  display_name: string
  role: string
}

export type SearchResult = {
  id: string
  result_type: string
  title: string
  space_key: string
  url: string
  snippet: string
  updated_at: string
}

export type SearchParams = {
  q?: string
  space?: string
  task_key?: string
  phase_key?: string
  document_type?: string
  include_archived?: boolean
  limit?: number
}

export type CreateEvidenceRequest = {
  space?: string | null
  document_id?: string | null
  task_key?: string | null
  phase_key?: string | null
  title: string
  evidence_type: string
  url?: string | null
  attachment_id?: string | null
  checksum?: string | null
}

function queryString(params: Record<string, string | number | boolean | null | undefined>): string {
  const query = new URLSearchParams()
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== null && value !== '') query.set(key, String(value))
  }
  const serialized = query.toString()
  return serialized ? `?${serialized}` : ''
}

export function listSpaces(): Promise<{ spaces: Space[] }> {
  return apiRequest('/api/v1/spaces')
}

export function getSpaceTree(
  spaceKey: string,
): Promise<{ space_key: string; documents: SpaceTreeNode[] }> {
  return apiRequest(`/api/v1/spaces/${encodeURIComponent(spaceKey)}/tree`)
}

export function createDocument(spaceKey: string, body: CreateDocumentRequest): Promise<Document> {
  return apiRequest(`/api/v1/spaces/${encodeURIComponent(spaceKey)}/documents`, {
    method: 'POST',
    body,
  })
}

export function getDocument(documentId: string): Promise<Document> {
  return apiRequest(`/api/v1/documents/${encodeURIComponent(documentId)}`)
}

export function listDocumentRevisions(
  documentId: string,
): Promise<{ revisions: DocumentRevision[] }> {
  return apiRequest(`/api/v1/documents/${encodeURIComponent(documentId)}/revisions`)
}

export function listTasks(spaceKey: string): Promise<{ tasks: TaskPage[] }> {
  return apiRequest(`/api/v1/spaces/${encodeURIComponent(spaceKey)}/tasks`)
}

export function getTask(spaceKey: string, taskKey: string): Promise<TaskPage> {
  return apiRequest(
    `/api/v1/spaces/${encodeURIComponent(spaceKey)}/tasks/${encodeURIComponent(taskKey)}`,
  )
}

export function listPhases(spaceKey: string): Promise<{ phases: PhasePage[] }> {
  return apiRequest(`/api/v1/spaces/${encodeURIComponent(spaceKey)}/phases`)
}

export function getPhase(spaceKey: string, phaseKey: string): Promise<PhasePage> {
  return apiRequest(
    `/api/v1/spaces/${encodeURIComponent(spaceKey)}/phases/${encodeURIComponent(phaseKey)}`,
  )
}

export function listEvidence(
  params: Partial<SearchParams> = {},
): Promise<{ evidence: Evidence[] }> {
  return apiRequest(`/api/v1/evidence${queryString(params)}`)
}

export function createEvidence(body: CreateEvidenceRequest): Promise<Evidence> {
  return apiRequest('/api/v1/evidence', { method: 'POST', body })
}

export function uploadAttachment(file: File): Promise<Attachment> {
  const form = new FormData()
  form.set('file', file)
  return apiRequest('/api/v1/attachments', { method: 'POST', body: form })
}

export function listTemplates(): Promise<{ templates: Template[] }> {
  return apiRequest('/api/v1/templates')
}

export function listAuditLog(): Promise<{ entries: AuditEntry[] }> {
  return apiRequest('/api/v1/audit-log')
}

export function listUsers(): Promise<{ users: User[] }> {
  return apiRequest('/api/v1/users')
}

export function createUser(body: CreateUserRequest): Promise<User> {
  return apiRequest('/api/v1/users', { method: 'POST', body })
}

export function searchWiki(params: SearchParams): Promise<{ results: SearchResult[] }> {
  return apiRequest(`/api/v1/search${queryString(params)}`)
}
