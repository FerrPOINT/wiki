import { SpaceSelector, useSelectedSpaceKey } from '@/widgets/dossier-space'
import { type FormEvent, useEffect, useRef, useState } from 'react'
import {
  SectionHeader,
  NamespaceLink as Link,
  ResourceList,
  ResourceListRow,
  EmptyState,
  ErrorState,
  LoadingState,
  Button,
  Input,
  Label,
  SearchInput,
  PageNavigation,
  PageHeader,
  Notice,
  TextLink,
} from '@pdlc/ui/ui'
import { useNavigate, useParams } from 'react-router'
import { FileCheck2, FileText, GitBranch, Link2 } from 'lucide-react'
import { defaultSpaceKey, useLinkTaskDocument, useTask, useTaskSummaries } from '@/shared/api/hooks'

import { formatApiErrorForUser } from '@/shared/lib/api-error'
import { pathWithSearchParams, useDossierCatalogUrl } from '@/shared/lib/dossier-catalog-url'
import {
  formatDateTime,
  formatDocumentStatus,
  formatDocumentType,
  formatEvidenceType,
} from '@/shared/lib/wiki-format'

const pageSize = 12

function scopedPath(path: string, spaceKey: string): string {
  return spaceKey === defaultSpaceKey ? path : `${path}?space=${encodeURIComponent(spaceKey)}`
}

function evidenceTaskPath(spaceKey: string, taskKey: string, evidenceId: string): string {
  const params = new URLSearchParams({ space: spaceKey, task_key: taskKey, id: evidenceId })
  return `/evidence?${params.toString()}`
}

function TaskCatalog({ spaceKey }: { spaceKey: string }) {
  const {
    appliedSearch,
    changePage,
    currentPage,
    cursor,
    hasPreviousPage,
    search,
    searchParams,
    setSearch,
  } = useDossierCatalogUrl()
  const listRef = useRef<HTMLElement>(null)
  const tasksQuery = useTaskSummaries(spaceKey, {
    limit: pageSize,
    cursor,
    q: appliedSearch || undefined,
  })
  const tasks = tasksQuery.data?.tasks ?? []
  const nextCursor = tasksQuery.data?.next_cursor

  function navigatePage(direction: 'previous' | 'next') {
    changePage(direction, direction === 'next' ? (nextCursor ?? undefined) : undefined)
    listRef.current?.scrollIntoView?.({ block: 'start' })
  }

  return (
    <section ref={listRef} aria-label="Список задач" className="scroll-mt-16 space-y-3">
      <div className="max-w-md">
        <SearchInput
          aria-label="Найти задачу"
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
            cursor
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
          <ResourceList asChild>
            <ul>
              {tasks.map((task) => (
                <li key={task.task_key}>
                  <ResourceListRow asChild interactive>
                    <Link
                      to={pathWithSearchParams(`/tasks/${task.task_key}`, searchParams)}
                      className="flex min-w-0 flex-wrap items-center justify-between gap-x-4 gap-y-1"
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
                  </ResourceListRow>
                </li>
              ))}
            </ul>
          </ResourceList>
        </>
      )}
      {(hasPreviousPage || nextCursor) && (
        <PageNavigation
          label="Страницы задач"
          page={currentPage}
          hasNext={Boolean(nextCursor) && !tasksQuery.isError}
          pending={tasksQuery.isLoading || tasksQuery.isFetching}
          previousLabel="Назад"
          nextLabel="Далее"
          summary={`Страница ${currentPage}`}
          onPageChange={(page) => navigatePage(page < currentPage ? 'previous' : 'next')}
        />
      )}
    </section>
  )
}

export function TaskDossiersPage() {
  const [selectedSpaceKey, setSelectedSpaceKey, spacesQuery] = useSelectedSpaceKey()

  return (
    <div className="space-y-5">
      <PageHeader
        title="Задачи"
        context="Wiki собирает документы и материалы вокруг внешнего ключа задачи, но не владеет её статусом в трекере."
        actions={
          <SpaceSelector
            id="task-space-selector"
            value={selectedSpaceKey}
            onChange={setSelectedSpaceKey}
          />
        }
      />

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
  const navigate = useNavigate()
  const { taskKey = 'BASE-42' } = useParams()
  const [selectedSpaceKey, setSelectedSpaceKey, spacesQuery, searchParams] = useSelectedSpaceKey()
  const taskQuery = useTask(taskKey, selectedSpaceKey)
  const linkDocument = useLinkTaskDocument()
  const [documentId, setDocumentId] = useState('')
  const [linkMessage, setLinkMessage] = useState('')
  const task = taskQuery.data
  useEffect(() => {
    const stable = task?.managed_task_ref
    if (stable)
      navigate(
        `/managed-tasks/${stable.tracker_instance_id}/${stable.task_id}?${searchParams.toString()}`,
        { replace: true },
      )
  }, [task?.managed_task_ref, navigate, searchParams])
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
        <div className="space-y-2">
          <TextLink asChild>
            <Link
              to={pathWithSearchParams('/tasks', searchParams)}
              className="inline-flex min-h-10 items-center text-sm"
            >
              К задачам
            </Link>
          </TextLink>
          <PageHeader
            title={task.task_key}
            context={
              task.title && (
                <p className="mt-1 break-words text-sm text-text-secondary">{task.title}</p>
              )
            }
            actions={
              <>
                <SpaceSelector
                  id="task-space-selector"
                  value={selectedSpaceKey}
                  disabled={linkDocument.isPending}
                  onChange={(spaceKey) => {
                    setSelectedSpaceKey(spaceKey)
                    setDocumentId('')
                    setLinkMessage('')
                  }}
                />
              </>
            }
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

      <div className="page-split items-start">
        <section className="min-w-0 space-y-3" aria-labelledby="task-documents-title">
          <SectionHeader headingId="task-documents-title" title={<>Документы задачи</>} />
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

                  value={documentId}
                  onChange={(event) => setDocumentId(event.target.value)}
                  placeholder="product-requirements"
                  disabled={linkDocument.isPending}
                />
              </div>
              <Button
                type="submit"

                disabled={linkDocument.isPending || !documentId.trim()}
              >
                <Link2 className="h-4 w-4" />
                {linkDocument.isPending ? 'Привязываем' : 'Привязать'}
              </Button>
            </div>
            {linkDocument.isError && (
              <Notice className="mt-2" role="alert" appearance="inline" tone="danger">
                {formatApiErrorForUser(linkDocument.error, 'Не удалось привязать документ')}
              </Notice>
            )}
            {linkMessage && !linkDocument.isError && (
              <p className="mt-2 text-sm text-success">{linkMessage}</p>
            )}
          </form>
          {task.documents.length === 0 ? (
            <EmptyState message="С задачей пока не связан ни один документ" />
          ) : (
            <ResourceList asChild>
              <ul>
                {task.documents.map((document) => (
                  <li key={document.id}>
                    <ResourceListRow asChild interactive>
                      <Link
                        to={`/documents/${document.id}`}
                        className="flex min-w-0 items-start gap-3"
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
                    </ResourceListRow>
                  </li>
                ))}
              </ul>
            </ResourceList>
          )}
        </section>

        <aside className="min-w-0 space-y-3" aria-labelledby="task-phases-title">
          <SectionHeader headingId="task-phases-title" title={<>Фазы материалов</>} />
          {phaseKeys.length === 0 ? (
            <EmptyState message="Материалы задачи пока не привязаны к фазам" />
          ) : (
            <ResourceList asChild>
              <ul>
                {phaseKeys.map((phaseKey) => (
                  <li key={phaseKey}>
                    <ResourceListRow asChild interactive>
                      <Link
                        to={scopedPath(`/phases/${phaseKey}`, selectedSpaceKey)}
                        className="flex min-w-0 items-center gap-3 text-sm"
                      >
                        <GitBranch className="h-4 w-4 shrink-0 text-accent" aria-hidden />
                        <span className="min-w-0 break-words">{phaseKey}</span>
                      </Link>
                    </ResourceListRow>
                  </li>
                ))}
              </ul>
            </ResourceList>
          )}
        </aside>
      </div>

      <section className="space-y-3" aria-labelledby="task-evidence-title">
        <SectionHeader headingId="task-evidence-title" title={<>Материалы задачи</>} />
        {task.evidence.length === 0 ? (
          <EmptyState message="Материалы пока не прикреплены" />
        ) : (
          <ResourceList asChild>
            <ul>
              {task.evidence.map((item) => (
                <li key={item.id}>
                  <ResourceListRow asChild interactive>
                    <Link
                      to={evidenceTaskPath(selectedSpaceKey, task.task_key, item.id)}
                      className="flex min-w-0 items-start gap-3"
                    >
                      <FileCheck2 className="mt-0.5 h-4 w-4 shrink-0 text-accent" aria-hidden />
                      <span className="min-w-0">
                        <span className="block break-words text-sm font-medium text-text-primary">
                          {item.title}
                        </span>
                        <span className="block text-xs text-text-muted">
                          {formatEvidenceType(item.evidence_type)} ·{' '}
                          {formatDateTime(item.created_at)}
                        </span>
                      </span>
                    </Link>
                  </ResourceListRow>
                </li>
              ))}
            </ul>
          </ResourceList>
        )}
      </section>
    </div>
  )
}
