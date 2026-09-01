import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useNavigate } from 'react-router'
import { getCurrentUser, login, logout, register } from '@/api/auth'
import {
  archiveSpace,
  archiveDocument,
  createDocument,
  createEvidence,
  createSpace,
  createTemplate,
  createUser,
  deleteSpaceMember,
  downloadAttachment,
  getDocument,
  getDocumentRevision,
  getEvidence,
  getAttachment,
  getWikiSettings,
  getPhase,
  getSpaceTree,
  getTask,
  linkPhaseDocument,
  linkTaskDocument,
  listSpaceMembers,
  listAuditLog,
  listDocumentRevisions,
  listEvidence,
  listPhases,
  listSpaces,
  listTasks,
  listTemplates,
  listUsers,
  moveDocument,
  publishDocument,
  searchWiki,
  type AuditLogParams,
  type CreateSpaceRequest,
  type CreateDocumentRequest,
  type CreateEvidenceRequest,
  type Document,
  type DocumentRevisionListParams,
  type EvidenceListParams,
  type LinkDocumentRequest,
  type MoveDocumentRequest,
  type PublishDocumentRequest,
  type SearchParams,
  updateDocumentDraft,
  updateSpace,
  updateUser,
  type UpdateDocumentDraftRequest,
  type UpdateSpaceRequest,
  type UpdateUserRequest,
  uploadAttachment,
  upsertSpaceMember,
  type CreateTemplateRequest,
  type CreateUserRequest,
  type UpsertSpaceMemberRequest,
} from '@/api/wiki'
import { storeRefreshToken, useAuthStore } from '@/shared/auth/store'

const authKeys = {
  me: ['me'] as const,
}

export const defaultSpaceKey = 'SDLC'
const defaultDocumentRevisionListParams: DocumentRevisionListParams = { limit: 20 }
const defaultEvidenceListParams: EvidenceListParams = { limit: 30 }
const defaultAuditLogParams: AuditLogParams = { limit: 50 }

export const wikiKeys = {
  spaces: ['wiki', 'spaces'] as const,
  settings: ['wiki', 'settings'] as const,
  spaceMembers: (spaceKey: string) => ['wiki', 'spaces', spaceKey, 'members'] as const,
  spaceTree: (spaceKey: string) => ['wiki', 'spaces', spaceKey, 'tree'] as const,
  document: (documentId: string) => ['wiki', 'documents', documentId] as const,
  documentRevisionsList: (documentId: string, params: DocumentRevisionListParams = {}) =>
    ['wiki', 'documents', documentId, 'revisions', params] as const,
  documentRevision: (documentId: string, revisionId: string) =>
    ['wiki', 'documents', documentId, 'revisions', revisionId] as const,
  tasks: (spaceKey: string) => ['wiki', 'spaces', spaceKey, 'tasks'] as const,
  task: (spaceKey: string, taskKey: string) =>
    ['wiki', 'spaces', spaceKey, 'tasks', taskKey] as const,
  phases: (spaceKey: string) => ['wiki', 'spaces', spaceKey, 'phases'] as const,
  phase: (spaceKey: string, phaseKey: string) =>
    ['wiki', 'spaces', spaceKey, 'phases', phaseKey] as const,
  evidence: (params: EvidenceListParams) => ['wiki', 'evidence', params] as const,
  evidenceItem: (evidenceId: string) => ['wiki', 'evidence', evidenceId] as const,
  attachment: (attachmentId: string) => ['wiki', 'attachments', attachmentId] as const,
  templates: ['wiki', 'templates'] as const,
  auditLog: ['wiki', 'audit-log'] as const,
  auditLogList: (params: AuditLogParams = {}) => ['wiki', 'audit-log', params] as const,
  users: ['wiki', 'users'] as const,
  search: (params: SearchParams) => ['wiki', 'search', params] as const,
}

export function useLogin() {
  const setAuth = useAuthStore((state) => state.setAuth)
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: login,
    onSuccess: (data) => {
      storeRefreshToken(data.refresh_token ?? null)
      setAuth({
        token: data.access_token,
        userId: data.user_id,
        email: data.email,
        username: data.username ?? undefined,
        displayName: data.display_name ?? undefined,
      })
      queryClient.invalidateQueries({ queryKey: authKeys.me })
    },
  })
}

export function useRegister() {
  const setAuth = useAuthStore((state) => state.setAuth)
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: register,
    onSuccess: (data) => {
      storeRefreshToken(data.refresh_token ?? null)
      setAuth({
        token: data.access_token,
        userId: data.user_id,
        email: data.email,
        username: data.username ?? undefined,
        displayName: data.display_name ?? undefined,
      })
      queryClient.invalidateQueries({ queryKey: authKeys.me })
    },
  })
}

export function useCurrentUser() {
  const token = useAuthStore((state) => state.token)
  return useQuery({
    queryKey: authKeys.me,
    queryFn: getCurrentUser,
    enabled: Boolean(token),
    staleTime: 5 * 60 * 1000,
  })
}

export function useLogout() {
  const clearAuth = useAuthStore((state) => state.logout)
  const queryClient = useQueryClient()
  const navigate = useNavigate()

  return useMutation({
    mutationFn: logout,
    onSettled: () => {
      clearAuth()
      queryClient.clear()
      navigate('/login')
    },
  })
}

export function useSpaces() {
  return useQuery({
    queryKey: wikiKeys.spaces,
    queryFn: listSpaces,
  })
}

export function useCreateSpace() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (body: CreateSpaceRequest) => createSpace(body),
    onSuccess: (space) => {
      queryClient.invalidateQueries({ queryKey: wikiKeys.spaces })
      queryClient.invalidateQueries({ queryKey: wikiKeys.spaceTree(space.key) })
      queryClient.invalidateQueries({ queryKey: wikiKeys.auditLog })
    },
  })
}

export function useUpdateSpace() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ spaceKey, body }: { spaceKey: string; body: UpdateSpaceRequest }) =>
      updateSpace(spaceKey, body),
    onSuccess: (space, variables) => {
      queryClient.invalidateQueries({ queryKey: wikiKeys.spaces })
      queryClient.invalidateQueries({ queryKey: wikiKeys.spaceTree(space.key) })
      queryClient.invalidateQueries({ queryKey: wikiKeys.spaceTree(variables.spaceKey) })
      queryClient.invalidateQueries({ queryKey: wikiKeys.auditLog })
    },
  })
}

export function useArchiveSpace() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: archiveSpace,
    onSuccess: (space, requestedSpaceKey) => {
      queryClient.invalidateQueries({ queryKey: wikiKeys.spaces })
      queryClient.invalidateQueries({ queryKey: wikiKeys.spaceTree(space.key) })
      queryClient.invalidateQueries({ queryKey: wikiKeys.spaceTree(requestedSpaceKey) })
      queryClient.invalidateQueries({ queryKey: wikiKeys.auditLog })
    },
  })
}

export function useWikiSettings() {
  return useQuery({
    queryKey: wikiKeys.settings,
    queryFn: getWikiSettings,
  })
}

export function useSpaceMembers(spaceKey: string, enabled = true) {
  return useQuery({
    queryKey: wikiKeys.spaceMembers(spaceKey),
    queryFn: () => listSpaceMembers(spaceKey),
    enabled: enabled && Boolean(spaceKey),
  })
}

export function useUpsertSpaceMember() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({
      spaceKey,
      userId,
      body,
    }: {
      spaceKey: string
      userId: string
      body: UpsertSpaceMemberRequest
    }) => upsertSpaceMember(spaceKey, userId, body),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: wikiKeys.spaceMembers(variables.spaceKey) })
      queryClient.invalidateQueries({ queryKey: wikiKeys.spaces })
      queryClient.invalidateQueries({ queryKey: wikiKeys.auditLog })
    },
  })
}

export function useDeleteSpaceMember() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ spaceKey, userId }: { spaceKey: string; userId: string }) =>
      deleteSpaceMember(spaceKey, userId),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: wikiKeys.spaceMembers(variables.spaceKey) })
      queryClient.invalidateQueries({ queryKey: wikiKeys.spaces })
      queryClient.invalidateQueries({ queryKey: wikiKeys.auditLog })
    },
  })
}

export function useSpaceTree(spaceKey: string) {
  return useQuery({
    queryKey: wikiKeys.spaceTree(spaceKey),
    queryFn: () => getSpaceTree(spaceKey),
    enabled: Boolean(spaceKey),
  })
}

export function useDocument(documentId: string) {
  return useQuery({
    queryKey: wikiKeys.document(documentId),
    queryFn: () => getDocument(documentId),
    enabled: Boolean(documentId),
  })
}

export function useDocumentRevisions(
  documentId: string,
  params: DocumentRevisionListParams = defaultDocumentRevisionListParams,
) {
  return useQuery({
    queryKey: wikiKeys.documentRevisionsList(documentId, params),
    queryFn: () => listDocumentRevisions(documentId, params),
    enabled: Boolean(documentId),
  })
}

export function useDocumentRevision(documentId: string, revisionId: string, enabled = true) {
  return useQuery({
    queryKey: wikiKeys.documentRevision(documentId, revisionId),
    queryFn: () => getDocumentRevision(documentId, revisionId),
    enabled: enabled && Boolean(documentId) && Boolean(revisionId),
  })
}

export function useTasks(spaceKey = defaultSpaceKey) {
  return useQuery({
    queryKey: wikiKeys.tasks(spaceKey),
    queryFn: () => listTasks(spaceKey),
  })
}

export function useTask(taskKey: string, spaceKey = defaultSpaceKey) {
  return useQuery({
    queryKey: wikiKeys.task(spaceKey, taskKey),
    queryFn: () => getTask(spaceKey, taskKey),
    enabled: Boolean(taskKey),
  })
}

export function useLinkTaskDocument() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({
      spaceKey,
      taskKey,
      body,
    }: {
      spaceKey: string
      taskKey: string
      body: LinkDocumentRequest
    }) => linkTaskDocument(spaceKey, taskKey, body),
    onSuccess: (task, variables) => {
      queryClient.setQueryData(wikiKeys.task(variables.spaceKey, variables.taskKey), task)
      queryClient.invalidateQueries({ queryKey: wikiKeys.tasks(variables.spaceKey) })
      queryClient.invalidateQueries({ queryKey: ['wiki', 'documents'] })
      queryClient.invalidateQueries({ queryKey: ['wiki', 'search'] })
      queryClient.invalidateQueries({ queryKey: wikiKeys.auditLog })
    },
  })
}

export function usePhases(spaceKey = defaultSpaceKey) {
  return useQuery({
    queryKey: wikiKeys.phases(spaceKey),
    queryFn: () => listPhases(spaceKey),
  })
}

export function usePhase(phaseKey: string, spaceKey = defaultSpaceKey) {
  return useQuery({
    queryKey: wikiKeys.phase(spaceKey, phaseKey),
    queryFn: () => getPhase(spaceKey, phaseKey),
    enabled: Boolean(phaseKey),
  })
}

export function useLinkPhaseDocument() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({
      spaceKey,
      phaseKey,
      body,
    }: {
      spaceKey: string
      phaseKey: string
      body: LinkDocumentRequest
    }) => linkPhaseDocument(spaceKey, phaseKey, body),
    onSuccess: (phase, variables) => {
      queryClient.setQueryData(wikiKeys.phase(variables.spaceKey, variables.phaseKey), phase)
      queryClient.invalidateQueries({ queryKey: wikiKeys.phases(variables.spaceKey) })
      queryClient.invalidateQueries({ queryKey: ['wiki', 'documents'] })
      queryClient.invalidateQueries({ queryKey: ['wiki', 'search'] })
      queryClient.invalidateQueries({ queryKey: wikiKeys.auditLog })
    },
  })
}

export function useEvidence(params: EvidenceListParams = defaultEvidenceListParams) {
  return useQuery({
    queryKey: wikiKeys.evidence(params),
    queryFn: () => listEvidence(params),
  })
}

export function useEvidenceItem(evidenceId: string | null | undefined) {
  return useQuery({
    queryKey: wikiKeys.evidenceItem(evidenceId ?? ''),
    queryFn: () => getEvidence(evidenceId ?? ''),
    enabled: Boolean(evidenceId),
  })
}

export function useAttachment(attachmentId: string | null | undefined) {
  return useQuery({
    queryKey: wikiKeys.attachment(attachmentId ?? ''),
    queryFn: () => getAttachment(attachmentId ?? ''),
    enabled: Boolean(attachmentId),
  })
}

export function useDownloadAttachment() {
  return useMutation({
    mutationFn: downloadAttachment,
  })
}

export function useTemplates() {
  return useQuery({
    queryKey: wikiKeys.templates,
    queryFn: listTemplates,
  })
}

export function useCreateTemplate() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (body: CreateTemplateRequest) => createTemplate(body),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: wikiKeys.templates })
      queryClient.invalidateQueries({ queryKey: wikiKeys.auditLog })
    },
  })
}

export function useAuditLog(params: AuditLogParams = defaultAuditLogParams) {
  return useQuery({
    queryKey: wikiKeys.auditLogList(params),
    queryFn: () => listAuditLog(params),
  })
}

export function useUsers(enabled = true) {
  return useQuery({
    queryKey: wikiKeys.users,
    queryFn: listUsers,
    enabled,
  })
}

export function useWikiSearch(params: SearchParams) {
  return useQuery({
    queryKey: wikiKeys.search(params),
    queryFn: () => searchWiki(params),
  })
}

export function useCreateDocument() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ spaceKey, body }: { spaceKey: string; body: CreateDocumentRequest }) =>
      createDocument(spaceKey, body),
    onSuccess: (document) => {
      queryClient.setQueryData(wikiKeys.document(document.slug), document)
      queryClient.invalidateQueries({ queryKey: wikiKeys.spaces })
      queryClient.invalidateQueries({ queryKey: wikiKeys.spaceTree(document.space_key) })
      queryClient.invalidateQueries({ queryKey: wikiKeys.tasks(document.space_key) })
      queryClient.invalidateQueries({ queryKey: wikiKeys.phases(document.space_key) })
      queryClient.invalidateQueries({ queryKey: ['wiki', 'search'] })
      queryClient.invalidateQueries({ queryKey: wikiKeys.auditLog })
    },
  })
}

function writeDocumentCache(
  queryClient: ReturnType<typeof useQueryClient>,
  document: Document,
  requestedId?: string,
) {
  if (requestedId) queryClient.setQueryData(wikiKeys.document(requestedId), document)
  queryClient.setQueryData(wikiKeys.document(document.id), document)
  queryClient.setQueryData(wikiKeys.document(document.slug), document)
  queryClient.invalidateQueries({ queryKey: wikiKeys.spaces })
  queryClient.invalidateQueries({ queryKey: wikiKeys.spaceTree(document.space_key) })
  queryClient.invalidateQueries({ queryKey: wikiKeys.tasks(document.space_key) })
  queryClient.invalidateQueries({ queryKey: wikiKeys.phases(document.space_key) })
  queryClient.invalidateQueries({ queryKey: ['wiki', 'search'] })
  queryClient.invalidateQueries({ queryKey: wikiKeys.auditLog })
}

export function useUpdateDocumentDraft() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ documentId, body }: { documentId: string; body: UpdateDocumentDraftRequest }) =>
      updateDocumentDraft(documentId, body),
    onSuccess: (document, variables) => {
      writeDocumentCache(queryClient, document, variables.documentId)
    },
  })
}

export function usePublishDocument() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ documentId, body }: { documentId: string; body: PublishDocumentRequest }) =>
      publishDocument(documentId, body),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['wiki', 'documents'] })
      queryClient.invalidateQueries({ queryKey: ['wiki', 'spaces'] })
      queryClient.invalidateQueries({ queryKey: ['wiki', 'search'] })
      queryClient.invalidateQueries({ queryKey: wikiKeys.auditLog })
    },
  })
}

export function useArchiveDocument() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: archiveDocument,
    onSuccess: (document, requestedId) => {
      writeDocumentCache(queryClient, document, requestedId)
    },
  })
}

export function useMoveDocument() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ documentId, body }: { documentId: string; body: MoveDocumentRequest }) =>
      moveDocument(documentId, body),
    onSuccess: (document, variables) => {
      writeDocumentCache(queryClient, document, variables.documentId)
    },
  })
}

export function useCreateEvidence() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: createEvidence,
    onSuccess: (evidence) => {
      queryClient.invalidateQueries({ queryKey: ['wiki', 'evidence'] })
      queryClient.invalidateQueries({ queryKey: wikiKeys.tasks(evidence.space_key) })
      queryClient.invalidateQueries({ queryKey: wikiKeys.phases(evidence.space_key) })
      queryClient.invalidateQueries({ queryKey: ['wiki', 'search'] })
      queryClient.invalidateQueries({ queryKey: wikiKeys.auditLog })
    },
  })
}

export function useCreateFileEvidence() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({
      file,
      evidence,
    }: {
      file: File
      evidence: Omit<CreateEvidenceRequest, 'evidence_type' | 'attachment_id' | 'checksum' | 'url'>
    }) => {
      const attachment = await uploadAttachment(file)
      return createEvidence({
        ...evidence,
        evidence_type: 'uploaded_file',
        attachment_id: attachment.id,
      })
    },
    onSuccess: (evidence) => {
      queryClient.invalidateQueries({ queryKey: ['wiki', 'evidence'] })
      queryClient.invalidateQueries({ queryKey: wikiKeys.tasks(evidence.space_key) })
      queryClient.invalidateQueries({ queryKey: wikiKeys.phases(evidence.space_key) })
      queryClient.invalidateQueries({ queryKey: ['wiki', 'search'] })
      queryClient.invalidateQueries({ queryKey: wikiKeys.auditLog })
    },
  })
}

export function useCreateUser() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (body: CreateUserRequest) => createUser(body),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: wikiKeys.users })
      queryClient.invalidateQueries({ queryKey: wikiKeys.auditLog })
    },
  })
}

export function useUpdateUser() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ userId, body }: { userId: string; body: UpdateUserRequest }) =>
      updateUser(userId, body),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: wikiKeys.users })
      queryClient.invalidateQueries({ queryKey: wikiKeys.auditLog })
      queryClient.invalidateQueries({ queryKey: authKeys.me })
    },
  })
}
