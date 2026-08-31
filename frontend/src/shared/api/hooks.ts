import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useNavigate } from 'react-router'
import { getCurrentUser, login, logout, register } from '@/api/auth'
import {
  createDocument,
  createEvidence,
  createUser,
  getDocument,
  getPhase,
  getSpaceTree,
  getTask,
  listAuditLog,
  listDocumentRevisions,
  listEvidence,
  listPhases,
  listSpaces,
  listTasks,
  listTemplates,
  listUsers,
  searchWiki,
  type CreateDocumentRequest,
  type CreateEvidenceRequest,
  type SearchParams,
  uploadAttachment,
  type CreateUserRequest,
} from '@/api/wiki'
import { storeRefreshToken, useAuthStore } from '@/shared/auth/store'

const authKeys = {
  me: ['me'] as const,
}

export const defaultSpaceKey = 'SDLC'

export const wikiKeys = {
  spaces: ['wiki', 'spaces'] as const,
  spaceTree: (spaceKey: string) => ['wiki', 'spaces', spaceKey, 'tree'] as const,
  document: (documentId: string) => ['wiki', 'documents', documentId] as const,
  documentRevisions: (documentId: string) =>
    ['wiki', 'documents', documentId, 'revisions'] as const,
  tasks: (spaceKey: string) => ['wiki', 'spaces', spaceKey, 'tasks'] as const,
  task: (spaceKey: string, taskKey: string) =>
    ['wiki', 'spaces', spaceKey, 'tasks', taskKey] as const,
  phases: (spaceKey: string) => ['wiki', 'spaces', spaceKey, 'phases'] as const,
  phase: (spaceKey: string, phaseKey: string) =>
    ['wiki', 'spaces', spaceKey, 'phases', phaseKey] as const,
  evidence: (params: Partial<SearchParams>) => ['wiki', 'evidence', params] as const,
  templates: ['wiki', 'templates'] as const,
  auditLog: ['wiki', 'audit-log'] as const,
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

export function useDocumentRevisions(documentId: string) {
  return useQuery({
    queryKey: wikiKeys.documentRevisions(documentId),
    queryFn: () => listDocumentRevisions(documentId),
    enabled: Boolean(documentId),
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

export function useEvidence(params: Partial<SearchParams> = {}) {
  return useQuery({
    queryKey: wikiKeys.evidence(params),
    queryFn: () => listEvidence(params),
  })
}

export function useTemplates() {
  return useQuery({
    queryKey: wikiKeys.templates,
    queryFn: listTemplates,
  })
}

export function useAuditLog() {
  return useQuery({
    queryKey: wikiKeys.auditLog,
    queryFn: listAuditLog,
  })
}

export function useUsers() {
  return useQuery({
    queryKey: wikiKeys.users,
    queryFn: listUsers,
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
        checksum: attachment.checksum,
      })
    },
    onSuccess: (evidence) => {
      queryClient.invalidateQueries({ queryKey: ['wiki', 'evidence'] })
      queryClient.invalidateQueries({ queryKey: wikiKeys.tasks(evidence.space_key) })
      queryClient.invalidateQueries({ queryKey: wikiKeys.phases(evidence.space_key) })
      queryClient.invalidateQueries({ queryKey: ['wiki', 'search'] })
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
