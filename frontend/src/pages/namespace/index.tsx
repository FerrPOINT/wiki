import { NamespaceLink as Link } from '@sdlc/ui/ui'

import { useQuery } from '@tanstack/react-query'
import type { components } from '@/api/generated'
import { apiRequest } from '@/api/client'
import { withNamespaceLocation } from '@sdlc/ui/lib'
import { useNamespaceContext } from '@/widgets/namespace-context'
function taskIntent() {
  const source = new URLSearchParams(window.location.search)
  const target = new URLSearchParams()
  for (const key of ['task_id', 'tracker_instance_id']) {
    const value = source.get(key)
    if (value) target.set(key, value)
  }
  return target.toString()
}
type TreeNode = components['schemas']['SpaceTreeNodeResponse']
function Documents({
  nodes,
  context,
}: {
  nodes: TreeNode[]
  context: NonNullable<ReturnType<typeof useNamespaceContext>['ref']>
}) {
  return (
    <ul className="space-y-3">
      {nodes.map((document) => (
        <li key={document.id}>
          <Link
            className="text-accent hover:underline"
            to={withNamespaceLocation(`/documents/${document.id}?${taskIntent()}`, context)}
          >
            {document.title}
          </Link>
          {document.children.length > 0 && (
            <div className="ml-4 mt-3">
              <Documents nodes={document.children} context={context} />
            </div>
          )}
        </li>
      ))}
    </ul>
  )
}
export function NamespacePage() {
  const { ref, malformed, query } = useNamespaceContext()
  const resource = query.data
  const tree = useQuery({
    queryKey: [
      'namespace-documents',
      ref?.registry_instance_id,
      ref?.namespace_id,
      resource?.binding.resource.resource_id,
    ],
    enabled: Boolean(resource),
    queryFn: ({ signal }) =>
      apiRequest<components['schemas']['SpaceTreeResponse']>(
        `/api/v1/spaces/${encodeURIComponent(resource!.resource_key)}/tree`,
        { signal },
      ),
  })
  if (malformed)
    return (
      <p role="alert" className="text-danger">
        Некорректная ссылка на проект.
      </p>
    )
  if (!ref)
    return (
      <div className="space-y-3">
        <h1 className="text-xl font-semibold">Документы проекта</h1>
        <p>Выберите проект в верхней панели.</p>
        <Link className="text-accent" to="/spaces">
          Пространства Wiki
        </Link>
      </div>
    )
  if (query.isPending) return <p role="status">Разрешаем привязку Wiki…</p>
  if (query.isError || !resource)
    return (
      <p role="alert" className="text-danger">
        Привязка Wiki недоступна. Другое пространство не выбрано.
      </p>
    )
  return (
    <div className="space-y-5">
      <h1 className="text-xl font-semibold">{resource.label}</h1>
      <p className="text-sm text-text-muted">
        {resource.binding.state === 'archived'
          ? 'Проект в архиве. История доступна для чтения.'
          : 'Документы проекта'}
      </p>
      {resource.binding.state === 'active' && (
        <Link
          className="text-accent"
          to={withNamespaceLocation(
            `/documents/new?space=${encodeURIComponent(resource.resource_key)}&${taskIntent()}`,
            ref,
          )}
        >
          Создать документ
        </Link>
      )}
      {tree.isPending ? (
        <p role="status">Загружаем документы…</p>
      ) : tree.isError ? (
        <p role="alert" className="text-danger">
          Документы недоступны.
        </p>
      ) : tree.data?.documents.length === 0 ? (
        <p className="text-text-muted">Документов пока нет.</p>
      ) : (
        <Documents nodes={tree.data?.documents ?? []} context={ref} />
      )}
    </div>
  )
}
