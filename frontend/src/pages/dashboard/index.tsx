import { Link } from 'react-router'
import { CheckCircle2, FilePlus2, FileText, GitBranch, Library, Search } from 'lucide-react'
import {
  defaultSpaceKey,
  useEvidence,
  usePhases,
  useSpaces,
  useTasks,
  useWikiSearch,
} from '@/shared/api/hooks'
import { EmptyState, ErrorState, LoadingState } from '@sdlc/ui/ui'
import { Button } from '@sdlc/ui/ui'
import { Card, CardContent, CardHeader, CardTitle } from '@sdlc/ui/ui'
import { formatFirstApiErrorForUser } from '@/shared/lib/api-error'
import { formatDateTime } from '@/shared/lib/wiki-format'

export function DashboardPage() {
  const spacesQuery = useSpaces()
  const searchQuery = useWikiSearch({ space: defaultSpaceKey, limit: 6 })
  const tasksQuery = useTasks(defaultSpaceKey)
  const phasesQuery = usePhases(defaultSpaceKey)
  const evidenceQuery = useEvidence({ space: defaultSpaceKey, limit: 6 })

  const spaces = spacesQuery.data?.spaces ?? []
  const results = searchQuery.data?.results ?? []
  const tasks = tasksQuery.data?.tasks ?? []
  const phases = phasesQuery.data?.phases ?? []
  const evidence = evidenceQuery.data?.evidence ?? []
  const recentDocuments = results.filter((result) => result.result_type === 'document').slice(0, 3)
  const focusTasks = tasks.slice(0, 3)
  const documentCount = spaces.reduce((sum, space) => sum + space.document_count, 0)
  const isLoading =
    spacesQuery.isLoading || searchQuery.isLoading || tasksQuery.isLoading || phasesQuery.isLoading
  const isError =
    spacesQuery.isError || searchQuery.isError || tasksQuery.isError || phasesQuery.isError
  const overviewError = formatFirstApiErrorForUser(
    [spacesQuery.error, searchQuery.error, tasksQuery.error, phasesQuery.error],
    'Не удалось загрузить обзор Wiki',
  )

  function retryOverview() {
    void spacesQuery.refetch()
    void searchQuery.refetch()
    void tasksQuery.refetch()
    void phasesQuery.refetch()
  }

  return (
    <div className="space-y-6">
      <section className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold">Wiki</h1>
          <p className="mt-1 max-w-3xl text-sm text-text-muted">
            База знаний для документов по задачам SDLC и фазам выполненного процесса.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button asChild size="sm">
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
        </div>
      </section>

      <section className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Пространства</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-3">
              <Library className="h-5 w-5 text-accent" />
              <span className="text-2xl font-semibold">{spaces.length}</span>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Документы</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-3">
              <FileText className="h-5 w-5 text-accent" />
              <span className="text-2xl font-semibold">{documentCount}</span>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Фазы</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-3">
              <GitBranch className="h-5 w-5 text-accent" />
              <span className="text-2xl font-semibold">{phases.length}</span>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Материалы</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-3">
              <CheckCircle2 className="h-5 w-5 text-success" />
              <span className="text-2xl font-semibold">{evidence.length}</span>
            </div>
          </CardContent>
        </Card>
      </section>

      <section className="grid gap-4 lg:grid-cols-[1.5fr_1fr]">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Последние документы</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            {isLoading && <LoadingState message="Загружаем документы" />}
            {isError && <ErrorState message={overviewError} onRetry={retryOverview} />}
            {!isLoading && !isError && recentDocuments.length === 0 && (
              <EmptyState
                message="В этом пространстве пока нет документов"
                action={
                  <Button asChild size="sm">
                    <Link to="/documents/new">Создать документ</Link>
                  </Button>
                }
              />
            )}
            {!isLoading &&
              !isError &&
              recentDocuments.map((document) => (
                <Link
                  key={document.id}
                  to={document.url}
                  className="flex flex-col gap-2 rounded-md border border-border p-3 hover:bg-surface-raised sm:flex-row sm:items-center sm:justify-between"
                >
                  <span>
                    <span className="block text-sm font-medium text-text-primary">
                      {document.title}
                    </span>
                    <span className="mt-1 block text-xs text-text-muted">
                      {document.space_key} · {formatDateTime(document.updated_at)}
                    </span>
                  </span>
                  <span className="w-fit rounded bg-surface-raised px-2 py-1 text-xs text-text-secondary">
                    документ
                  </span>
                </Link>
              ))}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">Задачи в Wiki</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            {isLoading && <LoadingState message="Загружаем задачи" />}
            {isError && <ErrorState message={overviewError} onRetry={retryOverview} />}
            {!isLoading && !isError && focusTasks.length === 0 && (
              <EmptyState message="Документы ещё не связаны с задачами" />
            )}
            {!isLoading &&
              !isError &&
              focusTasks.map((task) => (
                <Link
                  key={task.task_key}
                  to={`/tasks/${task.task_key}`}
                  className="block rounded-md border border-border p-3 hover:bg-surface-raised"
                >
                  <div className="text-sm font-medium text-text-primary">{task.task_key}</div>
                  <div className="mt-1 text-xs text-text-muted">
                    Документы: {task.document_count} · Материалы: {task.evidence_count}
                  </div>
                </Link>
              ))}
          </CardContent>
        </Card>
      </section>
    </div>
  )
}
