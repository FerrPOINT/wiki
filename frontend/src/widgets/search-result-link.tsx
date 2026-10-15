import type { ReactNode } from 'react'
import { Link, useLocation } from 'react-router'
import { useQuery } from '@tanstack/react-query'
import { NamespaceLink, ResourceListRow } from '@pdlc/ui/ui'
import { withNamespaceLocation } from '@pdlc/ui/lib'
import { apiRequest, ApiError } from '@/api/client'
import type { SearchResult } from '@/api/wiki'
import { useAuthStore } from '@/shared/auth/store'
import type { SpaceProjects } from '@/api/workspaces'

function resultLocation(result: SearchResult) {
  if (result.result_type === 'document') return result.url
  const target = new URL(result.url, window.location.origin)
  target.searchParams.set('space', result.space_key)
  return target.pathname + target.search + target.hash
}

export function SearchResultLink({
  result,
  allSpaces,
  children,
}: {
  result: SearchResult
  allSpaces: boolean
  children: ReactNode
}) {
  const { search } = useLocation()
  const resolve = import.meta.env.VITE_NAMESPACE_ENABLED === 'true' && allSpaces
  if (!resolve)
    return (
      <ResourceListRow asChild interactive>
        <NamespaceLink to={resultLocation(result)} className={resultClassName}>
          {children}
        </NamespaceLink>
      </ResourceListRow>
    )
  return (
    <ResolvedResultLink result={result} search={search}>
      {children}
    </ResolvedResultLink>
  )
}
const resultClassName = 'flex min-w-0 items-start gap-3'
function ResolvedResultLink({
  result,
  search,
  children,
}: {
  result: SearchResult
  search: string
  children: ReactNode
}) {
  const actor = useAuthStore((state) => state.userId)
  const document = result.result_type === 'document'
  const context = useQuery({
    queryKey: [
      'wiki',
      document ? 'document-space' : 'space-projects',
      actor,
      document ? result.id : result.space_key,
    ],
    queryFn: ({ signal }) =>
      apiRequest<SpaceProjects>(
        document
          ? `/api/v1/documents/${encodeURIComponent(result.id)}/space-context`
          : `/api/v1/spaces/${encodeURIComponent(result.space_key)}/projects`,
        { signal },
      ),
  })
  const className = resultClassName
  if (context.isPending)
    return (
      <ResourceListRow asChild>
        <div className={className} aria-busy="true">
          {children}
          <span role="status">Разрешаем контекст…</span>
        </div>
      </ResourceListRow>
    )
  if (context.isError) {
    const forbidden = context.error instanceof ApiError && [401, 403].includes(context.error.status)
    return (
      <ResourceListRow asChild>
        <div className={className}>
          {!forbidden && children}
          <span role="alert">Контекст результата недоступен.</span>
        </div>
      </ResourceListRow>
    )
  }
  return (
    <ResourceListRow asChild interactive>
      <Link to={withNamespaceLocation(resultLocation(result), null, search)} className={className}>
        {children}
      </Link>
    </ResourceListRow>
  )
}
