import { useLocation } from 'react-router'
import {
  PageHeader,
  SectionHeader,
  NamespaceLink as Link,
  ProjectAvatar,
  ResourceList,
  ResourceListRow,
  LoadingState,
  ErrorState,
  Notice,
  EmptyState,
  TextLink,
  ResourceListTitle,
  ResourceListDescription,
} from '@pdlc/ui/ui'
import { projectIdentity, withNamespaceLocation } from '@pdlc/ui/lib'
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
          <TextLink asChild>
            <Link to={documentPath(node.id, search)}>{node.title}</Link>
          </TextLink>
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
  if (catalog.isPending) return <LoadingState message="Загружаем проекты…" />
  if (catalog.isError) return <ErrorState message="Каталог проектов недоступен." />
  return (
    <div className="space-y-5">
      <PageHeader title={'Проекты'} />
      <ResourceList asChild>
        <ul>
          {catalog.data.map((project) => (
            <li key={projectIdentity(project.namespace)}>
              <ResourceListRow asChild interactive>
                <Link
                  className="flex items-center gap-3"
                  to={withNamespaceLocation(
                    workspacePath(project.namespace) + '?project_scope=all',
                    project.namespace,
                  )}
                >
                  <ProjectAvatar projectKey={project.project_key} />
                  <ResourceListTitle asChild>
                    <span>
                      {project.name} · {project.project_key}
                    </span>
                  </ResourceListTitle>
                  <ResourceListDescription asChild>
                    <span>{projectResourceLabel(project)}</span>
                  </ResourceListDescription>
                </Link>
              </ResourceListRow>
            </li>
          ))}
        </ul>
      </ResourceList>
      {!catalog.data.length && <EmptyState message="Нет доступных проектов" />}
    </div>
  )
}
export function WorkspacePage({ documents = false }: { documents?: boolean }) {
  const context = useNamespaceContext(),
    location = useLocation()
  const project = context.workspace.data
  const resource = context.query.isError ? undefined : context.query.data
  const tree = useSpaceTree(resource?.resource_key ?? '', documents && !!resource)
  if (context.malformed) return <ErrorState message="Некорректный контекст проекта." />
  if (context.workspace.isPending) return <LoadingState message="Загружаем проект…" />
  if (documents && context.query.isError)
    return <ErrorState message="Документы проекта недоступны. Другое пространство не выбрано." />
  if (context.workspace.isError && documents && resource)
    return (
      <section className="space-y-3">
        <PageHeader title={'Документы проекта'} />
        <DossierMenu spaceKey={resource.resource_key} namespace={context.ref} />
        <Notice role="status" appearance="inline" tone="neutral">
          Сведения о проекте временно недоступны. Подтверждённое пространство сохранено.
        </Notice>
        {tree.isPending ? (
          <LoadingState message="Загружаем документы…" />
        ) : tree.isError ? (
          <ErrorState message="Документы недоступны." />
        ) : (
          <Documents nodes={tree.data.documents} search={location.search} />
        )}
      </section>
    )
  if (context.workspace.isError || !project)
    return <ErrorState message="Проект недоступен. Другой проект не выбран." />
  return (
    <div className="space-y-5">
      <PageHeader
        title={project.name}
        leading={<ProjectAvatar projectKey={project.project_key} />}
        context={
          <>
            {project.project_key}
            {project.state === 'archived' ? ' · В архиве' : ''}
            {project.stale ? ' · Метаданные устарели' : ''}
          </>
        }
      />
      {project.resource_status !== 'connected' ? (
        <div>
          <Notice role="status" appearance="inline" tone="neutral">
            {projectResourceLabel(project)}
          </Notice>
          {project.resource_status === 'not_connected' && (
            <TextLink asChild>
              <Link to="/spaces?project_scope=all">Связать пространство в каталоге Wiki</Link>
            </TextLink>
          )}
        </div>
      ) : documents ? (
        <section>
          <SectionHeader title={<>Документы</>} />
          {resource && (
            <DossierMenu spaceKey={resource.resource_key} namespace={project.namespace} />
          )}
          {tree.isPending ? (
            <LoadingState message="Загружаем документы…" />
          ) : tree.isError ? (
            <ErrorState message="Документы недоступны." />
          ) : tree.data?.documents.length ? (
            <Documents nodes={tree.data.documents} search={location.search} />
          ) : (
            <EmptyState message="Документов пока нет." />
          )}
        </section>
      ) : (
        <section className="space-y-3">
          <SectionHeader title={<>Пространства проекта</>} />
          <ResourceList asChild>
            <ul>
              {project.spaces.map((space) => (
                <li key={space.id}>
                  <ResourceListRow asChild interactive>
                    <Link
                      className="flex min-w-0 flex-wrap items-center justify-between gap-3"
                      to={withNamespaceLocation(
                        workspacePath(project.namespace, 'documents') +
                          '?space=' +
                          encodeURIComponent(space.key),
                        project.namespace,
                      )}
                    >
                      <ResourceListTitle asChild>
                        <span className="min-w-0 break-words">
                          {space.name} · {space.key}
                        </span>
                      </ResourceListTitle>
                      <ResourceListDescription asChild>
                        <span>{space.status === 'archived' ? 'Пространство в архиве' : 'Документы'}</span>
                      </ResourceListDescription>
                    </Link>
                  </ResourceListRow>
                </li>
              ))}
            </ul>
          </ResourceList>
          <TextLink asChild>
            <Link to="/spaces?project_scope=all">Управлять связями пространств</Link>
          </TextLink>
        </section>
      )}
      {resource?.status === 'active' && (
        <TextLink asChild>
          <Link
            to={withNamespaceLocation(
              `/documents/new?space=${encodeURIComponent(resource.resource_key)}`,
              project.namespace,
            )}
          >
            Создать документ
          </Link>
        </TextLink>
      )}
    </div>
  )
}
