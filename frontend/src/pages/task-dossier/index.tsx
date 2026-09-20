import { type FormEvent, useEffect, useRef, useState } from 'react'
import { Link, useParams, useSearchParams } from 'react-router'
import { FileCheck2, FileText, GitBranch, Link2, Search } from 'lucide-react'
import {
  defaultSpaceKey,
  useLinkTaskDocument,
  useSpaces,
  useTask,
  useTaskSummaries,
} from '@/shared/api/hooks'
import { EmptyState, ErrorState, LoadingState } from '@sdlc/ui/ui'
import { Button } from '@sdlc/ui/ui'
import { Input } from '@sdlc/ui/ui'
import { Label } from '@sdlc/ui/ui'
import { formatApiErrorForUser } from '@/shared/lib/api-error'
import { resolveSpaceKey } from '@/shared/lib/space-selection'
import {
  formatDateTime,
  formatDocumentStatus,
  formatDocumentType,
  formatEvidenceType,
} from '@/shared/lib/wiki-format'

const pageSize = 12

const selectClassName =
  'flex min-h-10 w-full rounded-md border border-border-strong bg-surface px-3 py-1 text-sm text-text-primary shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent disabled:cursor-not-allowed disabled:opacity-50'

type SpaceOption = {
  key: string
  name: string
}

function useSelectedSpaceKey() {
  const [searchParams, setSearchParams] = useSearchParams()
  const spacesQuery = useSpaces()
  const selectedSpaceKey = resolveSpaceKey(
    searchParams.get('space'),
    spacesQuery.data?.spaces ?? [],
  )

  function setSelectedSpaceKey(spaceKey: string) {
    const normalized = spaceKey.trim().toUpperCase()
    const nextParams = new URLSearchParams(searchParams)
    if (normalized) nextParams.set('space', normalized)
    else nextParams.delete('space')
    setSearchParams(nextParams, { replace: true })
  }

  return [selectedSpaceKey, setSelectedSpaceKey, spacesQuery] as const
}

function scopedPath(path: string, spaceKey: string): string {
  return spaceKey === defaultSpaceKey ? path : `${path}?space=${encodeURIComponent(spaceKey)}`
}

function evidenceTaskPath(spaceKey: string, taskKey: string, evidenceId: string): string {
  const params = new URLSearchParams({ space: spaceKey, task_key: taskKey, id: evidenceId })
  return `/evidence?${params.toString()}`
}

function SpaceSelector({
  value,
  onChange,
  disabled = false,
}: {
  value: string
  onChange: (spaceKey: string) => void
  disabled?: boolean
}) {
  const spacesQuery = useSpaces()
  const spaces: SpaceOption[] = spacesQuery.data?.spaces ?? []
  const hasSelected = spaces.some((space) => space.key === value)
  const options = value && !hasSelected ? [{ key: value, name: value }, ...spaces] : spaces

  return (
    <div className="w-full space-y-1.5 sm:w-64">
      <Label htmlFor="task-space-selector">Пространство</Label>
      <select
        id="task-space-selector"
        className={selectClassName}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        disabled={disabled || spacesQuery.isLoading || spacesQuery.isError || options.length === 0}
      >
        {options.length === 0 && <option value="">Нет пространств</option>}
        {options.map((space) => (
          <option key={space.key} value={space.key}>
            {space.name ? `${space.key} · ${space.name}` : space.key}
          </option>
        ))}
      </select>
    </div>
  )
}

function TaskCatalog({ spaceKey }: { spaceKey: string }) {
  const [search, setSearch] = useState('')
  const [appliedSearch, setAppliedSearch] = useState('')
  const [cursors, setCursors] = useState<(string | null)[]>([null])
  const listRef = useRef<HTMLElement>(null)
  const cursor = cursors[cursors.length - 1] ?? undefined
  const tasksQuery = useTaskSummaries(spaceKey, {
    limit: pageSize,
    cursor,
    q: appliedSearch || undefined,
  })
  const tasks = tasksQuery.data?.tasks ?? []
  const nextCursor = tasksQuery.data?.next_cursor

  useEffect(() => {
    if (search.trim() === appliedSearch) return
    const timeout = window.setTimeout(() => {
      setAppliedSearch(search.trim())
      setCursors([null])
    }, 300)
    return () => window.clearTimeout(timeout)
  }, [search, appliedSearch])

  function changePage(nextCursors: (string | null)[]) {
    setCursors(nextCursors)
    listRef.current?.scrollIntoView?.({ block: 'start' })
  }

  return (
    <section ref={listRef} aria-label="Список задач" className="scroll-mt-16 space-y-3">
      <div className="relative max-w-md">
        <Search
          className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-text-muted"
          aria-hidden
        />
        <Input
          type="search"
          aria-label="Найти задачу"
          className="min-h-10 pl-9"
          placeholder="Ключ, название или документ"
          value={search}
          onChange={(event) => setSearch(event.target.value)}
        />
      </div>

      {tasksQuery.isLoading && <LoadingState message="Загружаем задачи" />}
      {tasksQuery.isError && (
        <ErrorState
          message={formatApiErrorForUser(tasksQuery.error, 'Не удалось загрузить задачи')}
          onRetry={() => tasksQuery.refetch()}
        />
      )}
      {!tasksQuery.isLoading && !tasksQuery.isError && tasks.length === 0 && (
        <EmptyState
          message={
            cursors.length > 1
              ? 'На этой странице задач больше нет'
              : appliedSearch
                ? 'По запросу задачи не найдены'
                : 'Документы ещё не связаны с задачами'
          }
        />
      )}
      {!tasksQuery.isLoading && !tasksQuery.isError && tasks.length > 0 && (
        <>
          <p role="status" className="text-xs text-text-muted">
            Задачи: {tasks.length} из {tasksQuery.data?.total}
          </p>
          <ul className="divide-y divide-border border-y border-border">
            {tasks.map((task) => (
              <li key={task.task_key}>
                <Link
                  to={scopedPath(`/tasks/${task.task_key}`, spaceKey)}
                  className="flex min-h-16 min-w-0 flex-wrap items-center justify-between gap-x-4 gap-y-1 px-2 py-3 hover:bg-surface-raised focus-visible:outline-2 focus-visible:outline-accent"
                >
                  <span className="min-w-0">
                    <span className="block break-words text-sm font-medium text-text-primary">
                      {task.task_key}
                    </span>
                    {task.title && (
                      <span className="block break-words text-sm text-text-secondary">
                        {task.title}
                      </span>
                    )}
                  </span>
                  <span className="shrink-0 text-xs text-text-muted">
                    Документы: {task.document_count} · Материалы: {task.evidence_count}
                  </span>
                </Link>
              </li>
            ))}
          </ul>
        </>
      )}
      {(cursors.length > 1 || nextCursor) && (
        <nav aria-label="Страницы задач" className="flex items-center justify-end gap-2">
          <Button
            type="button"
            size="sm"
            variant="outline"
            className="min-h-10"
            disabled={cursors.length === 1 || tasksQuery.isLoading || tasksQuery.isFetching}
            onClick={() => changePage(cursors.slice(0, -1))}
          >
            Назад
          </Button>
          <span className="text-sm text-text-muted">Страница {cursors.length}</span>
          <Button
            type="button"
            size="sm"
            variant="outline"
            className="min-h-10"
            disabled={
              !nextCursor || tasksQuery.isLoading || tasksQuery.isFetching || tasksQuery.isError
            }
            onClick={() => nextCursor && changePage([...cursors, nextCursor])}
          >
            Далее
          </Button>
        </nav>
      )}
    </section>
  )
}

export function TaskDossiersPage() {
  const [selectedSpaceKey, setSelectedSpaceKey, spacesQuery] = useSelectedSpaceKey()

  return (
    <div className="space-y-5">
      <section className="flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold">Задачи</h1>
          <p className="mt-1 max-w-3xl text-sm text-text-muted">
            Wiki собирает документы и материалы вокруг внешнего ключа задачи, но не владеет её
            статусом в трекере.
          </p>
        </div>
        <SpaceSelector value={selectedSpaceKey} onChange={setSelectedSpaceKey} />
      </section>

      {spacesQuery.isLoading && <LoadingState message="Загружаем пространства" />}
      {spacesQuery.isError && (
        <ErrorState
          message={formatApiErrorForUser(spacesQuery.error, 'Не удалось загрузить пространства')}
          onRetry={() => spacesQuery.refetch()}
        />
      )}
      {!spacesQuery.isLoading && !spacesQuery.isError && !selectedSpaceKey && (
        <EmptyState message="Сначала создайте пространство" />
      )}
      {selectedSpaceKey && <TaskCatalog key={selectedSpaceKey} spaceKey={selectedSpaceKey} />}
    </div>
  )
}

export function TaskDossierPage() {
  const { taskKey = 'BASE-42' } = useParams()
  const [selectedSpaceKey, setSelectedSpaceKey, spacesQuery] = useSelectedSpaceKey()
  const taskQuery = useTask(taskKey, selectedSpaceKey)
  const linkDocument = useLinkTaskDocument()
  const [documentId, setDocumentId] = useState('')
  const [linkMessage, setLinkMessage] = useState('')
  const task = taskQuery.data
  const phaseKeys = Array.from(
    new Set((task?.evidence ?? []).map((item) => item.phase_key).filter(Boolean)),
  ) as string[]

  function handleLinkDocument(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const trimmedDocumentId = documentId.trim()
    if (!trimmedDocumentId || linkDocument.isPending) return
    setLinkMessage('')
    linkDocument.mutate(
      {
        spaceKey: selectedSpaceKey,
        taskKey,
        body: { document_id: trimmedDocumentId },
      },
      {
        onSuccess: () => {
          setDocumentId('')
          setLinkMessage('Документ привязан к задаче')
        },
      },
    )
  }

  if (!selectedSpaceKey) {
    if (spacesQuery.isLoading) return <LoadingState message="Загружаем пространства" />
    if (spacesQuery.isError) {
      return (
        <ErrorState
          message={formatApiErrorForUser(spacesQuery.error, 'Не удалось загрузить пространства')}
          onRetry={() => spacesQuery.refetch()}
        />
      )
    }
    return <EmptyState message="Сначала создайте пространство" />
  }
  if (taskQuery.isLoading) return <LoadingState message="Загружаем задачу" />
  if (taskQuery.isError || !task) {
    return (
      <ErrorState
        message={formatApiErrorForUser(taskQuery.error, 'Не удалось открыть задачу')}
        onRetry={() => taskQuery.refetch()}
      />
    )
  }

  return (
    <div className="space-y-5">
      <section className="space-y-3">
        <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
          <div className="min-w-0">
            <Link
              to={scopedPath('/tasks', selectedSpaceKey)}
              className="inline-flex min-h-10 items-center text-sm text-accent hover:text-accent-hover"
            >
              К задачам
            </Link>
            <h1 className="mt-2 break-words text-2xl font-bold">{task.task_key}</h1>
            {task.title && (
              <p className="mt-1 break-words text-sm text-text-secondary">{task.title}</p>
            )}
          </div>
          <SpaceSelector
            value={selectedSpaceKey}
            disabled={linkDocument.isPending}
            onChange={(spaceKey) => {
              setSelectedSpaceKey(spaceKey)
              setDocumentId('')
              setLinkMessage('')
            }}
          />
        </div>
        <div className="flex flex-wrap gap-x-5 gap-y-1 border-y border-border py-2 text-sm text-text-muted">
          <span>Документы: {task.document_count}</span>
          <span>Фазы материалов: {phaseKeys.length}</span>
          <span>Материалы: {task.evidence_count}</span>
        </div>
      </section>

      {spacesQuery.isError && (
        <ErrorState
          message={formatApiErrorForUser(spacesQuery.error, 'Не удалось загрузить пространства')}
          onRetry={() => spacesQuery.refetch()}
        />
      )}

      <div className="grid gap-6 xl:grid-cols-[minmax(0,1.2fr)_minmax(0,0.8fr)]">
        <section className="min-w-0 space-y-3" aria-labelledby="task-documents-title">
          <h2 id="task-documents-title" className="text-base font-semibold">
            Документы задачи
          </h2>
          <form
            onSubmit={handleLinkDocument}
            className="border-y border-border bg-surface-raised p-3"
            aria-label="Привязать документ к задаче"
          >
            <div className="grid gap-3 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-end">
              <div className="space-y-1.5">
                <Label htmlFor="task-document-link">Документ для задачи</Label>
                <Input
                  id="task-document-link"
                  className="min-h-10"
                  value={documentId}
                  onChange={(event) => setDocumentId(event.target.value)}
                  placeholder="product-requirements"
                  disabled={linkDocument.isPending}
                />
              </div>
              <Button
                type="submit"
                className="min-h-10 sm:min-h-10"
                disabled={linkDocument.isPending || !documentId.trim()}
              >
                <Link2 className="h-4 w-4" />
                {linkDocument.isPending ? 'Привязываем' : 'Привязать'}
              </Button>
            </div>
            {linkDocument.isError && (
              <p className="mt-2 text-sm text-danger" role="alert">
                {formatApiErrorForUser(linkDocument.error, 'Не удалось привязать документ')}
              </p>
            )}
            {linkMessage && !linkDocument.isError && (
              <p className="mt-2 text-sm text-success">{linkMessage}</p>
            )}
          </form>
          {task.documents.length === 0 ? (
            <EmptyState message="С задачей пока не связан ни один документ" />
          ) : (
            <ul className="divide-y divide-border border-y border-border">
              {task.documents.map((document) => (
                <li key={document.id}>
                  <Link
                    to={`/documents/${document.id}`}
                    className="flex min-h-14 min-w-0 items-start gap-3 px-2 py-3 hover:bg-surface-raised focus-visible:outline-2 focus-visible:outline-accent"
                  >
                    <FileText className="mt-0.5 h-4 w-4 shrink-0 text-accent" aria-hidden />
                    <span className="min-w-0">
                      <span className="block break-words text-sm font-medium text-text-primary">
                        {document.title}
                      </span>
                      <span className="block text-xs text-text-muted">
                        {formatDocumentType(document.document_type)} ·{' '}
                        {formatDocumentStatus(document.status)}
                      </span>
                    </span>
                  </Link>
                </li>
              ))}
            </ul>
          )}
        </section>

        <section className="min-w-0 space-y-3" aria-labelledby="task-phases-title">
          <h2 id="task-phases-title" className="text-base font-semibold">
            Фазы материалов
          </h2>
          {phaseKeys.length === 0 ? (
            <EmptyState message="Материалы задачи пока не привязаны к фазам" />
          ) : (
            <ul className="divide-y divide-border border-y border-border">
              {phaseKeys.map((phaseKey) => (
                <li key={phaseKey}>
                  <Link
                    to={scopedPath(`/phases/${phaseKey}`, selectedSpaceKey)}
                    className="flex min-h-12 min-w-0 items-center gap-3 px-2 py-2 text-sm hover:bg-surface-raised focus-visible:outline-2 focus-visible:outline-accent"
                  >
                    <GitBranch className="h-4 w-4 shrink-0 text-accent" aria-hidden />
                    <span className="min-w-0 break-words">{phaseKey}</span>
                  </Link>
                </li>
              ))}
            </ul>
          )}
        </section>
      </div>

      <section className="space-y-3" aria-labelledby="task-evidence-title">
        <h2 id="task-evidence-title" className="text-base font-semibold">
          Материалы задачи
        </h2>
        {task.evidence.length === 0 ? (
          <EmptyState message="Материалы пока не прикреплены" />
        ) : (
          <ul className="divide-y divide-border border-y border-border">
            {task.evidence.map((item) => (
              <li key={item.id}>
                <Link
                  to={evidenceTaskPath(selectedSpaceKey, task.task_key, item.id)}
                  className="flex min-h-14 min-w-0 items-start gap-3 px-2 py-3 hover:bg-surface-raised focus-visible:outline-2 focus-visible:outline-accent"
                >
                  <FileCheck2 className="mt-0.5 h-4 w-4 shrink-0 text-accent" aria-hidden />
                  <span className="min-w-0">
                    <span className="block break-words text-sm font-medium text-text-primary">
                      {item.title}
                    </span>
                    <span className="block text-xs text-text-muted">
                      {formatEvidenceType(item.evidence_type)} · {formatDateTime(item.created_at)}
                    </span>
                  </span>
                </Link>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  )
}
