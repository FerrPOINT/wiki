import { useMutation, useQuery, useQueryClient, type QueryClient } from '@tanstack/react-query'
import { projectIdentity, sortProjectNavigation, type NamespaceLocation } from '@pdlc/ui/lib'
import type { components } from './generated'
import { apiRequest } from './client'
import { useAuthStore } from '@/shared/auth/store'
export type Workspace = components['schemas']['WorkspaceProject']
export const workspacePath = (ref: NamespaceLocation, section = '') =>
  `/workspaces/${projectIdentity(ref)}${section ? '/' + section : ''}`

export function documentPath(
  id: string,
  search: string,
  all = new URLSearchParams(search).get('project_scope') === 'all',
) {
  const source = new URLSearchParams(search),
    params = new URLSearchParams()
  for (const key of ['task_id', 'tracker_instance_id', 'registry_instance_id', 'namespace_id'])
    for (const value of source.getAll(key)) params.append(key, value)
  if (all) params.set('project_scope', 'all')
  return `/documents/${encodeURIComponent(id)}${params.size ? '?' + params : ''}`
}
export function useWorkspace(ref: NamespaceLocation | null, enabled = true) {
  const actor = useAuthStore((state) => state.userId)
  return useQuery({
    queryKey: ['wiki', 'workspace', actor, ref?.registry_instance_id, ref?.namespace_id],
    enabled: enabled && !!ref,
    queryFn: ({ signal }) =>
      apiRequest<Workspace>(`/api/v1/workspace-projects/${projectIdentity(ref!)}`, { signal }),
  })
}
export function useWorkspaceCatalog(enabled = true) {
  const actor = useAuthStore((state) => state.userId)
  return useQuery({
    queryKey: ['wiki', 'workspace-catalog', actor],
    enabled,
    refetchInterval: 60000,
    queryFn: async ({ signal }) => {
      const items: Workspace[] = []
      for (let offset = 0; ; offset += 100) {
        const page = await apiRequest<components['schemas']['WorkspaceProjectPage']>(
          `/api/v1/workspace-projects?limit=100&offset=${offset}`,
          { signal },
        )
        items.push(...page.items)
        if (items.length >= page.total)
          return sortProjectNavigation(
            items,
            (p) => p.name,
            (p) => projectIdentity(p.namespace),
          )
        if (!page.items.length) throw new Error('Каталог изменился. Обновите страницу.')
      }
    },
  })
}

export type SpaceProjects = components['schemas']['SpaceProjectsResponse']
export type WorkspaceSpace = components['schemas']['WorkspaceSpace']
export function invalidateWorkspaceQueries(client: QueryClient) {
  return client.invalidateQueries({
    predicate: ({ queryKey }) =>
      queryKey[0] === 'wiki' &&
      ['space-projects', 'document-space', 'workspace', 'workspace-catalog', 'audit-log'].includes(
        String(queryKey[1]),
      ),
  })
}
export function useSpaceProjects(key: string, enabled = true) {
  const actor = useAuthStore((state) => state.userId)
  return useQuery({
    queryKey: ['wiki', 'space-projects', actor, key],
    enabled: enabled && !!key,
    queryFn: ({ signal }) =>
      apiRequest<SpaceProjects>(`/api/v1/spaces/${encodeURIComponent(key)}/projects`, { signal }),
  })
}
export function useSpaceProjectMutation(key: string) {
  const client = useQueryClient()
  return useMutation({
    mutationFn: ({ project, attach }: { project: NamespaceLocation; attach: boolean }) =>
      apiRequest<void>(
        `/api/v1/spaces/${encodeURIComponent(key)}/projects/${projectIdentity(project)}`,
        {
          method: attach ? 'PUT' : 'DELETE',
        },
      ),
    onSuccess: async () => {
      await invalidateWorkspaceQueries(client)
    },
  })
}
