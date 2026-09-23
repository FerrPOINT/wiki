import { FormEvent, useEffect, useMemo, useRef, useState } from 'react'
import { Link, useSearchParams } from 'react-router'
import {
  CheckCircle2,
  ChevronLeft,
  ChevronRight,
  Download,
  ExternalLink,
  FileText,
  Info,
  Link2,
  Plus,
  RotateCcw,
  Search,
  Upload,
  X,
} from 'lucide-react'
import {
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  EmptyState,
  ErrorState,
  Input,
  Label,
  LoadingState,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@sdlc/ui/ui'
import {
  useAttachment,
  useCreateEvidence,
  useCreateFileEvidence,
  useDownloadAttachment,
  useEvidence,
  useEvidenceItem,
  useSpaces,
} from '@/shared/api/hooks'
import { formatApiErrorForUser, formatFirstApiErrorForUser } from '@/shared/lib/api-error'
import { formatBytes, formatDateTime, formatEvidenceType } from '@/shared/lib/wiki-format'
import { defaultSpaceKey, resolveSpaceKey } from '@/shared/lib/space-selection'
import type { AttachmentDownload, Evidence } from '@/api/wiki'

type EvidenceMode = 'external_url' | 'uploaded_file'
const PAGE_SIZE = 20
const FILTER_KEYS = ['q', 'space', 'document_id', 'task_key', 'phase_key'] as const

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

function scopedDossierPath(path: string, spaceKey: string): string {
  return spaceKey === defaultSpaceKey ? path : `${path}?space=${encodeURIComponent(spaceKey)}`
}

function AttachmentMetadata({ item }: { item: Evidence }) {
  const hasAttachment = item.evidence_type === 'uploaded_file' && Boolean(item.attachment_id)
  const attachmentQuery = useAttachment(hasAttachment ? item.attachment_id : null)
  const downloadAttachment = useDownloadAttachment()
  const attachment = attachmentQuery.data
  const checksum = attachment?.checksum ?? item.checksum
  const fileName = attachment?.file_name ?? item.attachment_id ?? item.title

  if (!hasAttachment) return null

  function handleDownload() {
    if (!item.attachment_id) return
    downloadAttachment.mutate(item.attachment_id, {
      onSuccess: (download) => browserDownload(download, fileName),
    })
  }

  return (
    <div className="min-w-0 space-y-2 text-xs text-text-muted">
      <div className="min-w-0 break-all font-medium text-text-secondary">
        {attachmentQuery.isLoading ? 'Загружаем файл...' : fileName}
      </div>
      {attachment && (
        <div>
          {formatBytes(attachment.size_bytes)} · {attachment.content_type}
        </div>
      )}
      {attachmentQuery.isError && (
        <Button
          type="button"
          size="sm"
          variant="outline"
          className="min-h-10 sm:min-h-10"
          onClick={() => attachmentQuery.refetch()}
        >
          Повторить загрузку метаданных
        </Button>
      )}
      <div className="min-w-0 break-all font-mono">
        {checksum ?? 'Контрольная сумма недоступна'}
      </div>
      <Button
        type="button"
        size="sm"
        variant="outline"
        className="min-h-10 sm:min-h-10"
        onClick={handleDownload}
        disabled={downloadAttachment.isPending}
        aria-label={`Скачать ${item.title}`}
      >
        <Download className="h-4 w-4" />
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
    <div className="flex min-w-0 flex-wrap gap-x-3 text-sm">
      {item.document_id && (
        <Link
          to={`/documents/${item.document_id}`}
          className="inline-flex min-h-10 min-w-0 items-center break-all text-accent hover:text-accent-hover"
        >
          документ {item.document_id}
        </Link>
      )}
      {item.task_key && (
        <Link
          to={scopedDossierPath(`/tasks/${item.task_key}`, item.space_key)}
          className="inline-flex min-h-10 min-w-0 items-center break-all text-accent hover:text-accent-hover"
        >
          задача {item.task_key}
        </Link>
      )}
      {item.phase_key && (
        <Link
          to={scopedDossierPath(`/phases/${item.phase_key}`, item.space_key)}
          className="inline-flex min-h-10 min-w-0 items-center break-all text-accent hover:text-accent-hover"
        >
          фаза {item.phase_key}
        </Link>
      )}
    </div>
  )
}

function EvidenceTitle({ item, onSelect }: { item: Evidence; onSelect: (id: string) => void }) {
  if (item.url) {
    return (
      <a
        href={item.url}
        target="_blank"
        rel="noreferrer"
        className="inline-flex min-h-10 min-w-0 items-center break-words font-medium text-accent hover:text-accent-hover"
      >
        {item.title}
      </a>
    )
  }
  return (
    <button
      type="button"
      onClick={() => onSelect(item.id)}
      className="min-h-10 min-w-0 break-words text-left font-medium text-accent hover:text-accent-hover"
    >
      {item.title}
    </button>
  )
}

export function EvidencePage() {
  const [searchParams, setSearchParams] = useSearchParams()
  const initialSpace = searchParams.get('space')?.trim().toUpperCase() ?? ''
  const appliedQuery = searchParams.get('q') ?? ''
  const appliedSpace = searchParams.get('space') ?? ''
  const appliedDocument = searchParams.get('document_id') ?? ''
  const appliedTask = searchParams.get('task_key') ?? ''
  const appliedPhase = searchParams.get('phase_key') ?? ''
  const spacesQuery = useSpaces()
  const suggestedSpace = resolveSpaceKey(initialSpace, spacesQuery.data?.spaces ?? [])
  const [space, setSpace] = useState(initialSpace)
  const [documentId, setDocumentId] = useState(searchParams.get('document_id')?.trim() ?? '')
  const [task, setTask] = useState(searchParams.get('task_key')?.trim() ?? '')
  const [phase, setPhase] = useState(searchParams.get('phase_key')?.trim() ?? '')
  const [title, setTitle] = useState('')
  const [url, setUrl] = useState('')
  const [file, setFile] = useState<File | null>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)
  const registryRef = useRef<HTMLDivElement>(null)
  const [mode, setMode] = useState<EvidenceMode>('external_url')
  const [isCreateOpen, setCreateOpen] = useState(false)
  const [createdEvidence, setCreatedEvidence] = useState<Evidence | null>(null)
  const [targetError, setTargetError] = useState('')
  const [filterError, setFilterError] = useState('')
  const [filterQuery, setFilterQuery] = useState(appliedQuery)
  const [filterSpace, setFilterSpace] = useState(initialSpace)
  const [filterDocument, setFilterDocument] = useState(appliedDocument)
  const [filterTask, setFilterTask] = useState(appliedTask)
  const [filterPhase, setFilterPhase] = useState(appliedPhase)
  const [cursors, setCursors] = useState<(string | null)[]>([null])
  const selectedEvidenceId = searchParams.get('id')?.trim() ?? ''
  const evidenceParams = useMemo(
    () => ({
      q: optional(searchParams.get('q') ?? '') ?? undefined,
      space: optional(searchParams.get('space') ?? '') ?? undefined,
      document_id: optional(searchParams.get('document_id') ?? '') ?? undefined,
      task_key: optional(searchParams.get('task_key') ?? '') ?? undefined,
      phase_key: optional(searchParams.get('phase_key') ?? '') ?? undefined,
      cursor: cursors[cursors.length - 1] ?? undefined,
      limit: PAGE_SIZE,
    }),
    [searchParams, cursors],
  )
  const evidenceQuery = useEvidence(evidenceParams)
  const selectedEvidenceQuery = useEvidenceItem(selectedEvidenceId)
  const createLink = useCreateEvidence()
  const createFile = useCreateFileEvidence()
  const items = evidenceQuery.data?.evidence ?? []
  const nextCursor = evidenceQuery.data?.next_cursor
  const selectedEvidence =
    selectedEvidenceQuery.data ?? items.find((item) => item.id === selectedEvidenceId)
  const linkCount = items.filter((item) => item.evidence_type === 'external_url').length
  const fileCount = items.filter((item) => item.evidence_type === 'uploaded_file').length
  const isSaving = createLink.isPending || createFile.isPending
  const saveError = formatFirstApiErrorForUser(
    [createLink.error, createFile.error],
    'Не удалось сохранить материал',
  )

  useEffect(() => {
    if (!space && suggestedSpace) setSpace(suggestedSpace)
  }, [space, suggestedSpace])

  useEffect(() => {
    setFilterQuery(appliedQuery)
    setFilterSpace(appliedSpace)
    setFilterDocument(appliedDocument)
    setFilterTask(appliedTask)
    setFilterPhase(appliedPhase)
    setCursors([null])
  }, [appliedQuery, appliedSpace, appliedDocument, appliedTask, appliedPhase])

  function resetForm() {
    setTitle('')
    setUrl('')
    setFile(null)
    if (fileInputRef.current) fileInputRef.current.value = ''
    setCreateOpen(false)
    setTargetError('')
    setCursors([null])
  }

  function handleSaved(evidence: Evidence) {
    resetForm()
    setCreatedEvidence(evidence)
  }

  function applyFilters(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (optional(filterSpace)?.length === 1) {
      setFilterError('Ключ пространства должен содержать минимум 2 символа')
      return
    }
    setFilterError('')
    const nextParams = new URLSearchParams(searchParams)
    const values: Record<(typeof FILTER_KEYS)[number], string> = {
      q: filterQuery,
      space: filterSpace.toUpperCase(),
      document_id: filterDocument,
      task_key: filterTask,
      phase_key: filterPhase,
    }
    FILTER_KEYS.forEach((key) => {
      const value = optional(values[key])
      if (value) nextParams.set(key, value)
      else nextParams.delete(key)
    })
    setCursors([null])
    setSearchParams(nextParams)
  }

  function resetFilters() {
    setFilterError('')
    setFilterQuery('')
    setFilterSpace('')
    setFilterDocument('')
    setFilterTask('')
    setFilterPhase('')
    const nextParams = new URLSearchParams(searchParams)
    FILTER_KEYS.forEach((key) => nextParams.delete(key))
    setCursors([null])
    setSearchParams(nextParams)
  }

  function selectEvidence(id: string | null) {
    const nextParams = new URLSearchParams(searchParams)
    if (id) nextParams.set('id', id)
    else nextParams.delete('id')
    setSearchParams(nextParams)
  }

  function changePage(nextCursors: (string | null)[]) {
    setCursors(nextCursors)
    registryRef.current?.scrollIntoView?.({ block: 'start' })
  }

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (isSaving) return
    if (!optional(documentId) && !optional(task) && !optional(phase)) {
      setTargetError('Укажите документ, задачу или фазу')
      return
    }
    setTargetError('')
    setCreatedEvidence(null)
    const evidence = {
      space: space.trim().toUpperCase(),
      document_id: optional(documentId),
      task_key: optional(task),
      phase_key: optional(phase),
      title: title.trim(),
    }
    if (mode === 'external_url') {
      createLink.mutate(
        { ...evidence, evidence_type: 'external_url', url: optional(url) },
        { onSuccess: handleSaved },
      )
      return
    }
    if (file) createFile.mutate({ file, evidence }, { onSuccess: handleSaved })
  }

  return (
    <div className="space-y-5">
      <header className="flex flex-wrap items-center justify-between gap-3">
        <h1 className="text-2xl font-bold">Материалы</h1>
        <Button
          type="button"
          className="min-h-10 sm:min-h-10"
          onClick={() => setCreateOpen((open) => !open)}
          disabled={isSaving}
          aria-expanded={isCreateOpen}
          aria-controls={isCreateOpen ? 'evidence-create-form' : undefined}
        >
          <Plus className="h-4 w-4" />
          {isCreateOpen ? 'Закрыть форму' : 'Добавить материал'}
        </Button>
      </header>

      {createdEvidence && (
        <div
          role="status"
          className="flex flex-wrap items-center gap-2 border-y border-border py-3 text-sm text-text-secondary"
        >
          <CheckCircle2 className="h-4 w-4 text-success" />
          <span className="mr-auto min-w-0 break-words">
            Материал «{createdEvidence.title}» добавлен
          </span>
          <Button
            type="button"
            size="sm"
            variant="outline"
            className="min-h-10 sm:min-h-10"
            onClick={() => selectEvidence(createdEvidence.id)}
          >
            Открыть
          </Button>
          <Button
            type="button"
            size="icon"
            variant="ghost"
            className="h-10 w-10"
            aria-label="Закрыть уведомление"
            title="Закрыть уведомление"
            onClick={() => setCreatedEvidence(null)}
          >
            <X className="h-4 w-4" />
          </Button>
        </div>
      )}

      {isCreateOpen && (
        <form
          id="evidence-create-form"
          onSubmit={handleSubmit}
          className="space-y-3 border-y border-border py-4"
        >
          <div className="flex flex-wrap gap-2" role="group" aria-label="Тип материала">
            <Button
              type="button"
              size="sm"
              className="min-h-10 sm:min-h-10"
              variant={mode === 'external_url' ? 'default' : 'secondary'}
              onClick={() => setMode('external_url')}
              disabled={isSaving}
              aria-pressed={mode === 'external_url'}
            >
              <Link2 className="h-4 w-4" />
              Ссылка
            </Button>
            <Button
              type="button"
              size="sm"
              className="min-h-10 sm:min-h-10"
              variant={mode === 'uploaded_file' ? 'default' : 'secondary'}
              onClick={() => setMode('uploaded_file')}
              disabled={isSaving}
              aria-pressed={mode === 'uploaded_file'}
            >
              <Upload className="h-4 w-4" />
              Файл
            </Button>
          </div>
          <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-[10rem_13rem_minmax(0,1fr)_10rem_10rem]">
            <div className="space-y-1.5">
              <Label htmlFor="evidence-space">Пространство</Label>
              <Input
                id="evidence-space"
                className="min-h-10"
                value={space}
                onChange={(event) => setSpace(event.target.value.toUpperCase())}
                disabled={isSaving}
                required
              />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="evidence-document">Документ</Label>
              <Input
                id="evidence-document"
                className="min-h-10"
                value={documentId}
                onChange={(event) => {
                  setDocumentId(event.target.value)
                  setTargetError('')
                }}
                disabled={isSaving}
                placeholder="ID или slug"
              />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="evidence-title">Название</Label>
              <Input
                id="evidence-title"
                className="min-h-10"
                value={title}
                onChange={(event) => setTitle(event.target.value)}
                disabled={isSaving}
                required
              />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="evidence-task">Задача</Label>
              <Input
                id="evidence-task"
                className="min-h-10"
                value={task}
                onChange={(event) => {
                  setTask(event.target.value)
                  setTargetError('')
                }}
                disabled={isSaving}
              />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="evidence-phase">Фаза</Label>
              <Input
                id="evidence-phase"
                className="min-h-10"
                value={phase}
                onChange={(event) => {
                  setPhase(event.target.value)
                  setTargetError('')
                }}
                disabled={isSaving}
              />
            </div>
          </div>
          <div className="grid gap-3 sm:grid-cols-[minmax(0,1fr)_auto]">
            {mode === 'external_url' ? (
              <Input
                type="url"
                className="min-h-10"
                value={url}
                onChange={(event) => setUrl(event.target.value)}
                disabled={isSaving}
                placeholder="https://..."
                aria-label="URL материала"
                required
              />
            ) : (
              <Input
                ref={fileInputRef}
                type="file"
                className="min-h-10"
                aria-label="Файл материала"
                onChange={(event) => setFile(event.target.files?.[0] ?? null)}
                disabled={isSaving}
                required
              />
            )}
            <Button className="min-h-10 sm:min-h-10" disabled={isSaving}>
              {isSaving ? 'Сохраняем...' : 'Сохранить материал'}
            </Button>
          </div>
          {targetError && (
            <p role="alert" className="text-sm text-danger">
              {targetError}
            </p>
          )}
          {saveError && (
            <p role="alert" className="text-sm text-danger">
              {saveError}
            </p>
          )}
        </form>
      )}

      <form
        onSubmit={applyFilters}
        aria-label="Фильтры материалов"
        className="grid gap-2 border-y border-border py-3 sm:grid-cols-2 lg:grid-cols-3 2xl:grid-cols-[minmax(12rem,1fr)_8rem_12rem_10rem_10rem_auto_auto]"
      >
        <Input
          className="min-h-10"
          value={filterQuery}
          onChange={(event) => setFilterQuery(event.target.value)}
          maxLength={200}
          placeholder="Поиск по всем материалам"
          aria-label="Поиск материалов"
        />
        <Input
          className="min-h-10"
          value={filterSpace}
          onChange={(event) => {
            setFilterSpace(event.target.value.toUpperCase())
            setFilterError('')
          }}
          minLength={2}
          placeholder="Пространство"
          aria-label="Фильтр пространства"
        />
        <Input
          className="min-h-10"
          value={filterDocument}
          onChange={(event) => setFilterDocument(event.target.value)}
          placeholder="Документ"
          aria-label="Фильтр документа"
        />
        <Input
          className="min-h-10"
          value={filterTask}
          onChange={(event) => setFilterTask(event.target.value)}
          placeholder="Задача"
          aria-label="Фильтр задачи"
        />
        <Input
          className="min-h-10"
          value={filterPhase}
          onChange={(event) => setFilterPhase(event.target.value)}
          placeholder="Фаза"
          aria-label="Фильтр фазы"
        />
        <Button type="submit" size="sm" className="min-h-10 sm:min-h-10">
          <Search className="h-4 w-4" />
          Найти
        </Button>
        <Button
          type="button"
          size="sm"
          variant="outline"
          className="min-h-10 sm:min-h-10"
          onClick={resetFilters}
        >
          <RotateCcw className="h-4 w-4" />
          Сбросить
        </Button>
        {filterError && (
          <p role="alert" className="text-sm text-danger 2xl:col-span-full">
            {filterError}
          </p>
        )}
      </form>

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
                <div
                  className={
                    selectedEvidence.evidence_type === 'uploaded_file'
                      ? 'grid min-w-0 gap-4 lg:grid-cols-[minmax(0,1fr)_18rem]'
                      : 'min-w-0'
                  }
                >
                  <div className="min-w-0 space-y-3">
                    <div className="break-words font-medium">{selectedEvidence.title}</div>
                    <div className="text-xs text-text-muted">
                      {selectedEvidence.space_key} ·{' '}
                      {formatEvidenceType(selectedEvidence.evidence_type)} ·{' '}
                      {formatDateTime(selectedEvidence.created_at)}
                    </div>
                    {selectedEvidence.url && (
                      <a
                        href={selectedEvidence.url}
                        target="_blank"
                        rel="noreferrer"
                        className="inline-flex min-h-10 items-center gap-2 text-sm text-accent hover:text-accent-hover"
                      >
                        <ExternalLink className="h-4 w-4" />
                        Открыть ссылку
                      </a>
                    )}
                    <EvidenceTargetLinks item={selectedEvidence} />
                  </div>
                  {selectedEvidence.evidence_type === 'uploaded_file' && (
                    <AttachmentMetadata item={selectedEvidence} />
                  )}
                </div>
              )}
            {!selectedEvidenceQuery.isLoading &&
              !selectedEvidenceQuery.isError &&
              !selectedEvidence && <EmptyState message="Материал не найден в текущем доступе" />}
            <Button
              type="button"
              size="sm"
              variant="outline"
              className="min-h-10 sm:min-h-10"
              onClick={() => selectEvidence(null)}
            >
              Снять выделение
            </Button>
          </CardContent>
        </Card>
      )}

      {!evidenceQuery.isLoading && !evidenceQuery.isError && items.length > 0 && (
        <section
          aria-label="Сводка текущей страницы"
          className="flex flex-wrap gap-x-5 gap-y-1 border-b border-border pb-3 text-sm text-text-secondary"
        >
          <span>
            Показано <strong>{items.length}</strong> (лимит {PAGE_SIZE})
          </span>
          <span>Ссылки: {linkCount}</span>
          <span>Файлы: {fileCount}</span>
        </section>
      )}

      <Card ref={registryRef} className="scroll-mt-16">
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
          {!evidenceQuery.isLoading && !evidenceQuery.isError && items.length === 0 && (
            <EmptyState message="Материалы не найдены" />
          )}
          {!evidenceQuery.isLoading && !evidenceQuery.isError && items.length > 0 && (
            <>
              <ul className="divide-y divide-border xl:hidden" aria-label="Материалы">
                {items.map((item) => (
                  <li key={item.id} className="min-w-0 py-3 first:pt-0 last:pb-0">
                    <div className="flex min-w-0 items-start justify-between gap-2">
                      <EvidenceTitle item={item} onSelect={selectEvidence} />
                      <Button
                        type="button"
                        size="icon"
                        variant="ghost"
                        className="h-10 w-10 shrink-0 sm:min-h-10 sm:min-w-10"
                        aria-label={`Открыть материал ${item.title}`}
                        title="Открыть материал"
                        onClick={() => selectEvidence(item.id)}
                      >
                        <Info className="h-4 w-4" />
                      </Button>
                    </div>
                    <div className="mt-1 text-xs text-text-muted">
                      {item.space_key} · {formatEvidenceType(item.evidence_type)} ·{' '}
                      {formatDateTime(item.created_at)}
                    </div>
                    <div className="mt-2">
                      <EvidenceTargetLinks item={item} />
                    </div>
                  </li>
                ))}
              </ul>
              <div className="hidden xl:block">
                <Table className="table-fixed">
                  <TableHeader>
                    <TableRow>
                      <TableHead className="w-[25%]">Материал</TableHead>
                      <TableHead>Связи</TableHead>
                      <TableHead className="w-28">Тип</TableHead>
                      <TableHead className="w-36">Дата</TableHead>
                      <TableHead className="w-12">
                        <span className="sr-only">Действия</span>
                      </TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {items.map((item) => (
                      <TableRow
                        key={item.id}
                        className={item.id === selectedEvidenceId ? 'bg-accent/10' : undefined}
                      >
                        <TableCell className="min-w-0">
                          <EvidenceTitle item={item} onSelect={selectEvidence} />
                        </TableCell>
                        <TableCell className="min-w-0">
                          <EvidenceTargetLinks item={item} />
                        </TableCell>
                        <TableCell className="text-sm">
                          {formatEvidenceType(item.evidence_type)}
                        </TableCell>
                        <TableCell className="text-xs text-text-muted">
                          {formatDateTime(item.created_at)}
                        </TableCell>
                        <TableCell>
                          <Button
                            type="button"
                            size="icon"
                            variant="ghost"
                            className="h-10 w-10 shrink-0 sm:min-h-10 sm:min-w-10"
                            aria-label={`Открыть материал ${item.title}`}
                            title="Открыть материал"
                            onClick={() => selectEvidence(item.id)}
                          >
                            <Info className="h-4 w-4" />
                          </Button>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>
            </>
          )}
          {(cursors.length > 1 || nextCursor) && (
            <nav
              aria-label="Страницы материалов"
              className="mt-4 flex flex-wrap items-center gap-3 border-t border-border pt-3"
            >
              <span className="mr-auto text-sm text-text-secondary">Страница {cursors.length}</span>
              <Button
                type="button"
                variant="outline"
                className="h-10"
                disabled={
                  cursors.length === 1 || evidenceQuery.isLoading || evidenceQuery.isFetching
                }
                onClick={() => changePage(cursors.slice(0, -1))}
              >
                <ChevronLeft className="h-4 w-4" />
                Назад
              </Button>
              <Button
                type="button"
                variant="outline"
                className="h-10"
                disabled={
                  !nextCursor ||
                  evidenceQuery.isLoading ||
                  evidenceQuery.isFetching ||
                  evidenceQuery.isError
                }
                onClick={() => nextCursor && changePage([...cursors, nextCursor])}
              >
                Далее
                <ChevronRight className="h-4 w-4" />
              </Button>
            </nav>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
