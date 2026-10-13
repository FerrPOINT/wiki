import { useEffect, type ReactNode } from 'react'
import { useLocation, useNavigate } from 'react-router'
import { parseNamespaceLocation, withNamespaceLocation } from '@sdlc/ui/lib'
import { useNamespaceContext } from './namespace-context'

/** Resolve space before mounting legacy pages; they cannot issue a foreign-space query. */
export function ProjectBoundary({ children }: { children: ReactNode }) {
  const context = useNamespaceContext(),
    location = useLocation(),
    navigate = useNavigate()
  const needsSpace =
    /^\/(tasks|phases|evidence|search)(?:\/|$)/.test(location.pathname) ||
    /\/workspaces\/[^/]+\/[^/]+\/(tasks|phases|evidence|search)$/.test(location.pathname) ||
    location.pathname === '/documents/new'
  const scoped = context.ref && needsSpace
  const document =
    /^\/documents\/[^/]+/.test(location.pathname) && location.pathname !== '/documents/new'
  const params = new URLSearchParams(location.search)
  const explicitSpace = needsSpace && params.has('space')
  const space = context.query.data?.resource_key
  const conflict = Boolean(scoped && space && params.has('space') && params.get('space') !== space)
  const synchronizeSpace = Boolean(scoped && space && !params.has('space'))
  const synchronizeRef = Boolean(
    context.ref && !context.malformed && !parseNamespaceLocation(location.search),
  )
  let target: string | null = null
  if (!conflict && !context.malformed && (synchronizeSpace || synchronizeRef)) {
    const next = new URLSearchParams(location.search)
    if (synchronizeSpace && space) next.set('space', space)
    if (context.all) next.set('project_scope', 'all')
    target = withNamespaceLocation(location.pathname + '?' + next + location.hash, context.ref)
  }
  useEffect(() => {
    if (target) navigate(target, { replace: true })
  }, [target, navigate])
  if (import.meta.env.VITE_NAMESPACE_ENABLED !== 'true') return children
  if (context.malformed || conflict)
    return (
      <p role="alert">
        Некорректный или противоречивый контекст проекта. Другое пространство не выбрано.
      </p>
    )
  if (((scoped || document || explicitSpace) && context.query.isPending) || target)
    return <p role="status">Разрешаем пространство проекта…</p>
  if ((scoped || document || explicitSpace) && context.query.isError)
    return <p role="alert">Пространство проекта недоступно. Другое пространство не выбрано.</p>
  return children
}
