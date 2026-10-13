import { useLocation } from 'react-router'
import { NamespaceLink as Link, ProjectAvatar, usePlatformServices } from '@sdlc/ui/ui'
import { projectIdentity, withNamespaceLocation } from '@sdlc/ui/lib'
import { documentPath, useWorkspaceCatalog, workspacePath } from '@/api/workspaces'
import { useNamespaceContext } from '@/widgets/namespace-context'
import { projectResourceLabel } from '@/widgets/workspace-navigation'
import { useSpaceTree } from '@/shared/api/hooks'
import type { SpaceTreeNode } from '@/api/wiki'
import { DossierMenu } from '@/widgets/dossier-menu'

function Documents({ nodes, search }: { nodes: SpaceTreeNode[]; search: string }) {
  return (
    <ul className="space-y-2">
      {nodes.map((node) => (
        <li key={node.id}>
          <Link className="text-accent" to={documentPath(node.id, search)}>
            {node.title}
          </Link>
          {node.children.length > 0 && (
            <div className="ml-3">
              <Documents nodes={node.children} search={search} />
            </div>
          )}
        </li>
      ))}
    </ul>
  )
}
export function ProjectsPage() {
  const catalog = useWorkspaceCatalog()
  if (catalog.isPending) return <p role="status">Загружаем проекты…</p>
  if (catalog.isError) return <p role="alert">Каталог проектов недоступен.</p>
  return (
    <div className="space-y-5">
      <h1 className="text-2xl font-semibold">Проекты</h1>
      <ul className="divide-y divide-border">
        {catalog.data.map((project) => (
          <li key={projectIdentity(project.namespace)}>
            <Link
              className="flex items-center gap-3 p-3 hover:bg-surface-raised"
              to={withNamespaceLocation(
                workspacePath(project.namespace) + '?project_scope=all',
                project.namespace,
              )}
            >
              <ProjectAvatar projectKey={project.project_key} />
              <span>
                {project.name} · {project.project_key}
              </span>
              <span className="text-sm text-text-muted">{projectResourceLabel(project)}</span>
            </Link>
          </li>
        ))}
      </ul>
      {!catalog.data.length && <p>Нет доступных проектов</p>}
    </div>
  )
}
export function WorkspacePage({ documents = false }: { documents?: boolean }) {
  const context = useNamespaceContext(),
    { services } = usePlatformServices(),
    location = useLocation()
  const project = context.workspace.data
  const resource = context.query.isError ? undefined : context.query.data
  const tree = useSpaceTree(resource?.resource_key ?? '', documents && !!resource)
  if (context.malformed) return <p role="alert">Некорректный контекст проекта.</p>
  if (context.workspace.isPending) return <p role="status">Загружаем проект…</p>
  if (documents && context.query.isError)
    return <p role="alert">Документы проекта недоступны. Другое пространство не выбрано.</p>
  if (context.workspace.isError && documents && resource)
    return (
      <section className="space-y-3">
        <h1 className="text-2xl font-semibold">Документы проекта</h1>
        <DossierMenu spaceKey={resource.resource_key} namespace={context.ref} />
        <p role="status">
          Сведения о проекте временно недоступны. Подтверждённое пространство сохранено.
        </p>
        {tree.isPending ? (
          <p role="status">Загружаем документы…</p>
        ) : tree.isError ? (
          <p role="alert">Документы недоступны.</p>
        ) : (
          <Documents nodes={tree.data.documents} search={location.search} />
        )}
      </section>
    )
  if (context.workspace.isError || !project)
    return <p role="alert">Проект недоступен. Другой проект не выбран.</p>
  const admin = services.find((service) => service.key === 'admin-panel')?.ui_url
  return (
    <div className="space-y-5">
      <h1 className="flex items-center gap-3 text-2xl font-semibold">
        <ProjectAvatar projectKey={project.project_key} />
        {project.name}
      </h1>
      <p>
        {project.project_key}
        {project.state === 'archived' ? ' · В архиве' : ''}
        {project.stale ? ' · Метаданные устарели' : ''}
      </p>
      {project.resource_status !== 'connected' ? (
        <div>
          <p role="status">{projectResourceLabel(project)}</p>
          {project.resource_status === 'not_connected' && admin && (
            <a
              className="text-accent"
              href={withNamespaceLocation(
                admin + `/namespaces/${project.namespace.namespace_id}`,
                project.namespace,
              )}
            >
              Подключить Wiki в Admin
            </a>
          )}
        </div>
      ) : documents ? (
        <section>
          <h2 className="text-lg font-semibold">Документы</h2>
          {resource && (
            <DossierMenu spaceKey={resource.resource_key} namespace={project.namespace} />
          )}
          {tree.isPending ? (
            <p role="status">Загружаем документы…</p>
          ) : tree.isError ? (
            <p role="alert">Документы недоступны.</p>
          ) : tree.data?.documents.length ? (
            <Documents nodes={tree.data.documents} search={location.search} />
          ) : (
            <p>Документов пока нет.</p>
          )}
        </section>
      ) : (
        <Link
          className="text-accent"
          to={withNamespaceLocation(
            workspacePath(project.namespace, 'documents') + location.search,
            project.namespace,
          )}
        >
          Открыть документы проекта
        </Link>
      )}
      {project.resource_status === 'connected' &&
        project.state === 'active' &&
        project.binding?.state === 'active' && (
          <Link
            className="text-accent"
            to={withNamespaceLocation(
              `/documents/new?space=${encodeURIComponent(project.space_key!)}`,
              project.namespace,
            )}
          >
            Создать документ
          </Link>
        )}
    </div>
  )
}
