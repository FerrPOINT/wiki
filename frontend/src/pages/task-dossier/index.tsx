import { type FormEvent, useState } from 'react'
import { Link, useParams } from 'react-router'
import { CircleDashed, FileCheck2, FileText, GitBranch, Link2 } from 'lucide-react'
import { defaultSpaceKey, useLinkTaskDocument, useTask, useTasks } from '@/shared/api/hooks'
import { EmptyState, ErrorState, LoadingState } from '@/shared/ui/async-states'
import { Button } from '@/shared/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'
import { Input } from '@/shared/ui/input'
import { Label } from '@/shared/ui/label'
import { Progress } from '@/shared/ui/progress'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/shared/ui/table'
import { formatApiErrorForUser } from '@/shared/lib/api-error'
import {
  formatDateTime,
  formatDocumentStatus,
  formatDocumentType,
  formatEvidenceType,
} from '@/shared/lib/wiki-format'
import type { TaskPage } from '@/api/wiki'

function readiness(task: Pick<TaskPage, 'document_count' | 'evidence_count'>): number {
  const score = task.document_count * 45 + task.evidence_count * 35
  return Math.max(0, Math.min(100, score))
}

export function TaskDossiersPage() {
  const tasksQuery = useTasks(defaultSpaceKey)
  const tasks = tasksQuery.data?.tasks ?? []

  return (
    <div className="space-y-5">
      <section>
        <h1 className="text-2xl font-bold">Задачи</h1>
        <p className="mt-1 max-w-3xl text-sm text-text-muted">
          Wiki собирает документы и материалы вокруг внешнего ключа задачи, но не владеет её
          статусом в трекере.
        </p>
      </section>

      {tasksQuery.isLoading && <LoadingState message="Загружаем задачи" />}
      {tasksQuery.isError && (
        <ErrorState
          message={formatApiErrorForUser(tasksQuery.error, 'Не удалось загрузить задачи')}
          onRetry={() => tasksQuery.refetch()}
        />
      )}
      {!tasksQuery.isLoading && !tasksQuery.isError && tasks.length === 0 && (
        <EmptyState message="Документы ещё не связаны с задачами" />
      )}
      {!tasksQuery.isLoading && !tasksQuery.isError && tasks.length > 0 && (
        <section className="grid gap-4 lg:grid-cols-3">
          {tasks.map((task) => {
            const value = readiness(task)
            return (
              <Card key={task.task_key}>
                <CardHeader>
                  <CardTitle className="text-base">{task.task_key}</CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <p className="min-h-10 text-sm text-text-secondary">
                    {task.title ?? 'Документы и материалы по задаче'}
                  </p>
                  <div className="grid grid-cols-2 gap-2 text-xs text-text-muted">
                    <span className="rounded bg-surface-raised px-2 py-1">
                      Документы: {task.document_count}
                    </span>
                    <span className="rounded bg-surface-raised px-2 py-1">
                      Материалы: {task.evidence_count}
                    </span>
                  </div>
                  <div className="space-y-1.5">
                    <div className="flex justify-between text-xs text-text-muted">
                      <span>Заполненность</span>
                      <span>{value}%</span>
                    </div>
                    <Progress value={value} />
                  </div>
                  <Link to={`/tasks/${task.task_key}`} className="inline-flex text-sm text-accent">
                    Открыть задачу
                  </Link>
                </CardContent>
              </Card>
            )
          })}
        </section>
      )}
    </div>
  )
}

export function TaskDossierPage() {
  const { taskKey = 'SDLC-42' } = useParams()
  const taskQuery = useTask(taskKey, defaultSpaceKey)
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
    if (!trimmedDocumentId) return
    setLinkMessage('')
    linkDocument.mutate(
      {
        spaceKey: defaultSpaceKey,
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
      <section className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
        <div>
          <div className="text-sm text-text-muted">карточка задачи</div>
          <h1 className="mt-2 text-2xl font-bold">{task.task_key}</h1>
          <p className="mt-1 max-w-3xl text-sm text-text-muted">
            Документы, фазы и материалы, связанные с задачей.
          </p>
        </div>
        <div className="grid grid-cols-3 gap-2 text-center text-xs text-text-muted sm:min-w-80">
          <div className="rounded-md border border-border p-2">
            <div className="text-lg font-semibold text-text-primary">{task.document_count}</div>
            документы
          </div>
          <div className="rounded-md border border-border p-2">
            <div className="text-lg font-semibold text-text-primary">{phaseKeys.length}</div>
            фазы
          </div>
          <div className="rounded-md border border-border p-2">
            <div className="text-lg font-semibold text-text-primary">{task.evidence_count}</div>
            материалы
          </div>
        </div>
      </section>

      <section className="grid gap-4 xl:grid-cols-[1.2fr_0.8fr]">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Документы задачи</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <form
              onSubmit={handleLinkDocument}
              className="rounded-md bg-surface-raised p-3"
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
                <Button type="submit" disabled={linkDocument.isPending || !documentId.trim()}>
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
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Документ</TableHead>
                    <TableHead>Тип</TableHead>
                    <TableHead>Статус</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {task.documents.map((document) => (
                    <TableRow key={document.id}>
                      <TableCell>
                        <Link
                          to={`/documents/${document.slug}`}
                          className="inline-flex items-center gap-2 text-accent hover:text-accent-hover"
                        >
                          <FileText className="h-4 w-4" />
                          {document.title}
                        </Link>
                      </TableCell>
                      <TableCell>{formatDocumentType(document.document_type)}</TableCell>
                      <TableCell>
                        <span className="rounded bg-surface-raised px-2 py-1 text-xs text-text-secondary">
                          {formatDocumentStatus(document.status)}
                        </span>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">Фазы</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            {phaseKeys.length === 0 && (
              <EmptyState message="Материалы задачи пока не привязаны к фазам" />
            )}
            {phaseKeys.map((phaseKey) => (
              <Link
                key={phaseKey}
                to={`/phases/${phaseKey}`}
                className="flex items-center justify-between rounded-md border border-border p-3 text-sm hover:bg-surface-raised"
              >
                <span className="inline-flex items-center gap-2">
                  <GitBranch className="h-4 w-4 text-accent" />
                  {phaseKey}
                </span>
                <span className="inline-flex items-center gap-2 text-xs text-text-muted">
                  есть материалы
                  <CircleDashed className="h-4 w-4 text-warning" />
                </span>
              </Link>
            ))}
          </CardContent>
        </Card>
      </section>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-base">
            <FileCheck2 className="h-4 w-4 text-accent" />
            Материалы задачи
          </CardTitle>
        </CardHeader>
        <CardContent className="grid gap-3 lg:grid-cols-3">
          {task.evidence.length === 0 && <EmptyState message="Материалы пока не прикреплены" />}
          {task.evidence.map((item) => (
            <Link
              key={item.id}
              to="/evidence"
              className="rounded-md border border-border p-3 hover:bg-surface-raised"
            >
              <div className="flex items-center gap-2 text-sm font-medium">
                <Link2 className="h-4 w-4 text-accent" />
                {item.title}
              </div>
              <div className="mt-2 flex gap-2 text-xs text-text-muted">
                <span>{formatEvidenceType(item.evidence_type)}</span>
                <span>·</span>
                <span>{formatDateTime(item.created_at)}</span>
              </div>
            </Link>
          ))}
        </CardContent>
      </Card>
    </div>
  )
}
