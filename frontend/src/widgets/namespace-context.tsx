import { useLocation, useMatch, useNavigate } from 'react-router'
import { useQuery } from '@tanstack/react-query'
import { NamespacePicker } from '@pdlc/ui/ui'
import {
  parseNamespaceLocation,
  projectIdentity,
  projectNavigationScope,
  withNamespaceLocation,
} from '@pdlc/ui/lib'
import { apiRequest } from '@/api/client'
import {
  useWorkspace,
  useWorkspaceCatalog,
  useSpaceProjects,
  workspacePath,
  type SpaceProjects,
} from '@/api/workspaces'
import { useAuthStore } from '@/shared/auth/store'

/** Presentation-only scope for controls; the shell resolves and authorizes the resource. */
export function useProjectRouteRef() {
  const location = useLocation(),
    match = useMatch('/workspaces/:registry/:namespace/*')
  const pathRef = match
    ? parseNamespaceLocation(
        `?registry_instance_id=${match.params.registry}&namespace_id=${match.params.namespace}`,
      )
    : null
  return import.meta.env.VITE_NAMESPACE_ENABLED === 'true'
    ? projectNavigationScope(location.search, pathRef)
    : projectNavigationScope('')
}

export function useNamespaceContext() {
  const location = useLocation()
  const match = useMatch('/workspaces/:registry/:namespace/*')
  const pathRef = match
    ? parseNamespaceLocation(
        `?registry_instance_id=${match.params.registry}&namespace_id=${match.params.namespace}`,
      )
    : null
  const scope = projectNavigationScope(location.search, pathRef)
  const enabled = import.meta.env.VITE_NAMESPACE_ENABLED === 'true'
  const actor = useAuthStore((state) => state.userId)
  const spaces = new URLSearchParams(location.search).getAll('space')
  const explicitSpace = spaces[0]
  const malformed = scope.malformed || Boolean(match && !pathRef) || spaces.length > 1
  const directSpace = useSpaceProjects(explicitSpace ?? '', enabled && !malformed)
  const document = useMatch('/documents/:documentId/*')?.params.documentId
  const direct = useQuery({
    queryKey: ['wiki', 'document-space', actor, document],
    enabled: enabled && !malformed && !!document && document !== 'new',
    queryFn: ({ signal }) =>
      apiRequest<SpaceProjects>(
        `/api/v1/documents/${encodeURIComponent(document!)}/space-context`,
        { signal },
      ),
  })
  const clearProjectScope = Boolean(
    enabled &&
    document &&
    document !== 'new' &&
    direct.data &&
    scope.ref &&
    !direct.data.projects.some(
      (project) => projectIdentity(project.namespace) === projectIdentity(scope.ref!),
    ),
  )
  const ref = enabled && !clearProjectScope ? scope.ref : null
  const workspace = useWorkspace(ref, enabled && !malformed)
  const choices = workspace.data?.spaces ?? []
  const own =
    document && document !== 'new' ? direct.data : explicitSpace ? directSpace.data : undefined
  const space =
    own?.space ??
    (choices.length === 1 && !explicitSpace && choices[0]?.resource_status === 'connected'
      ? choices[0]
      : undefined)
  const conflict = Boolean(
    (space && explicitSpace && space.key !== explicitSpace) ||
    (own &&
      ref &&
      !own.projects.some((project) => projectIdentity(project.namespace) === projectIdentity(ref))),
  )
  return {
    ref,
    clearProjectScope,
    malformed,
    all: clearProjectScope || scope.all,
    workspace,
    choices,
    conflict,
    projects: own?.projects ?? [],
    query: {
      data: space
        ? { resource_key: space.key, label: space.name, id: space.id, status: space.status }
        : undefined,
      isPending: Boolean(
        (ref && !own && workspace.isPending) ||
        (document && document !== 'new' && direct.isPending) ||
        (explicitSpace && directSpace.isPending),
      ),
      isError: Boolean((ref && !own && workspace.isError) || direct.isError || directSpace.isError),
    },
  }
}

export function NamespaceShellContext() {
  const navigate = useNavigate()
  const { ref, malformed, all } = useNamespaceContext()
  const catalog = useWorkspaceCatalog()
  return (
    <NamespacePicker
      projectIcons
      value={malformed ? 'invalid' : all ? '' : ref ? projectIdentity(ref) : ''}
      loading={catalog.isPending}
      unavailable={catalog.isError}
      options={(catalog.data ?? []).map((item) => ({
        value: projectIdentity(item.namespace),
        label: `${item.name} · ${item.project_key}`,
        projectKey: item.project_key,
      }))}
      onChange={(next) => {
        const [registry_instance_id = '', namespace_id = ''] = next.split('/')
        navigate(
          next
            ? withNamespaceLocation(workspacePath({ registry_instance_id, namespace_id }), {
                registry_instance_id,
                namespace_id,
              })
            : '/projects?project_scope=all',
        )
      }}
    />
  )
}
