import { useState } from 'react'
import { Link } from 'react-router'
import { ArrowRight, FilePlus2, FileText, GitBranch, Library, Search } from 'lucide-react'
import { usePhaseSummaries, useSpaces, useTaskSummaries, useWikiSearch } from '@/shared/api/hooks'
import { Button, EmptyState, ErrorState, LoadingState } from '@sdlc/ui/ui'
import { formatApiErrorForUser } from '@/shared/lib/api-error'
import { formatDateTime } from '@/shared/lib/wiki-format'
import { resolveSpaceKey } from '@/shared/lib/space-selection'

const selectClassName =
  'min-h-10 rounded-md border border-border bg-surface px-3 text-sm text-text-primary focus-visible:outline-2 focus-visible:outline-accent'

export function DashboardPage() {
  const [requestedSpaceKey, setRequestedSpaceKey] = useState('')
  const spacesQuery = useSpaces()
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
  const recentDocuments = (searchQuery.data?.results ?? [])
    .filter((result) => result.result_type === 'document')
    .slice(0, 4)
  const focusTasks = tasksQuery.data?.tasks ?? []
  const documentCount = spaces.reduce((sum, space) => sum + space.document_count, 0)
  const stats = [
    { label: 'Пространства', value: spaces.length, icon: Library },
    { label: 'Документы всего', value: documentCount, icon: FileText },
    {
      label: 'Задачи в пространстве',
      value: tasksQuery.isLoading ? '…' : tasksQuery.isError ? '—' : (tasksQuery.data?.total ?? 0),
      icon: FileText,
    },
    {
      label: 'Фазы в пространстве',
      value: phasesQuery.isLoading
        ? '…'
        : phasesQuery.isError
          ? '—'
          : (phasesQuery.data?.total ?? 0),
      icon: GitBranch,
    },
  ]

  return (
    <div className="space-y-5">
      <header className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold">Wiki</h1>
          <p className="mt-1 text-sm text-text-muted">Документы, задачи и фазы процесса.</p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button asChild size="sm" className="min-h-10">
            <Link to="/documents/new">
              <FilePlus2 className="h-4 w-4" />
              Новый документ
            </Link>
          </Button>
          <Button asChild size="sm" variant="secondary" className="min-h-10">
            <Link to="/search">
              <Search className="h-4 w-4" />
              Найти
            </Link>
          </Button>
        </div>
      </header>

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
              <select
                id="overview-space"
                className={`${selectClassName} max-w-64 min-w-0`}
                value={activeSpaceKey}
                onChange={(event) => setRequestedSpaceKey(event.target.value)}
                disabled={spacesQuery.isLoading || spaces.length === 0}
              >
                {spaces.length === 0 && <option value="">Нет пространств</option>}
                {spaces.map((space) => (
                  <option key={space.key} value={space.key}>
                    {space.name}
                  </option>
                ))}
              </select>
            </div>
            <Link
              to="/spaces"
              className="inline-flex min-h-10 items-center gap-1 text-sm text-accent hover:underline"
            >
              Все пространства <ArrowRight className="h-4 w-4" aria-hidden />
            </Link>
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
                {stats.map(({ label, value, icon: Icon }, index) => (
                  <div key={label} className="flex items-center gap-3">
                    <Icon className="h-5 w-5 shrink-0 text-accent" aria-hidden />
                    <div>
                      <div className="text-lg font-semibold">
                        {spacesQuery.isLoading && index < 2 ? '…' : value}
                      </div>
                      <div className="text-xs text-text-muted">{label}</div>
                    </div>
                  </div>
                ))}
              </section>
              {phasesQuery.isError && (
                <ErrorState
                  message={formatApiErrorForUser(phasesQuery.error, 'Не удалось загрузить фазы')}
                  onRetry={() => void phasesQuery.refetch()}
                />
              )}

              <div className="grid gap-6 lg:grid-cols-[1.5fr_1fr]">
                <section aria-labelledby="recent-documents" className="min-w-0">
                  <div className="mb-2 flex items-center justify-between gap-3">
                    <h2 id="recent-documents" className="text-base font-semibold">
                      Последние документы
                    </h2>
                    <Link
                      to="/search"
                      className="inline-flex min-h-10 items-center gap-1 text-sm text-accent hover:underline"
                    >
                      Все <ArrowRight className="h-4 w-4" aria-hidden />
                    </Link>
                  </div>
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
                    <ul className="divide-y divide-border border-y border-border">
                      {recentDocuments.map((document) => (
                        <li key={document.id}>
                          <Link
                            to={document.url}
                            className="flex min-h-14 items-center justify-between gap-3 py-2 hover:bg-surface-raised"
                          >
                            <span className="min-w-0">
                              <span className="block truncate text-sm font-medium text-text-primary">
                                {document.title}
                              </span>
                              <span className="block text-xs text-text-muted">
                                {formatDateTime(document.updated_at)}
                              </span>
                            </span>
                            <ArrowRight className="h-4 w-4 shrink-0 text-text-muted" aria-hidden />
                          </Link>
                        </li>
                      ))}
                    </ul>
                  )}
                </section>

                <section aria-labelledby="focus-tasks" className="min-w-0">
                  <div className="mb-2 flex items-center justify-between gap-3">
                    <h2 id="focus-tasks" className="text-base font-semibold">
                      Задачи в Wiki
                    </h2>
                    <Link
                      to={`/tasks?space=${encodeURIComponent(activeSpaceKey)}`}
                      className="inline-flex min-h-10 items-center gap-1 text-sm text-accent hover:underline"
                    >
                      Все <ArrowRight className="h-4 w-4" aria-hidden />
                    </Link>
                  </div>
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
                    <ul className="divide-y divide-border border-y border-border">
                      {focusTasks.map((task) => (
                        <li key={task.task_key}>
                          <Link
                            to={`/tasks/${task.task_key}?space=${encodeURIComponent(activeSpaceKey)}`}
                            className="flex min-h-14 items-center justify-between gap-3 py-2 hover:bg-surface-raised"
                          >
                            <span className="min-w-0">
                              <span className="block truncate text-sm font-medium text-text-primary">
                                {task.title || task.task_key}
                              </span>
                              <span className="block text-xs text-text-muted">
                                {task.task_key} · Документы: {task.document_count}
                              </span>
                            </span>
                            <ArrowRight className="h-4 w-4 shrink-0 text-text-muted" aria-hidden />
                          </Link>
                        </li>
                      ))}
                    </ul>
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
