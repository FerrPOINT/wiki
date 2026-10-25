import { useState, type FormEvent } from 'react'
import { Link } from 'react-router'
import {
  TextLink,
  SectionHeader,
  Card,
  CardContent,
  Select,
  Button,
  EmptyState,
  ErrorState,
  Label,
  LoadingState,
  ResourceList,
  ResourceListRow,
} from '@pdlc/ui/ui'
import { projectIdentity, withNamespaceLocation } from '@pdlc/ui/lib'
import {
  useSpaceProjects,
  useSpaceProjectMutation,
  useWorkspaceCatalog,
  workspacePath,
} from '@/api/workspaces'
import { formatApiErrorForUser } from '@/shared/lib/api-error'

export function SpaceProjectsEditor({
  spaceKey,
  canManage,
}: {
  spaceKey: string
  canManage: boolean
}) {
  const projects = useSpaceProjects(spaceKey)
  const catalog = useWorkspaceCatalog(canManage)
  const mutation = useSpaceProjectMutation(spaceKey)
  const [selected, setSelected] = useState('')
  const linked = new Set(
    projects.data?.projects.map((project) => projectIdentity(project.namespace)),
  )
  const available =
    catalog.data?.filter(
      (project) =>
        project.state === 'active' &&
        !project.stale &&
        !linked.has(projectIdentity(project.namespace)),
    ) ?? []
  function attach(event: FormEvent) {
    event.preventDefault()
    const project = available.find((project) => projectIdentity(project.namespace) === selected)
    if (project)
      mutation.mutate(
        { project: project.namespace, attach: true },
        { onSuccess: () => setSelected('') },
      )
  }
  return (
    <Card asChild>
      <section aria-label="Проекты пространства">
        <CardContent className="space-y-3">
          <SectionHeader title={<>Проекты пространства</>} />
          <p className="text-sm text-text-muted">
            Пространство может быть общим для нескольких проектов. Отвязка сохраняет документы и их
            историю.
          </p>
          {projects.isPending ? (
            <LoadingState message="Загружаем связи" />
          ) : projects.isError ? (
            <ErrorState
              message="Не удалось загрузить проекты пространства"
              onRetry={() => projects.refetch()}
            />
          ) : projects.data.projects.length ? (
            <ResourceList asChild>
              <ul>
                {projects.data.projects.map((project) => (
                  <ResourceListRow asChild key={projectIdentity(project.namespace)}>
                    <li className="flex min-w-0 flex-wrap items-center justify-between gap-3">
                      <div className="min-w-0 flex-1 break-words">
                        {project.name ? (
                          <TextLink asChild><Link
                            to={withNamespaceLocation(
                              workspacePath(project.namespace),
                              project.namespace,
                            )}
                          >
                            {project.name} · {project.project_key}
                          </Link></TextLink>
                        ) : (
                          <span>Проект {project.namespace.namespace_id}</span>
                        )}
                        {(!project.available || project.state === 'archived') && (
                          <p className="text-sm text-text-muted">
                            {project.state === 'archived'
                              ? 'Проект в архиве'
                              : 'Сведения о проекте недоступны'}
                          </p>
                        )}
                      </div>
                      {canManage && (
                        <Button
                          type="button"
                          variant="ghost"
                          disabled={mutation.isPending}
                          aria-label={`Отвязать проект ${project.name ?? project.namespace.namespace_id}`}
                          onClick={() =>
                            mutation.mutate({ project: project.namespace, attach: false })
                          }
                        >
                          Отвязать
                        </Button>
                      )}
                    </li>
                  </ResourceListRow>
                ))}
              </ul>
            </ResourceList>
          ) : (
            <EmptyState message="Самостоятельное пространство · проектов нет" />
          )}
          {canManage && !projects.isPending && !projects.isError && (
            <form onSubmit={attach} className="flex flex-wrap items-end gap-2">
              <div className="min-w-0 flex-1 space-y-1.5">
                <Label htmlFor={`${spaceKey}-project`}>Добавить проект</Label>
                <Select
                  id={`${spaceKey}-project`}
                  value={selected}
                  onValueChange={(nextValue) => setSelected(nextValue)}
                  disabled={mutation.isPending || !available.length}
                  className="w-full"
                  options={[
                    {
                      value: '',
                      label: String(
                        catalog.isPending
                          ? 'Загружаем проекты…'
                          : available.length
                            ? 'Выберите проект'
                            : 'Нет доступных проектов для добавления',
                      ),
                    },
                    ...(available.map((project) => ({
                      value: String(projectIdentity(project.namespace)),
                      label: [project.name, ' · ', project.project_key].join(''),
                    })) ?? []),
                  ]}
                />
              </div>
              <Button disabled={!selected || mutation.isPending}>Добавить</Button>
            </form>
          )}
          {canManage && catalog.isError && (
            <ErrorState message="Каталог проектов недоступен" onRetry={() => catalog.refetch()} />
          )}
          {mutation.isError && (
            <ErrorState
              message={formatApiErrorForUser(mutation.error, 'Не удалось изменить связь')}
            />
          )}
        </CardContent>
      </section>
    </Card>
  )
}
