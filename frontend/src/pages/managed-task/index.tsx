import {
  PageHeader,
  ResourceList,
  ResourceListRow,
  LoadingState,
  usePlatformServices,
  ErrorState,
  TextLink,
} from '@pdlc/ui/ui'
import type { components } from '@/api/generated'
import { Link, useLocation, useParams } from 'react-router'
import { useQuery } from '@tanstack/react-query'

import { withNamespaceLocation } from '@pdlc/ui/lib'
import { apiRequest } from '@/api/client'
import { useNamespaceContext } from '@/widgets/namespace-context'

type RevisionLink = components['schemas']['TaskRevisionLinkResponse']
export function ManagedTaskPage() {
  const { search } = useLocation()
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
  if (query.isPending) return <LoadingState message="Загружаем досье задачи…" />
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
    return <ErrorState message="Досье недоступно в выбранном проекте." />
  const tracker = services.find((service) => service.key === 'task-tracker')?.ui_url
  return (
    <div className="space-y-5">
      <PageHeader title={`Документы задачи ${query.data[0]?.task_key ?? taskId}`} />
      {tracker && (
        <TextLink href={withNamespaceLocation(`${tracker}/issues/${taskId}`, ref, search)}>
          Открыть задачу в Tracker
        </TextLink>
      )}
      <ResourceList asChild>
        <ul>
          {query.data.map((link) => (
            <ResourceListRow asChild key={link.revision_id}>
              <li>
                <TextLink asChild><Link
                  to={withNamespaceLocation(
                    `/documents/${link.document_id}/revisions/${link.revision_id}`,
                    null,
                  )}
                >
                  {link.title} · ревизия {link.version}
                </Link></TextLink>
              </li>
            </ResourceListRow>
          ))}
          {query.data.length === 0 && (
            <ResourceListRow asChild>
              <li className="text-text-muted">Сохранённых связей с ревизиями пока нет.</li>
            </ResourceListRow>
          )}
        </ul>
      </ResourceList>
    </div>
  )
}
