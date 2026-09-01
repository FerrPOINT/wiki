import { FormEvent, useMemo, useState } from 'react'
import { Link, useSearchParams } from 'react-router'
import {
  CheckCircle2,
  Download,
  ExternalLink,
  FileText,
  Link2,
  RotateCcw,
  Upload,
} from 'lucide-react'
import {
  defaultSpaceKey,
  useAttachment,
  useCreateEvidence,
  useCreateFileEvidence,
  useDownloadAttachment,
  useEvidence,
  useEvidenceItem,
} from '@/shared/api/hooks'
import { EmptyState, ErrorState, LoadingState } from '@/shared/ui/async-states'
import { Button } from '@/shared/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'
import { Input } from '@/shared/ui/input'
import { Label } from '@/shared/ui/label'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/shared/ui/table'
import { formatApiErrorForUser, formatFirstApiErrorForUser } from '@/shared/lib/api-error'
import { formatBytes, formatDateTime, formatEvidenceType } from '@/shared/lib/wiki-format'
import type { AttachmentDownload, Evidence } from '@/api/wiki'

type EvidenceMode = 'external_url' | 'uploaded_file'

function optional(value: string) {
  const trimmed = value.trim()
  return trimmed.length > 0 ? trimmed : null
}

function browserDownload(download: AttachmentDownload, fallbackFileName: string) {
  const href = URL.createObjectURL(download.blob)
  const anchor = window.document.createElement('a')
  anchor.href = href
  anchor.download = download.fileName ?? fallbackFileName
  anchor.click()
  URL.revokeObjectURL(href)
}

function AttachmentMetadata({ item }: { item: Evidence }) {
  const hasAttachment = item.evidence_type === 'uploaded_file' && Boolean(item.attachment_id)
  const attachmentQuery = useAttachment(hasAttachment ? item.attachment_id : null)
  const downloadAttachment = useDownloadAttachment()
  const attachment = attachmentQuery.data
  const checksum = attachment?.checksum ?? item.checksum
  const fileName = attachment?.file_name ?? item.attachment_id ?? item.title

  if (!hasAttachment) {
    return <span className="text-xs text-text-muted">внешняя ссылка</span>
  }

  function handleDownload() {
    if (!item.attachment_id) return
    downloadAttachment.mutate(item.attachment_id, {
      onSuccess: (download) => browserDownload(download, fileName),
    })
  }

  return (
    <div className="space-y-2 text-xs text-text-muted">
      <div>
        <div className="max-w-[16rem] truncate font-medium text-text-secondary" title={fileName}>
          {attachmentQuery.isLoading ? 'загружаем файл...' : fileName}
        </div>
        {attachment && (
          <div>
            {formatBytes(attachment.size_bytes)} · {attachment.content_type}
          </div>
        )}
        {attachmentQuery.isError && <div className="text-warning">метаданные недоступны</div>}
      </div>
      {checksum ? (
        <code
          className="block max-w-[16rem] truncate rounded bg-background px-2 py-1"
          title={checksum}
        >
          {checksum}
        </code>
      ) : (
        <span>контрольная сумма не получена</span>
      )}
      <Button
        type="button"
        size="sm"
        variant="outline"
        onClick={handleDownload}
        disabled={downloadAttachment.isPending}
        aria-label={`Скачать ${item.title}`}
      >
        <Download className="h-3.5 w-3.5" />
        {downloadAttachment.isPending ? 'Скачиваем...' : 'Скачать'}
      </Button>
      {downloadAttachment.isError && (
        <div className="text-danger">
          {formatApiErrorForUser(downloadAttachment.error, 'Не удалось скачать файл')}
        </div>
      )}
    </div>
  )
}

function EvidenceTargetLinks({ item }: { item: Evidence }) {
  return (
    <div className="flex flex-wrap gap-2 text-sm">
      {item.document_id && (
        <Link
          to={`/documents/${item.document_id}`}
          className="rounded bg-surface-raised px-2 py-1 text-accent"
        >
          документ {item.document_id}
        </Link>
      )}
      {item.task_key && (
        <Link
          to={`/tasks/${item.task_key}`}
          className="rounded bg-surface-raised px-2 py-1 text-accent"
        >
          задача {item.task_key}
        </Link>
      )}
      {item.phase_key && (
        <Link
          to={`/phases/${item.phase_key}`}
          className="rounded bg-surface-raised px-2 py-1 text-accent"
        >
          фаза {item.phase_key}
        </Link>
      )}
    </div>
  )
}

export function EvidencePage() {
  const [searchParams, setSearchParams] = useSearchParams()
  const [query, setQuery] = useState('')
  const [mode, setMode] = useState<EvidenceMode>('external_url')
  const [title, setTitle] = useState('')
  const [url, setUrl] = useState('')
  const [space, setSpace] = useState(defaultSpaceKey)
  const [documentId, setDocumentId] = useState('')
  const [task, setTask] = useState('')
  const [phase, setPhase] = useState('')
  const [filterSpace, setFilterSpace] = useState(defaultSpaceKey)
  const [filterDocument, setFilterDocument] = useState('')
  const [filterTask, setFilterTask] = useState('')
  const [filterPhase, setFilterPhase] = useState('')
  const [file, setFile] = useState<File | null>(null)
  const selectedEvidenceId = searchParams.get('id')?.trim() ?? ''
  const evidenceParams = useMemo(
    () => ({
      space: optional(filterSpace) ?? defaultSpaceKey,
      document_id: optional(filterDocument) ?? undefined,
      task_key: optional(filterTask) ?? undefined,
      phase_key: optional(filterPhase) ?? undefined,
    }),
    [filterDocument, filterPhase, filterSpace, filterTask],
  )
  const evidenceQuery = useEvidence(evidenceParams)
  const createLink = useCreateEvidence()
  const createFile = useCreateFileEvidence()
  const selectedEvidenceQuery = useEvidenceItem(selectedEvidenceId)
  const items = useMemo(() => evidenceQuery.data?.evidence ?? [], [evidenceQuery.data?.evidence])
  const selectedEvidence =
    selectedEvidenceQuery.data ?? items.find((item) => item.id === selectedEvidenceId)
  const filtered = useMemo(() => {
    const needle = query.trim().toLowerCase()
    if (!needle) return items
    return items.filter((item) =>
      [item.title, item.document_id, item.task_key, item.phase_key, item.url, item.evidence_type]
        .filter(Boolean)
        .some((value) => value?.toLowerCase().includes(needle)),
    )
  }, [items, query])
  const linkCount = items.filter((item) => item.evidence_type === 'external_url').length
  const fileCount = items.filter((item) => item.evidence_type === 'uploaded_file').length
  const isSaving = createLink.isPending || createFile.isPending
  const saveError = formatFirstApiErrorForUser(
    [createLink.error, createFile.error],
    'Не удалось сохранить материал',
  )

  function resetForm() {
    setTitle('')
    setUrl('')
    setFile(null)
  }

  function resetFilters() {
    setQuery('')
    setFilterSpace(defaultSpaceKey)
    setFilterDocument('')
    setFilterTask('')
    setFilterPhase('')
  }

  function clearSelectedEvidence() {
    const nextParams = new URLSearchParams(searchParams)
    nextParams.delete('id')
    setSearchParams(nextParams)
  }

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const evidence = {
      space: optional(space) ?? defaultSpaceKey,
      document_id: optional(documentId),
      task_key: optional(task),
      phase_key: optional(phase),
      title: title.trim(),
    }

    if (mode === 'external_url') {
      createLink.mutate(
        {
          ...evidence,
          evidence_type: 'external_url',
          url: optional(url),
        },
        { onSuccess: resetForm },
      )
      return
    }

    if (!file) return
    createFile.mutate({ file, evidence }, { onSuccess: resetForm })
  }

  return (
    <div className="space-y-5">
      <section className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold">Материалы</h1>
          <p className="mt-1 max-w-3xl text-sm text-text-muted">
            Артефакты, ссылки и файлы, подтверждающие документ, задачу или фазу процесса.
          </p>
        </div>
      </section>

      <form onSubmit={handleSubmit} className="rounded-md border border-border bg-surface p-3">
        <div className="flex flex-wrap gap-2">
          <Button
            type="button"
            size="sm"
            variant={mode === 'external_url' ? 'default' : 'secondary'}
            onClick={() => setMode('external_url')}
          >
            <Link2 className="h-4 w-4" />
            Ссылка
          </Button>
          <Button
            type="button"
            size="sm"
            variant={mode === 'uploaded_file' ? 'default' : 'secondary'}
            onClick={() => setMode('uploaded_file')}
          >
            <Upload className="h-4 w-4" />
            Файл
          </Button>
        </div>

        <div className="mt-3 grid gap-3 md:grid-cols-2 xl:grid-cols-[10rem_13rem_minmax(0,1fr)_10rem_10rem]">
          <div className="space-y-1.5">
            <Label htmlFor="evidence-space">Пространство</Label>
            <Input
              id="evidence-space"
              value={space}
              onChange={(event) => setSpace(event.target.value.toUpperCase())}
              required
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="evidence-document">Документ</Label>
            <Input
              id="evidence-document"
              value={documentId}
              onChange={(event) => setDocumentId(event.target.value)}
              placeholder="ID или slug документа"
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="evidence-title">Название</Label>
            <Input
              id="evidence-title"
              value={title}
              onChange={(event) => setTitle(event.target.value)}
              placeholder="Название материала"
              required
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="evidence-task">Задача</Label>
            <Input
              id="evidence-task"
              value={task}
              onChange={(event) => setTask(event.target.value)}
              placeholder="Ключ задачи"
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="evidence-phase">Фаза</Label>
            <Input
              id="evidence-phase"
              value={phase}
              onChange={(event) => setPhase(event.target.value)}
              placeholder="Ключ фазы"
            />
          </div>
        </div>

        <div className="mt-3 grid gap-3 md:grid-cols-[minmax(0,1fr)_auto]">
          {mode === 'external_url' ? (
            <Input
              value={url}
              onChange={(event) => setUrl(event.target.value)}
              placeholder="https://..."
              aria-label="URL материала"
              required
            />
          ) : (
            <Input
              type="file"
              aria-label="Файл материала"
              onChange={(event) => setFile(event.target.files?.[0] ?? null)}
              required
            />
          )}
          <Button disabled={isSaving}>
            <CheckCircle2 className="h-4 w-4" />
            {isSaving ? 'Сохраняем...' : 'Добавить материал'}
          </Button>
        </div>
        {saveError && <p className="mt-2 text-sm text-danger">{saveError}</p>}
      </form>

      <section className="grid gap-3 rounded-md border border-border bg-surface p-3 md:grid-cols-2 xl:grid-cols-[minmax(0,1.3fr)_8rem_13rem_10rem_10rem_auto]">
        <Input
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="Поиск по материалам, документу или типу"
          aria-label="Поиск материалов"
        />
        <Input
          value={filterSpace}
          onChange={(event) => setFilterSpace(event.target.value.toUpperCase())}
          placeholder="SDLC"
          aria-label="Фильтр пространства"
        />
        <Input
          value={filterDocument}
          onChange={(event) => setFilterDocument(event.target.value)}
          placeholder="Документ"
          aria-label="Фильтр документа"
        />
        <Input
          value={filterTask}
          onChange={(event) => setFilterTask(event.target.value)}
          placeholder="Задача"
          aria-label="Фильтр задачи"
        />
        <Input
          value={filterPhase}
          onChange={(event) => setFilterPhase(event.target.value)}
          placeholder="Фаза"
          aria-label="Фильтр фазы"
        />
        <Button size="sm" variant="outline" type="button" onClick={resetFilters}>
          <RotateCcw className="h-4 w-4" />
          Сбросить
        </Button>
      </section>

      {selectedEvidenceId && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base">
              <FileText className="h-4 w-4 text-accent" />
              Выбранный материал
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {selectedEvidenceQuery.isLoading && <LoadingState message="Загружаем материал" />}
            {selectedEvidenceQuery.isError && (
              <ErrorState
                message={formatApiErrorForUser(
                  selectedEvidenceQuery.error,
                  'Не удалось открыть материал',
                )}
                onRetry={() => selectedEvidenceQuery.refetch()}
              />
            )}
            {!selectedEvidenceQuery.isLoading &&
              !selectedEvidenceQuery.isError &&
              selectedEvidence && (
                <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_18rem]">
                  <div className="space-y-3">
                    <div>
                      <div className="text-sm font-medium text-text-primary">
                        {selectedEvidence.title}
                      </div>
                      <div className="mt-1 text-xs text-text-muted">
                        {formatEvidenceType(selectedEvidence.evidence_type)} ·{' '}
                        {formatDateTime(selectedEvidence.created_at)}
                      </div>
                    </div>
                    {selectedEvidence.url && (
                      <a
                        href={selectedEvidence.url}
                        className="inline-flex items-center gap-2 text-sm text-accent hover:text-accent-hover"
                        target="_blank"
                        rel="noreferrer"
                      >
                        <ExternalLink className="h-4 w-4" />
                        Открыть ссылку
                      </a>
                    )}
                    <EvidenceTargetLinks item={selectedEvidence} />
                  </div>
                  <AttachmentMetadata item={selectedEvidence} />
                </div>
              )}
            {!selectedEvidenceQuery.isLoading &&
              !selectedEvidenceQuery.isError &&
              !selectedEvidence && <EmptyState message="Материал не найден в текущем доступе" />}
            <Button type="button" size="sm" variant="outline" onClick={clearSelectedEvidence}>
              Снять выделение
            </Button>
          </CardContent>
        </Card>
      )}

      <section className="grid gap-4 sm:grid-cols-3">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Всего</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-3">
              <CheckCircle2 className="h-5 w-5 text-success" />
              <span className="text-2xl font-semibold">{items.length}</span>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Ссылки</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-3">
              <ExternalLink className="h-5 w-5 text-accent" />
              <span className="text-2xl font-semibold">{linkCount}</span>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Файлы</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-3">
              <FileText className="h-5 w-5 text-warning" />
              <span className="text-2xl font-semibold">{fileCount}</span>
            </div>
          </CardContent>
        </Card>
      </section>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Реестр материалов</CardTitle>
        </CardHeader>
        <CardContent>
          {evidenceQuery.isLoading && <LoadingState message="Загружаем материалы" />}
          {evidenceQuery.isError && (
            <ErrorState
              message={formatApiErrorForUser(evidenceQuery.error, 'Не удалось загрузить материалы')}
              onRetry={() => evidenceQuery.refetch()}
            />
          )}
          {!evidenceQuery.isLoading && !evidenceQuery.isError && filtered.length === 0 && (
            <EmptyState message="Материалы не найдены" />
          )}
          {!evidenceQuery.isLoading && !evidenceQuery.isError && filtered.length > 0 && (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Материал</TableHead>
                  <TableHead>Документ</TableHead>
                  <TableHead>Задача</TableHead>
                  <TableHead>Фаза</TableHead>
                  <TableHead>Тип</TableHead>
                  <TableHead>Метаданные</TableHead>
                  <TableHead>Дата</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {filtered.map((item) => (
                  <TableRow
                    key={item.id}
                    className={item.id === selectedEvidenceId ? 'bg-accent/10' : undefined}
                  >
                    <TableCell className="font-medium">
                      {item.url ? (
                        <a
                          href={item.url}
                          className="text-accent hover:text-accent-hover"
                          target="_blank"
                          rel="noreferrer"
                        >
                          {item.title}
                        </a>
                      ) : (
                        item.title
                      )}
                    </TableCell>
                    <TableCell>
                      {item.document_id ? (
                        <Link
                          to={`/documents/${item.document_id}`}
                          className="block max-w-[13rem] truncate text-accent hover:text-accent-hover"
                          title={item.document_id}
                        >
                          {item.document_id}
                        </Link>
                      ) : (
                        <span className="text-text-muted">-</span>
                      )}
                    </TableCell>
                    <TableCell>
                      {item.task_key ? (
                        <Link
                          to={`/tasks/${item.task_key}`}
                          className="text-accent hover:text-accent-hover"
                        >
                          {item.task_key}
                        </Link>
                      ) : (
                        <span className="text-text-muted">-</span>
                      )}
                    </TableCell>
                    <TableCell>
                      {item.phase_key ? (
                        <Link
                          to={`/phases/${item.phase_key}`}
                          className="text-accent hover:text-accent-hover"
                        >
                          {item.phase_key}
                        </Link>
                      ) : (
                        <span className="text-text-muted">-</span>
                      )}
                    </TableCell>
                    <TableCell>{formatEvidenceType(item.evidence_type)}</TableCell>
                    <TableCell>
                      <AttachmentMetadata item={item} />
                    </TableCell>
                    <TableCell className="whitespace-nowrap text-xs text-text-muted">
                      {formatDateTime(item.created_at)}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
