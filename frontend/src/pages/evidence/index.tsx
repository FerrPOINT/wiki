import { ContentSection } from '@pdlc/ui/ui'
import { downloadBlob } from '@pdlc/ui/lib'
import { useProjectRouteRef } from '@/widgets/namespace-context'
import { FormEvent, useEffect, useMemo, useRef, useState } from 'react'
import {
  PageNavigation,
  RadioGroup,
  PageHeader,
  NamespaceLink as Link,
  ResourceList,
  ResourceListRow,
  ResourceListTitle,
  ResourceListDescription,
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  EmptyState,
  ErrorState,
  Input,
  LoadingState,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  Notice,
  TextLink,
  FormField,
} from '@pdlc/ui/ui'
import { useSearchParams } from 'react-router'
import { Download, ExternalLink, FileText, Info, Plus, RotateCcw, Search, X } from 'lucide-react'

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
import type { Evidence } from '@/api/wiki'

type EvidenceMode = 'external_url' | 'uploaded_file'
const PAGE_SIZE = 20
const FILTER_KEYS = ['q', 'space', 'document_id', 'task_key', 'phase_key'] as const

function optional(value: string) {
  const trimmed = value.trim()
  return trimmed.length > 0 ? trimmed : null
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
      onSuccess: (download) => downloadBlob(download.blob, download.fileName ?? fileName),
    })
  }

  return (
    <ResourceListDescription asChild className="min-w-0 space-y-2">
      <div>
        <ResourceListTitle asChild className="min-w-0 break-all">
          <div>{attachmentQuery.isLoading ? 'Загружаем файл...' : fileName}</div>
        </ResourceListTitle>
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

          onClick={handleDownload}
          disabled={downloadAttachment.isPending}
          aria-label={`Скачать ${item.title}`}
        >
          <Download className="h-4 w-4" />
          {downloadAttachment.isPending ? 'Скачиваем...' : 'Скачать'}
        </Button>
        {downloadAttachment.isError && (
          <Notice role="alert" appearance="inline" tone="danger">
            {formatApiErrorForUser(downloadAttachment.error, 'Не удалось скачать файл')}
          </Notice>
        )}
      </div>
    </ResourceListDescription>
  )
}

function EvidenceTargetLinks({ item }: { item: Evidence }) {
  return (
    <div className="flex min-w-0 flex-wrap gap-x-3 text-sm">
      {item.document_id && (
        <TextLink asChild>
          <Link
            to={`/documents/${item.document_id}`}
            className="inline-flex min-h-10 min-w-0 items-center break-all"
          >
            документ {item.document_id}
          </Link>
        </TextLink>
      )}
      {item.task_key && (
        <TextLink asChild>
          <Link
            to={scopedDossierPath(`/tasks/${item.task_key}`, item.space_key)}
            className="inline-flex min-h-10 min-w-0 items-center break-all"
          >
            задача {item.task_key}
          </Link>
        </TextLink>
      )}
      {item.phase_key && (
        <TextLink asChild>
          <Link
            to={scopedDossierPath(`/phases/${item.phase_key}`, item.space_key)}
            className="inline-flex min-h-10 min-w-0 items-center break-all"
          >
            фаза {item.phase_key}
          </Link>
        </TextLink>
      )}
    </div>
  )
}

function EvidenceTitle({ item, onSelect }: { item: Evidence; onSelect: (id: string) => void }) {
  if (item.url) {
    return (
      <TextLink
        href={item.url}
        target="_blank"
        rel="noreferrer"
        className="inline-flex min-h-10 min-w-0 items-center break-words font-medium"
      >
        {item.title}
      </TextLink>
    )
  }
  return (
    <Button
      variant="link"
      type="button"
      onClick={() => onSelect(item.id)}
      className="min-w-0 break-words text-left"
    >
      {item.title}
    </Button>
  )
}

export function EvidencePage() {
  const namespace = useProjectRouteRef()
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
      <PageHeader
        title={'Материалы'}
        actions={
          <>
            <Button
              type="button"

              onClick={() => setCreateOpen((open) => !open)}
              disabled={isSaving}
              aria-expanded={isCreateOpen}
              aria-controls={isCreateOpen ? 'evidence-create-form' : undefined}
            >
              <Plus className="h-4 w-4" />
              {isCreateOpen ? 'Закрыть форму' : 'Добавить материал'}
            </Button>
          </>
        }
      />

      {createdEvidence && (
        <Notice
          role="status"
          tone="success"
          actions={
            <>
              <Button
                type="button"
                size="sm"
                variant="outline"
                onClick={() => selectEvidence(createdEvidence.id)}
              >
                Открыть
              </Button>
              <Button
                type="button"
                size="icon"
                variant="ghost"
                aria-label="Закрыть уведомление"
                title="Закрыть уведомление"
                onClick={() => setCreatedEvidence(null)}
              >
                <X className="h-4 w-4" />
              </Button>
            </>
          }
        >
          Материал «{createdEvidence.title}» добавлен
        </Notice>
      )}

      {isCreateOpen && (
        <ContentSection asChild dividers="both">
          <form id="evidence-create-form" onSubmit={handleSubmit} className="space-y-3">
            <RadioGroup
              label="Тип материала"
              appearance="segmented"
              options={[
                { value: 'external_url', label: 'Ссылка' },
                { value: 'uploaded_file', label: 'Файл' },
              ]}
              value={mode}
              onValueChange={(value) => setMode(value as EvidenceMode)}
              disabled={isSaving}
            />
            <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-[10rem_13rem_minmax(0,1fr)_10rem_10rem]">
              <FormField id="evidence-space" label={'Пространство'}>
                {(baseFieldAttrs) => (
                  <Input
                    {...baseFieldAttrs}
                    readOnly={Boolean(namespace.ref)}

                    value={space}
                    onChange={(event) => setSpace(event.target.value.toUpperCase())}
                    disabled={isSaving}
                    required
                  />
                )}
              </FormField>
              <FormField id="evidence-document" label={'Документ'}>
                {(baseFieldAttrs) => (
                  <Input
                    {...baseFieldAttrs}
                    value={documentId}
                    onChange={(event) => {
                      setDocumentId(event.target.value)
                      setTargetError('')
                    }}
                    disabled={isSaving}
                    placeholder="ID или slug"
                  />
                )}
              </FormField>
              <FormField id="evidence-title" label={'Название'}>
                {(baseFieldAttrs) => (
                  <Input
                    {...baseFieldAttrs}
                    value={title}
                    onChange={(event) => setTitle(event.target.value)}
                    disabled={isSaving}
                    required
                  />
                )}
              </FormField>
              <FormField id="evidence-task" label={'Задача'}>
                {(baseFieldAttrs) => (
                  <Input
                    {...baseFieldAttrs}
                    value={task}
                    onChange={(event) => {
                      setTask(event.target.value)
                      setTargetError('')
                    }}
                    disabled={isSaving}
                  />
                )}
              </FormField>
              <FormField id="evidence-phase" label={'Фаза'}>
                {(baseFieldAttrs) => (
                  <Input
                    {...baseFieldAttrs}
                    value={phase}
                    onChange={(event) => {
                      setPhase(event.target.value)
                      setTargetError('')
                    }}
                    disabled={isSaving}
                  />
                )}
              </FormField>
            </div>
            <div className="grid gap-3 sm:grid-cols-[minmax(0,1fr)_auto]">
              {mode === 'external_url' ? (
                <Input
                  type="url"

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

                  aria-label="Файл материала"
                  onChange={(event) => setFile(event.target.files?.[0] ?? null)}
                  disabled={isSaving}
                  required
                />
              )}
              <Button disabled={isSaving}>
                {isSaving ? 'Сохраняем...' : 'Сохранить материал'}
              </Button>
            </div>
            {targetError && (
              <Notice role="alert" appearance="inline" tone="danger">
                {targetError}
              </Notice>
            )}
            {saveError && (
              <Notice role="alert" appearance="inline" tone="danger">
                {saveError}
              </Notice>
            )}
          </form>
        </ContentSection>
      )}

      <ContentSection asChild dividers="both" density="compact">
        <form
          onSubmit={applyFilters}
          aria-label="Фильтры материалов"
          className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3 2xl:grid-cols-[minmax(12rem,1fr)_8rem_12rem_10rem_10rem_auto_auto]"
        >
          <Input
            value={filterQuery}
            onChange={(event) => setFilterQuery(event.target.value)}
            maxLength={200}
            placeholder="Поиск по всем материалам"
            aria-label="Поиск материалов"
          />
          <Input
            value={filterSpace}
            readOnly={Boolean(namespace.ref)}
            onChange={(event) => {
              setFilterSpace(event.target.value.toUpperCase())
              setFilterError('')
            }}
            minLength={2}
            placeholder="Пространство"
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
          <Button type="submit" size="sm">
            <Search className="h-4 w-4" />
            Найти
          </Button>
          <Button
            type="button"
            size="sm"
            variant="outline"

            onClick={resetFilters}
          >
            <RotateCcw className="h-4 w-4" />
            Сбросить
          </Button>
          {filterError && (
            <Notice role="alert" className="2xl:col-span-full" appearance="inline" tone="danger">
              {filterError}
            </Notice>
          )}
        </form>
      </ContentSection>

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
                      <TextLink
                        href={selectedEvidence.url}
                        target="_blank"
                        rel="noreferrer"
                        className="inline-flex min-h-10 items-center gap-2 text-sm"
                      >
                        <ExternalLink className="h-4 w-4" />
                        Открыть ссылку
                      </TextLink>
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

              onClick={() => selectEvidence(null)}
            >
              Снять выделение
            </Button>
          </CardContent>
        </Card>
      )}

      {!evidenceQuery.isLoading && !evidenceQuery.isError && items.length > 0 && (
        <ContentSection asChild dividers="end" density="compact">
          <section
            aria-label="Сводка текущей страницы"
            className="flex flex-wrap gap-x-5 gap-y-1 text-sm text-text-secondary"
          >
            <span>
              Показано <strong>{items.length}</strong> (лимит {PAGE_SIZE})
            </span>
            <span>Ссылки: {linkCount}</span>
            <span>Файлы: {fileCount}</span>
          </section>
        </ContentSection>
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
              <ResourceList asChild>
                <ul className="xl:hidden" aria-label="Материалы">
                  {items.map((item) => (
                    <ResourceListRow asChild key={item.id}>
                      <li className="min-w-0">
                        <div className="flex min-w-0 items-start justify-between gap-2">
                          <EvidenceTitle item={item} onSelect={selectEvidence} />
                          <Button
                            type="button"
                            size="icon"
                            variant="ghost"
                            className="shrink-0"
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
                    </ResourceListRow>
                  ))}
                </ul>
              </ResourceList>
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
                      <TableRow key={item.id} selected={item.id === selectedEvidenceId}>
                        <TableCell className="min-w-0">
                          <EvidenceTitle item={item} onSelect={selectEvidence} />
                        </TableCell>
                        <TableCell className="min-w-0">
                          <EvidenceTargetLinks item={item} />
                        </TableCell>
                        <TableCell className="text-sm">
                          {formatEvidenceType(item.evidence_type)}
                        </TableCell>
                        <TableCell className="text-text-muted" size="sm">
                          {formatDateTime(item.created_at)}
                        </TableCell>
                        <TableCell>
                          <Button
                            type="button"
                            size="icon"
                            variant="ghost"
                            className="shrink-0"
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
            <PageNavigation
              page={cursors.length}
              hasNext={
                !!nextCursor &&
                !evidenceQuery.isLoading &&
                !evidenceQuery.isFetching &&
                !evidenceQuery.isError
              }
              onPageChange={(nextPage) => {
                if (nextPage < cursors.length) changePage(cursors.slice(0, -1))
                else if (nextCursor) changePage([...cursors, nextCursor])
              }}
              label="Страницы материалов"
              previousLabel="Назад"
              nextLabel="Далее"
              pending={evidenceQuery.isLoading || evidenceQuery.isFetching}
              summary={<>Страница {cursors.length}</>}
              separated
              className="mt-4"
            />
          )}
        </CardContent>
      </Card>
    </div>
  )
}
