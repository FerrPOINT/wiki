import { useActiveSpaces } from '@/widgets/active-spaces'
import { useState } from 'react'
import {
  PageHeader,
  SectionHeader,
  Metric,
  type MetricProps,
  Select,
  NamespaceLink as Link,
  ResourceList,
  ResourceListRow,
  Button,
  EmptyState,
  ErrorState,
  LoadingState,
} from '@pdlc/ui/ui'

import { ArrowRight, FilePlus2, FileText, GitBranch, Library, Search } from 'lucide-react'
import { usePhaseSummaries, useTaskSummaries, useWikiSearch } from '@/shared/api/hooks'
import { formatApiErrorForUser } from '@/shared/lib/api-error'
import { formatDateTime } from '@/shared/lib/wiki-format'
import { resolveSpaceKey } from '@/shared/lib/space-selection'
import { usePlatformReadiness } from '@pdlc/ui/platform'

export function DashboardPage() {
  const [requestedSpaceKey, setRequestedSpaceKey] = useState('')
  const spacesQuery = useActiveSpaces()
  const spaces = spacesQuery.data?.spaces ?? []
  const activeSpaceKey = spaces.some((space) => space.key === requestedSpaceKey)
    ? requestedSpaceKey
    : resolveSpaceKey(null, spaces)
  const activeSpace = spaces.find((space) => space.key === activeSpaceKey)
  const searchQuery = useWikiSearch(
    { space: activeSpaceKey || undefined, limit: 100 },
    Boolean(activeSpaceKey),
  )
  const tasksQuery = useTaskSummaries(activeSpaceKey, { limit: 4 })
  const phasesQuery = usePhaseSummaries(activeSpaceKey, { limit: 1 })
  usePlatformReadiness({
    pending:
      spacesQuery.isLoading ||
      searchQuery.isLoading ||
      tasksQuery.isLoading ||
      phasesQuery.isLoading,
    error:
      spacesQuery.isError || searchQuery.isError || tasksQuery.isError || phasesQuery.isError
        ? 'Не удалось загрузить дашборд Wiki.'
        : null,
  })
  const recentDocuments = (searchQuery.data?.results ?? [])
    .filter((result) => result.result_type === 'document')
    .slice(0, 4)
  const focusTasks = tasksQuery.data?.tasks ?? []
  const documentCount = spaces.reduce((sum, space) => sum + space.document_count, 0)
  const stats = [
    { label: 'Пространства', value: spaces.length, icon: <Library className="h-5 w-5" /> },
    { label: 'Документы всего', value: documentCount, icon: <FileText className="h-5 w-5" /> },
    {
      label: 'Задачи в пространстве',
      value: tasksQuery.data?.total ?? 0,
      icon: <FileText className="h-5 w-5" />,
      state: tasksQuery.isLoading
        ? { kind: 'loading' }
        : (
              tasksQuery.isError
                ? formatApiErrorForUser(tasksQuery.error, 'Не удалось загрузить задачи')
                : undefined
            )
          ? {
              kind: 'error',
              message: tasksQuery.isError
                ? formatApiErrorForUser(tasksQuery.error, 'Не удалось загрузить задачи')
                : undefined,
            }
          : { kind: 'ready' },
    },
    {
      label: 'Фазы в пространстве',
      value: phasesQuery.data?.total ?? 0,
      icon: <GitBranch className="h-5 w-5" />,
      state: phasesQuery.isLoading
        ? { kind: 'loading' }
        : (
              phasesQuery.isError
                ? formatApiErrorForUser(phasesQuery.error, 'Не удалось загрузить фазы')
                : undefined
            )
          ? {
              kind: 'error',
              message: phasesQuery.isError
                ? formatApiErrorForUser(phasesQuery.error, 'Не удалось загрузить фазы')
                : undefined,
              onRetry: phasesQuery.isError ? () => void phasesQuery.refetch() : undefined,
            }
          : { kind: 'ready' },
    },
  ] satisfies MetricProps[]

  return (
    <div className="space-y-5">
      <PageHeader
        title={'Wiki'}
        context={
          <>
            <p className="mt-1 text-sm text-text-muted">Документы, задачи и фазы процесса.</p>
          </>
        }
        actions={
          <>
            <Button asChild size="sm" className="md:hidden">
              <Link to="/documents/new">
                <FilePlus2 className="h-4 w-4" />
                Новый документ
              </Link>
            </Button>
            <Button asChild size="sm" variant="secondary">
              <Link to="/search">
                <Search className="h-4 w-4" />
                Найти
              </Link>
            </Button>
          </>
        }
      />

      {spacesQuery.isError ? (
        <ErrorState
          message={formatApiErrorForUser(spacesQuery.error, 'Не удалось загрузить пространства')}
          onRetry={() => void spacesQuery.refetch()}
        />
      ) : (
        <>
          <div className="flex flex-wrap items-center justify-between gap-3 border-y border-border py-3">
            <div className="flex min-w-0 items-center gap-3">
              <label htmlFor="overview-space" className="shrink-0 text-sm text-text-muted">
                Пространство
              </label>
              <Select
                id="overview-space"
                className="w-auto max-w-64 min-w-0"
                value={activeSpaceKey}
                onValueChange={(nextValue) => setRequestedSpaceKey(nextValue)}
                disabled={spacesQuery.isLoading || spaces.length === 0}
                options={[
                  ...(spaces.length === 0 ? [{ value: '', label: 'Нет пространств' }] : []),
                  ...(spaces.map((space) => ({
                    value: String(space.key),
                    label: String(space.name),
                  })) ?? []),
                ]}
              />
            </div>
            <Button asChild variant="link">
              <Link to="/spaces">
                Все пространства <ArrowRight className="h-4 w-4" aria-hidden />
              </Link>
            </Button>
          </div>

          {spacesQuery.isLoading ? (
            <LoadingState message="Загружаем пространства" />
          ) : spaces.length === 0 ? (
            <EmptyState message="Пространства пока не созданы" />
          ) : (
            <>
              <section
                aria-label="Показатели Wiki"
                className="grid grid-cols-2 gap-x-5 gap-y-3 border-b border-border pb-4 lg:grid-cols-4"
              >
                {stats.map((metric) => (
                  <Metric key={metric.label} {...metric} appearance="inline" />
                ))}
              </section>

              <div className="grid gap-6 lg:grid-cols-[1.5fr_1fr]">
                <section aria-labelledby="recent-documents" className="min-w-0">
                  <SectionHeader
                    headingId="recent-documents"
                    className="mb-2"
                    title={<>Последние документы</>}
                    actions={
                      <>
                        <Button asChild variant="link">
                          <Link to="/search">
                            Все <ArrowRight className="h-4 w-4" aria-hidden />
                          </Link>
                        </Button>
                      </>
                    }
                  />
                  {searchQuery.isLoading ? (
                    <LoadingState message="Загружаем документы" />
                  ) : searchQuery.isError ? (
                    <ErrorState
                      message={formatApiErrorForUser(
                        searchQuery.error,
                        'Не удалось загрузить документы',
                      )}
                      onRetry={() => void searchQuery.refetch()}
                    />
                  ) : recentDocuments.length === 0 ? (
                    <EmptyState
                      message={
                        activeSpace?.document_count
                          ? 'Недавние документы не найдены'
                          : 'В этом пространстве пока нет документов'
                      }
                      action={
                        <Button asChild size="sm">
                          <Link to={activeSpace?.document_count ? '/search' : '/documents/new'}>
                            {activeSpace?.document_count ? 'Открыть поиск' : 'Создать документ'}
                          </Link>
                        </Button>
                      }
                    />
                  ) : (
                    <ResourceList asChild>
                      <ul>
                        {recentDocuments.map((document) => (
                          <li key={document.id}>
                            <ResourceListRow asChild interactive>
                              <Link
                                to={document.url}
                                className="flex items-center justify-between gap-3"
                              >
                                <span className="min-w-0">
                                  <span className="block truncate text-sm font-medium text-text-primary">
                                    {document.title}
                                  </span>
                                  <span className="block text-xs text-text-muted">
                                    {formatDateTime(document.updated_at)}
                                  </span>
                                </span>
                                <ArrowRight
                                  className="h-4 w-4 shrink-0 text-text-muted"
                                  aria-hidden
                                />
                              </Link>
                            </ResourceListRow>
                          </li>
                        ))}
                      </ul>
                    </ResourceList>
                  )}
                </section>

                <section aria-labelledby="focus-tasks" className="min-w-0">
                  <SectionHeader
                    headingId="focus-tasks"
                    className="mb-2"
                    title={<>Задачи в Wiki</>}
                    actions={
                      <>
                        <Button asChild variant="link">
                          <Link to={`/tasks?space=${encodeURIComponent(activeSpaceKey)}`}>
                            Все <ArrowRight className="h-4 w-4" aria-hidden />
                          </Link>
                        </Button>
                      </>
                    }
                  />
                  {tasksQuery.isLoading ? (
                    <LoadingState message="Загружаем задачи" />
                  ) : tasksQuery.isError ? (
                    <ErrorState
                      message={formatApiErrorForUser(
                        tasksQuery.error,
                        'Не удалось загрузить задачи',
                      )}
                      onRetry={() => void tasksQuery.refetch()}
                    />
                  ) : focusTasks.length === 0 ? (
                    <EmptyState message="Документы ещё не связаны с задачами" />
                  ) : (
                    <ResourceList asChild>
                      <ul>
                        {focusTasks.map((task) => (
                          <li key={task.task_key}>
                            <ResourceListRow asChild interactive>
                              <Link
                                to={`/tasks/${task.task_key}?space=${encodeURIComponent(activeSpaceKey)}`}
                                className="flex items-center justify-between gap-3"
                              >
                                <span className="min-w-0">
                                  <span className="block truncate text-sm font-medium text-text-primary">
                                    {task.title || task.task_key}
                                  </span>
                                  <span className="block text-xs text-text-muted">
                                    {task.task_key} · Документы: {task.document_count}
                                  </span>
                                </span>
                                <ArrowRight
                                  className="h-4 w-4 shrink-0 text-text-muted"
                                  aria-hidden
                                />
                              </Link>
                            </ResourceListRow>
                          </li>
                        ))}
                      </ul>
                    </ResourceList>
                  )}
                </section>
              </div>
            </>
          )}
        </>
      )}
    </div>
  )
}
