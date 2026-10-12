import type { components } from '@/api/generated'
import { useParams } from 'react-router'
import { useQuery } from '@tanstack/react-query'
import { NamespaceLink as Link, usePlatformServices } from '@sdlc/ui/ui'
import { withNamespaceLocation } from '@sdlc/ui/lib'
import { apiRequest } from '@/api/client'
import { useNamespaceContext } from '@/widgets/namespace-context'

type RevisionLink = components['schemas']['TaskRevisionLinkResponse']
export function ManagedTaskPage() {
  const { trackerInstance = '', taskId = '' } = useParams()
  const { ref, malformed } = useNamespaceContext()
  const { services } = usePlatformServices()
  const query = useQuery({
    queryKey: [
      'managed-task',
      ref?.registry_instance_id,
      ref?.namespace_id,
      trackerInstance,
      taskId,
    ],
    queryFn: ({ signal }) =>
      apiRequest<RevisionLink[]>(`/api/v1/managed-task-links/${trackerInstance}/${taskId}`, {
        signal,
      }),
  })
  if (query.isPending) return <p role="status">Загружаем досье задачи…</p>
  if (
    malformed ||
    query.isError ||
    !query.data ||
    (ref &&
      query.data.some(
        (link) =>
          link.namespace.namespace_id !== ref.namespace_id ||
          link.namespace.registry_instance_id !== ref.registry_instance_id,
      ))
  )
    return (
      <p role="alert" className="text-danger">
        Досье недоступно в выбранном проекте.
      </p>
    )
  const tracker = services.find((service) => service.key === 'task-tracker')?.ui_url
  return (
    <div className="space-y-5">
      <h1 className="text-xl font-semibold">
        Документы задачи {query.data[0]?.task_key ?? taskId}
      </h1>
      {tracker && (
        <a className="text-accent" href={withNamespaceLocation(`${tracker}/issues/${taskId}`, ref)}>
          Открыть задачу в Tracker
        </a>
      )}
      <ul className="space-y-3">
        {query.data.map((link) => (
          <li key={link.revision_id}>
            <Link
              className="text-accent"
              to={withNamespaceLocation(
                `/documents/${link.document_id}/revisions/${link.revision_id}`,
                ref,
              )}
            >
              {link.title} · ревизия {link.version}
            </Link>
          </li>
        ))}
        {query.data.length === 0 && (
          <li className="text-text-muted">Сохранённых связей с ревизиями пока нет.</li>
        )}
      </ul>
    </div>
  )
}
