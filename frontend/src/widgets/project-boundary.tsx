import { Select } from '@pdlc/ui/ui'
import { useEffect, type ReactNode } from 'react'
import { useLocation, useNavigate } from 'react-router'
import { parseNamespaceLocation, withNamespaceLocation } from '@pdlc/ui/lib'
import { useNamespaceContext } from './namespace-context'
import { projectResourceLabel } from './workspace-navigation'

/** Resolve space before mounting legacy pages; they cannot issue a foreign-space query. */
export function ProjectBoundary({ children }: { children: ReactNode }) {
  const context = useNamespaceContext(),
    location = useLocation(),
    navigate = useNavigate()
  const params = new URLSearchParams(location.search)
  const globalSearch = location.pathname === '/search' && params.get('search_scope') === 'all'
  const needsSpace =
    (!globalSearch && /^\/(tasks|phases|evidence|search)(?:\/|$)/.test(location.pathname)) ||
    /\/workspaces\/[^/]+\/[^/]+\/(documents|tasks|phases|evidence|search)$/.test(
      location.pathname,
    ) ||
    location.pathname === '/documents/new'
  const scoped = context.ref && needsSpace
  const document =
    /^\/documents\/[^/]+/.test(location.pathname) && location.pathname !== '/documents/new'
  const explicitSpace = needsSpace && params.has('space')
  const space = context.query.data?.resource_key
  const conflict = context.conflict
  const synchronizeSpace = Boolean(scoped && space && !params.has('space'))
  const synchronizeRef = Boolean(
    context.ref && !context.malformed && !parseNamespaceLocation(location.search),
  )
  let target: string | null = null
  if (
    !conflict &&
    !context.malformed &&
    (synchronizeSpace || synchronizeRef || context.clearProjectScope)
  ) {
    const next = new URLSearchParams(location.search)
    if (synchronizeSpace && space) next.set('space', space)
    if (context.all) next.set('project_scope', 'all')
    if (context.clearProjectScope) {
      next.delete('task_id')
      next.delete('tracker_instance_id')
    }
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
  if (scoped && context.workspace.data && context.workspace.data.resource_status !== 'connected')
    return <p role="status">{projectResourceLabel(context.workspace.data)}</p>
  if (((scoped || document || explicitSpace) && context.query.isPending) || target)
    return <p role="status">Разрешаем пространство проекта…</p>
  if ((scoped || document || explicitSpace) && context.query.isError)
    return <p role="alert">Пространство проекта недоступно. Другое пространство не выбрано.</p>
  if (location.pathname === '/documents/new' && context.query.data?.status === 'archived')
    return <p role="status">Архивное пространство доступно только для чтения.</p>
  if (scoped && !space && context.choices.length > 1)
    return (
      <section className="max-w-xl space-y-3">
        <h1 className="text-xl font-semibold">Выберите пространство проекта</h1>
        <label htmlFor="project-space">Пространство</label>
        <Select
          id="project-space"
          value=""
          className="w-full"
          onChange={(event) => {
            const next = new URLSearchParams(location.search)
            next.set('space', event.target.value)
            navigate(location.pathname + '?' + next + location.hash)
          }}
        >
          <option value="" disabled>
            Выберите пространство
          </option>
          {context.choices.map((choice) => (
            <option
              key={choice.id}
              value={choice.key}
              disabled={
                choice.resource_status !== 'connected' ||
                (location.pathname === '/documents/new' && choice.status === 'archived')
              }
            >
              {choice.name} · {choice.key}
              {choice.resource_status !== 'connected' ? ' · Недоступно' : ''}
              {choice.status === 'archived' ? ' · В архиве' : ''}
            </option>
          ))}
        </Select>
      </section>
    )
  return children
}
