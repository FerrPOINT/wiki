import { Link, useParams } from 'react-router'
import {
  CheckCircle2,
  Clock3,
  FileCheck2,
  FilePenLine,
  FileText,
  GitBranch,
  History,
  Link2,
} from 'lucide-react'
import { useDocument, useDocumentRevisions } from '@/shared/api/hooks'
import { EmptyState, ErrorState, LoadingState } from '@/shared/ui/async-states'
import { Button } from '@/shared/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'
import {
  formatDateTime,
  formatDocumentStatus,
  formatDocumentType,
  formatEvidenceType,
  shortText,
} from '@/shared/lib/wiki-format'

export function DocumentPage() {
  const { documentId = 'product-requirements' } = useParams()
  const documentQuery = useDocument(documentId)
  const revisionsQuery = useDocumentRevisions(documentId)
  const document = documentQuery.data
  const revisions = revisionsQuery.data?.revisions ?? []

  if (documentQuery.isLoading) return <LoadingState message="Загружаем документ" />
  if (documentQuery.isError || !document) {
    return (
      <ErrorState message="Не удалось открыть документ" onRetry={() => documentQuery.refetch()} />
    )
  }

  const content = document.body_markdown || document.draft_markdown

  return (
    <article className="space-y-5">
      <section className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
        <div>
          <div className="text-sm text-text-muted">
            {document.space_key} / {formatDocumentType(document.document_type)} / {document.slug}
          </div>
          <h1 className="mt-2 text-2xl font-bold">{document.title}</h1>
          <div className="mt-3 flex flex-wrap gap-2 text-xs text-text-secondary">
            <span className="inline-flex items-center gap-1 rounded bg-surface-raised px-2 py-1">
              <CheckCircle2 className="h-3.5 w-3.5 text-success" />
              {formatDocumentStatus(document.status)}
            </span>
            <span className="inline-flex items-center gap-1 rounded bg-surface-raised px-2 py-1">
              <Clock3 className="h-3.5 w-3.5" />
              ревизия {document.current_revision?.version ?? 0}
            </span>
            <span className="rounded bg-surface-raised px-2 py-1">
              обновлено {formatDateTime(document.updated_at)}
            </span>
          </div>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button asChild size="sm" variant="secondary">
            <Link to="/documents/new">
              <FilePenLine className="h-4 w-4" />
              Новый черновик
            </Link>
          </Button>
          <Button asChild size="sm" variant="outline">
            <Link to="/evidence">
              <FileCheck2 className="h-4 w-4" />
              Материалы
            </Link>
          </Button>
        </div>
      </section>

      <section className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_22rem]">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Содержимое</CardTitle>
          </CardHeader>
          <CardContent>
            {content.trim().length === 0 ? (
              <EmptyState message="В документе пока нет опубликованного содержания" />
            ) : (
              <pre className="whitespace-pre-wrap text-sm leading-6 text-text-secondary">
                {content}
              </pre>
            )}
          </CardContent>
        </Card>

        <div className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Связанные объекты</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3 text-sm">
              {document.task_keys.length === 0 && document.phase_keys.length === 0 && (
                <EmptyState message="Документ пока не связан с задачей или фазой" />
              )}
              {document.task_keys.map((taskKey) => (
                <Link
                  key={taskKey}
                  to={`/tasks/${taskKey}`}
                  className="flex items-center gap-2 rounded-md border border-border p-3 hover:bg-surface-raised"
                >
                  <FileText className="h-4 w-4 text-accent" />
                  {taskKey}
                </Link>
              ))}
              {document.phase_keys.map((phaseKey) => (
                <Link
                  key={phaseKey}
                  to={`/phases/${phaseKey}`}
                  className="flex items-center gap-2 rounded-md border border-border p-3 hover:bg-surface-raised"
                >
                  <GitBranch className="h-4 w-4 text-accent" />
                  {phaseKey}
                </Link>
              ))}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-base">Ревизии</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              {revisionsQuery.isLoading && <LoadingState message="Загружаем ревизии" />}
              {revisionsQuery.isError && <ErrorState message="Не удалось загрузить ревизии" />}
              {!revisionsQuery.isLoading && !revisionsQuery.isError && revisions.length === 0 && (
                <EmptyState message="Ревизий пока нет" />
              )}
              {revisions.map((revision) => (
                <div key={revision.id} className="rounded-md border border-border p-3">
                  <div className="flex items-center justify-between gap-3 text-sm font-medium">
                    <span className="inline-flex items-center gap-2">
                      <History className="h-4 w-4 text-accent" />
                      Ревизия {revision.version}
                    </span>
                    <span className="text-xs text-text-muted">
                      {formatDateTime(revision.published_at)}
                    </span>
                  </div>
                  <p className="mt-1 text-xs text-text-secondary">
                    {shortText(revision.summary, 'Без описания изменений')}
                  </p>
                  <p className="mt-1 text-xs text-text-muted">{revision.author_id}</p>
                </div>
              ))}
            </CardContent>
          </Card>
        </div>
      </section>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Связанные материалы</CardTitle>
        </CardHeader>
        <CardContent className="grid gap-3 lg:grid-cols-3">
          {document.evidence.length === 0 && <EmptyState message="Материалы пока не прикреплены" />}
          {document.evidence.map((item) => (
            <Link
              key={item.id}
              to="/evidence"
              className="rounded-md border border-border p-3 hover:bg-surface-raised"
            >
              <div className="flex items-center gap-2 text-sm font-medium">
                <Link2 className="h-4 w-4 text-accent" />
                {item.title}
              </div>
              <div className="mt-2 text-xs text-text-muted">
                {formatEvidenceType(item.evidence_type)}
              </div>
            </Link>
          ))}
        </CardContent>
      </Card>
    </article>
  )
}
