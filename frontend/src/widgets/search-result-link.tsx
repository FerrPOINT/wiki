import type { ReactNode } from 'react'
import { Link, useLocation } from 'react-router'
import { useQuery } from '@tanstack/react-query'
import { NamespaceLink } from '@sdlc/ui/ui'
import { withNamespaceLocation } from '@sdlc/ui/lib'
import { apiRequest, ApiError } from '@/api/client'
import type { SearchResult } from '@/api/wiki'
import { useAuthStore } from '@/shared/auth/store'
import type { ResourceContext } from './namespace-context'

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
      <NamespaceLink to={result.url} className={resultClassName}>
        {children}
      </NamespaceLink>
    )
  return (
    <ResolvedResultLink result={result} search={search}>
      {children}
    </ResolvedResultLink>
  )
}
const resultClassName =
  'flex min-h-16 min-w-0 items-start gap-3 px-2 py-3 hover:bg-surface-raised focus-visible:outline-2 focus-visible:outline-accent'
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
      document ? 'document-context' : 'space-context',
      actor,
      document ? result.id : result.space_key,
    ],
    queryFn: ({ signal }) =>
      apiRequest<ResourceContext | null>(
        document
          ? `/api/v1/documents/${encodeURIComponent(result.id)}/namespace-context`
          : `/api/v1/spaces/${encodeURIComponent(result.space_key)}/namespace-context`,
        { signal },
      ),
  })
  const className = resultClassName
  if (context.isPending)
    return (
      <div className={className} aria-busy="true">
        {children}
        <span role="status">Разрешаем контекст…</span>
      </div>
    )
  if (context.isError) {
    const forbidden = context.error instanceof ApiError && [401, 403].includes(context.error.status)
    return (
      <div className={className}>
        {!forbidden && children}
        <span role="alert">Контекст результата недоступен.</span>
      </div>
    )
  }
  return (
    <Link
      to={withNamespaceLocation(result.url, context.data?.binding.namespace ?? null, search)}
      className={className}
    >
      {children}
    </Link>
  )
}
