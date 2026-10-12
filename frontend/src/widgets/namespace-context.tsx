import { useLocation, useMatch, useNavigate } from 'react-router'
import { useQuery } from '@tanstack/react-query'
import { NamespacePicker } from '@sdlc/ui/ui'
import {
  parseNamespaceLocation,
  projectIdentity,
  projectNavigationScope,
  withNamespaceLocation,
} from '@sdlc/ui/lib'
import type { components } from '@/api/generated'
import { apiRequest } from '@/api/client'
import { useWorkspace, useWorkspaceCatalog, workspacePath } from '@/api/workspaces'
import { useAuthStore } from '@/shared/auth/store'
export type ResourceContext = components['schemas']['ResourceContextSummary']

/** Presentation-only scope for controls; the shell resolves and authorizes the resource. */
export function useProjectRouteRef() {
  const location=useLocation(),match=useMatch('/workspaces/:registry/:namespace/*')
  const pathRef=match?parseNamespaceLocation(`?registry_instance_id=${match.params.registry}&namespace_id=${match.params.namespace}`):null
  return projectNavigationScope(location.search,pathRef)
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
  const directSpace = useQuery({
    queryKey: ['wiki', 'space-context', actor, explicitSpace],
    enabled: enabled && !scope.ref && !scope.malformed && spaces.length === 1 && !!explicitSpace,
    queryFn: ({ signal }) =>
      apiRequest<ResourceContext | null>(
        `/api/v1/spaces/${encodeURIComponent(explicitSpace!)}/namespace-context`,
        { signal },
      ),
  })
  const document = useMatch('/documents/:documentId/*')?.params.documentId
  const direct = useQuery({
    queryKey: ['wiki', 'document-context', actor, document],
    enabled: enabled && !scope.ref && !scope.malformed && !!document && document !== 'new',
    queryFn: ({ signal }) =>
      apiRequest<ResourceContext | null>(`/api/v1/documents/${document}/namespace-context`, {
        signal,
      }),
  })
  const ref =
    scope.ref ?? direct.data?.binding.namespace ?? directSpace.data?.binding.namespace ?? null
  const malformed = scope.malformed || Boolean(match && !pathRef) || spaces.length > 1
  const workspace = useWorkspace(ref, enabled && !malformed)
  const resource = useQuery({
    queryKey: ['wiki', 'namespace-context', actor, ref?.registry_instance_id, ref?.namespace_id],
    enabled: enabled && !!ref && !malformed,
    queryFn: ({ signal }) =>
      apiRequest<ResourceContext>(`/api/v1/namespace-contexts/${projectIdentity(ref!)}`, {
        signal,
      }),
  })
  return {
    ref,
    malformed,
    all: scope.all,
    workspace,
    query: {
      ...resource,
      isPending:
        (resource.isPending && !!ref) ||
        (direct.isPending && direct.fetchStatus === 'fetching') ||
        (directSpace.isPending && directSpace.fetchStatus === 'fetching'),
      isError: resource.isError || direct.isError || directSpace.isError,
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
