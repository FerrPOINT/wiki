import { FormEvent, useEffect, useState } from 'react'
import { Link, useParams } from 'react-router'
import {
  Archive,
  CheckCircle2,
  Clock3,
  Eye,
  FileCheck2,
  FilePenLine,
  FileText,
  GitBranch,
  History,
  Link2,
  MoveRight,
  Save,
  Send,
  X,
} from 'lucide-react'
import {
  useArchiveDocument,
  useDocument,
  useDocumentRevision,
  useDocumentRevisions,
  useMoveDocument,
  usePublishDocument,
  useUpdateDocumentDraft,
} from '@/shared/api/hooks'
import { ConfirmDialog, EmptyState, ErrorState, LoadingState } from '@/shared/ui/async-states'
import { Button } from '@/shared/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'
import { Input } from '@/shared/ui/input'
import { Label } from '@/shared/ui/label'
import { Textarea } from '@/shared/ui/textarea'
import { formatApiErrorForUser, formatFirstApiErrorForUser } from '@/shared/lib/api-error'
import {
  formatDateTime,
  formatDocumentStatus,
  formatDocumentType,
  formatEvidenceType,
  shortText,
} from '@/shared/lib/wiki-format'

function optional(value: string) {
  const trimmed = value.trim()
  return trimmed.length > 0 ? trimmed : null
}

function RenderedDocumentBody({
  html,
  emptyMessage,
  compact = false,
}: {
  html: string
  emptyMessage: string
  compact?: boolean
}) {
  if (html.trim().length === 0) return <EmptyState message={emptyMessage} />

  return (
    <div
      className={compact ? 'wiki-rendered wiki-rendered-compact' : 'wiki-rendered'}
      dangerouslySetInnerHTML={{ __html: html }}
    />
  )
}

export function DocumentPage() {
  const { documentId = 'product-requirements' } = useParams()
  const documentQuery = useDocument(documentId)
  const revisionsQuery = useDocumentRevisions(documentId)
  const updateDraft = useUpdateDocumentDraft()
  const publishDocument = usePublishDocument()
  const archiveDocument = useArchiveDocument()
  const moveDocument = useMoveDocument()
  const document = documentQuery.data
  const revisions = revisionsQuery.data?.revisions ?? []
  const [selectedRevisionId, setSelectedRevisionId] = useState<string | null>(null)
  const selectedRevisionQuery = useDocumentRevision(
    documentId,
    selectedRevisionId ?? '',
    selectedRevisionId !== null,
  )
  const selectedRevision = selectedRevisionQuery.data
  const [draftTitle, setDraftTitle] = useState('')
  const [draftBody, setDraftBody] = useState('')
  const [publishSummary, setPublishSummary] = useState('')
  const [parentId, setParentId] = useState('')
  const [archiveOpen, setArchiveOpen] = useState(false)
  const [statusMessage, setStatusMessage] = useState('')
  const [isPublishingFlow, setIsPublishingFlow] = useState(false)
  const [loadedDocumentId, setLoadedDocumentId] = useState<string | null>(null)

  useEffect(() => {
    if (!document || loadedDocumentId === document.id) return
    setDraftTitle(document.title)
    setDraftBody(document.draft_markdown || document.body_markdown)
    setParentId(document.parent_id ?? '')
    setPublishSummary('')
    setStatusMessage('')
    setIsPublishingFlow(false)
    setSelectedRevisionId(null)
    setLoadedDocumentId(document.id)
  }, [document, loadedDocumentId])

  if (documentQuery.isLoading) return <LoadingState message="Загружаем документ" />
  if (documentQuery.isError || !document) {
    return (
      <ErrorState
        message={formatApiErrorForUser(documentQuery.error, 'Не удалось открыть документ')}
        onRetry={() => documentQuery.refetch()}
      />
    )
  }

  const isArchived = document.status === 'archived'
  const canEdit = document.can_edit && !isArchived
  const publishedHtml = document.body_html || document.current_revision?.body_html || ''
  const currentParentId = document.parent_id ?? null
  const nextParentId = optional(parentId)
  const parentChanged = nextParentId !== currentParentId
  const mutationError = formatFirstApiErrorForUser(
    [updateDraft.error, publishDocument.error, archiveDocument.error, moveDocument.error],
    'Не удалось выполнить действие',
  )

  function handleSaveDraft(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (!canEdit) return
    setStatusMessage('')
    updateDraft.mutate(
      {
        documentId,
        body: {
          title: draftTitle.trim(),
          content_markdown: draftBody,
        },
      },
      {
        onSuccess: (updated) => {
          setDraftTitle(updated.title)
          setDraftBody(updated.draft_markdown || updated.body_markdown)
          setStatusMessage('Черновик сохранён')
        },
      },
    )
  }

  async function handlePublish() {
    if (!document || !canEdit) return
    const baseRevisionId = document.current_revision?.id
    setStatusMessage('')
    setIsPublishingFlow(true)
    try {
      const draft = await updateDraft.mutateAsync({
        documentId,
        body: {
          title: draftTitle.trim(),
          content_markdown: draftBody,
        },
      })
      setDraftTitle(draft.title)
      setDraftBody(draft.draft_markdown || draft.body_markdown)

      const revision = await publishDocument.mutateAsync({
        documentId,
        body: {
          base_revision_id: baseRevisionId,
          summary: optional(publishSummary),
        },
      })
      setPublishSummary('')
      setStatusMessage(`Опубликована ревизия ${revision.version}`)
    } catch {
      // Mutation errors are rendered from React Query state above.
    } finally {
      setIsPublishingFlow(false)
    }
  }

  function handleMove(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (!canEdit) return
    setStatusMessage('')
    moveDocument.mutate(
      {
        documentId,
        body: {
          parent_id: nextParentId,
        },
      },
      {
        onSuccess: (updated) => {
          setParentId(updated.parent_id ?? '')
          setStatusMessage('Положение документа обновлено')
        },
      },
    )
  }

  function handleArchive() {
    if (!canEdit) return
    setArchiveOpen(false)
    setStatusMessage('')
    archiveDocument.mutate(documentId, {
      onSuccess: () => setStatusMessage('Документ архивирован'),
    })
  }

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
              Новый документ
            </Link>
          </Button>
          <Button asChild size="sm" variant="outline">
            <Link to="/evidence">
              <FileCheck2 className="h-4 w-4" />
              Материалы
            </Link>
          </Button>
          {canEdit && (
            <Button
              type="button"
              size="sm"
              variant="destructive"
              disabled={archiveDocument.isPending}
              onClick={() => setArchiveOpen(true)}
            >
              <Archive className="h-4 w-4" />
              Архивировать
            </Button>
          )}
        </div>
      </section>

      <ConfirmDialog
        open={archiveOpen}
        onOpenChange={setArchiveOpen}
        title="Архивировать документ?"
        description="Документ исчезнет из обычного дерева страниц, но останется в истории и аудите."
        onConfirm={handleArchive}
      />

      {(statusMessage || mutationError) && (
        <section className="rounded-md border border-border bg-surface p-3 text-sm">
          {statusMessage && <p className="text-success">{statusMessage}</p>}
          {mutationError && <p className="text-danger">{mutationError}</p>}
        </section>
      )}

      <section className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_22rem]">
        <div className="space-y-4">
          {canEdit && (
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Черновик</CardTitle>
              </CardHeader>
              <CardContent>
                <form onSubmit={handleSaveDraft} className="space-y-4">
                  <div className="space-y-1.5">
                    <Label htmlFor="document-edit-title">Название</Label>
                    <Input
                      id="document-edit-title"
                      value={draftTitle}
                      onChange={(event) => setDraftTitle(event.target.value)}
                      required
                    />
                  </div>
                  <div className="space-y-1.5">
                    <Label htmlFor="document-edit-markdown">Markdown</Label>
                    <Textarea
                      id="document-edit-markdown"
                      className="min-h-80 font-mono text-sm"
                      aria-label="Markdown черновика"
                      value={draftBody}
                      onChange={(event) => setDraftBody(event.target.value)}
                      required
                    />
                  </div>
                  <div className="grid gap-3 md:grid-cols-[minmax(0,1fr)_auto_auto]">
                    <Input
                      value={publishSummary}
                      onChange={(event) => setPublishSummary(event.target.value)}
                      placeholder="Комментарий к публикации"
                      aria-label="Комментарий к публикации"
                    />
                    <Button
                      type="submit"
                      variant="secondary"
                      disabled={
                        updateDraft.isPending || isPublishingFlow || draftTitle.trim().length === 0
                      }
                    >
                      <Save className="h-4 w-4" />
                      {updateDraft.isPending ? 'Сохраняем...' : 'Сохранить'}
                    </Button>
                    <Button
                      type="button"
                      disabled={
                        updateDraft.isPending ||
                        publishDocument.isPending ||
                        isPublishingFlow ||
                        draftTitle.trim().length === 0 ||
                        draftBody.trim().length === 0
                      }
                      onClick={handlePublish}
                    >
                      <Send className="h-4 w-4" />
                      {isPublishingFlow || publishDocument.isPending
                        ? 'Публикуем...'
                        : 'Опубликовать'}
                    </Button>
                  </div>
                </form>
              </CardContent>
            </Card>
          )}

          <Card>
            <CardHeader>
              <CardTitle className="text-base">Опубликованное содержание</CardTitle>
            </CardHeader>
            <CardContent>
              <RenderedDocumentBody
                html={publishedHtml}
                emptyMessage="В документе пока нет опубликованного содержания"
              />
            </CardContent>
          </Card>
        </div>

        <div className="space-y-4">
          {!canEdit && (
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Режим чтения</CardTitle>
              </CardHeader>
              <CardContent className="text-sm text-text-secondary">
                Вам доступна опубликованная версия документа. Черновик и действия редактирования
                скрыты.
              </CardContent>
            </Card>
          )}

          {canEdit && (
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Положение в дереве</CardTitle>
              </CardHeader>
              <CardContent>
                <form onSubmit={handleMove} className="space-y-3">
                  <div className="space-y-1.5">
                    <Label htmlFor="document-parent">Родительский документ</Label>
                    <Input
                      id="document-parent"
                      value={parentId}
                      onChange={(event) => setParentId(event.target.value)}
                      placeholder="корень пространства"
                    />
                  </div>
                  <Button
                    type="submit"
                    size="sm"
                    variant="secondary"
                    disabled={moveDocument.isPending || !parentChanged}
                  >
                    <MoveRight className="h-4 w-4" />
                    {moveDocument.isPending ? 'Сохраняем...' : 'Сохранить место'}
                  </Button>
                </form>
              </CardContent>
            </Card>
          )}

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
              {revisionsQuery.isError && (
                <ErrorState
                  message={formatApiErrorForUser(
                    revisionsQuery.error,
                    'Не удалось загрузить ревизии',
                  )}
                  onRetry={() => revisionsQuery.refetch()}
                />
              )}
              {!revisionsQuery.isLoading && !revisionsQuery.isError && revisions.length === 0 && (
                <EmptyState message="Ревизий пока нет" />
              )}
              {revisions.map((revision) => (
                <div
                  key={revision.id}
                  role="group"
                  aria-label={`Ревизия ${revision.version}`}
                  className="space-y-3 rounded-md border border-border p-3"
                >
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
                  <Button
                    type="button"
                    size="sm"
                    variant={selectedRevisionId === revision.id ? 'secondary' : 'outline'}
                    onClick={() => setSelectedRevisionId(revision.id)}
                  >
                    <Eye className="h-3.5 w-3.5" />
                    Открыть
                  </Button>
                </div>
              ))}
            </CardContent>
          </Card>

          {selectedRevisionId && (
            <Card>
              <CardHeader>
                <div className="flex items-center justify-between gap-3">
                  <CardTitle className="text-base">Снимок ревизии</CardTitle>
                  <Button
                    type="button"
                    size="sm"
                    variant="ghost"
                    onClick={() => setSelectedRevisionId(null)}
                  >
                    <X className="h-4 w-4" />
                    Закрыть
                  </Button>
                </div>
              </CardHeader>
              <CardContent className="space-y-3">
                {selectedRevisionQuery.isLoading && (
                  <LoadingState message="Загружаем снимок ревизии" />
                )}
                {selectedRevisionQuery.isError && (
                  <ErrorState
                    message={formatApiErrorForUser(
                      selectedRevisionQuery.error,
                      'Не удалось открыть ревизию',
                    )}
                    onRetry={() => selectedRevisionQuery.refetch()}
                  />
                )}
                {selectedRevision && (
                  <>
                    <div className="space-y-1 text-sm">
                      <div className="font-medium">
                        Ревизия {selectedRevision.version}: {selectedRevision.title}
                      </div>
                      <div className="text-xs text-text-muted">
                        {formatDateTime(selectedRevision.published_at)} · автор{' '}
                        {selectedRevision.author_id}
                      </div>
                      <p className="text-xs text-text-secondary">
                        {shortText(selectedRevision.summary, 'Без описания изменений')}
                      </p>
                    </div>
                    <div className="max-h-96 overflow-auto rounded-md border border-border bg-surface p-3">
                      <RenderedDocumentBody
                        html={selectedRevision.body_html}
                        emptyMessage="В ревизии нет опубликованного содержания"
                        compact
                      />
                    </div>
                  </>
                )}
              </CardContent>
            </Card>
          )}
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
