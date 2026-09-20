import { type FormEvent, useState } from 'react'
import { Link, useParams, useSearchParams } from 'react-router'
import { FileCheck2, FileText, Link2, Search } from 'lucide-react'
import {
  defaultSpaceKey,
  useLinkPhaseDocument,
  usePhase,
  usePhases,
  useSpaces,
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

function evidencePhasePath(spaceKey: string, phaseKey: string, evidenceId: string): string {
  const params = new URLSearchParams({ space: spaceKey, phase_key: phaseKey, id: evidenceId })
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
      <Label htmlFor="phase-space-selector">Пространство</Label>
      <select
        id="phase-space-selector"
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

export function PhaseDossiersPage() {
  const [selectedSpaceKey, setSelectedSpaceKey, spacesQuery] = useSelectedSpaceKey()
  const phasesQuery = usePhases(selectedSpaceKey)
  const phases = phasesQuery.data?.phases ?? []
  const [search, setSearch] = useState('')
  const [page, setPage] = useState(1)
  const needle = search.trim().toLocaleLowerCase('ru')
  const filteredPhases = needle
    ? phases.filter((phase) =>
        `${phase.phase_key} ${phase.title ?? ''}`.toLocaleLowerCase('ru').includes(needle),
      )
    : phases
  const totalPages = Math.max(1, Math.ceil(filteredPhases.length / pageSize))
  const currentPage = Math.min(page, totalPages)
  const visiblePhases = filteredPhases.slice((currentPage - 1) * pageSize, currentPage * pageSize)

  return (
    <div className="space-y-5">
      <section className="flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold">Фазы процесса</h1>
          <p className="mt-1 max-w-3xl text-sm text-text-muted">
            Каждая завершённая фаза должна иметь документы и материалы, достаточные для аудита.
          </p>
        </div>
        <SpaceSelector
          value={selectedSpaceKey}
          onChange={(spaceKey) => {
            setSelectedSpaceKey(spaceKey)
            setSearch('')
            setPage(1)
          }}
        />
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
      {selectedSpaceKey && phasesQuery.isLoading && <LoadingState message="Загружаем фазы" />}
      {selectedSpaceKey && phasesQuery.isError && (
        <ErrorState
          message={formatApiErrorForUser(phasesQuery.error, 'Не удалось загрузить фазы')}
          onRetry={() => phasesQuery.refetch()}
        />
      )}
      {selectedSpaceKey &&
        !phasesQuery.isLoading &&
        !phasesQuery.isError &&
        phases.length === 0 && (
          <EmptyState message="Документы и материалы ещё не связаны с фазами" />
        )}
      {selectedSpaceKey && !phasesQuery.isLoading && !phasesQuery.isError && phases.length > 0 && (
        <section aria-label="Список фаз" className="space-y-3">
          <div className="relative max-w-md">
            <Search
              className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-text-muted"
              aria-hidden
            />
            <Input
              type="search"
              aria-label="Найти фазу"
              className="min-h-10 pl-9"
              placeholder="Ключ или название"
              value={search}
              onChange={(event) => {
                setSearch(event.target.value)
                setPage(1)
              }}
            />
          </div>
          <p role="status" className="text-xs text-text-muted">
            Показано {visiblePhases.length} из {filteredPhases.length} фаз
          </p>
          {filteredPhases.length === 0 ? (
            <EmptyState message="По запросу фазы не найдены" />
          ) : (
            <ul className="divide-y divide-border border-y border-border">
              {visiblePhases.map((phase) => (
                <li key={phase.phase_key}>
                  <Link
                    to={scopedPath(`/phases/${phase.phase_key}`, selectedSpaceKey)}
                    className="flex min-h-16 min-w-0 flex-wrap items-center justify-between gap-x-4 gap-y-1 px-2 py-3 hover:bg-surface-raised focus-visible:outline-2 focus-visible:outline-accent"
                  >
                    <span className="min-w-0">
                      <span className="block break-words text-sm font-medium text-text-primary">
                        {phase.title ?? phase.phase_key}
                      </span>
                      {phase.title && (
                        <span className="block text-xs text-text-muted">{phase.phase_key}</span>
                      )}
                    </span>
                    <span className="shrink-0 text-xs text-text-muted">
                      Документы: {phase.document_count} · Материалы: {phase.evidence_count}
                    </span>
                  </Link>
                </li>
              ))}
            </ul>
          )}
          {totalPages > 1 && (
            <nav aria-label="Страницы фаз" className="flex items-center justify-end gap-2">
              <Button
                type="button"
                size="sm"
                variant="outline"
                className="min-h-10"
                disabled={currentPage === 1}
                onClick={() => setPage(currentPage - 1)}
              >
                Назад
              </Button>
              <span className="text-sm text-text-muted">
                {currentPage} / {totalPages}
              </span>
              <Button
                type="button"
                size="sm"
                variant="outline"
                className="min-h-10"
                disabled={currentPage === totalPages}
                onClick={() => setPage(currentPage + 1)}
              >
                Далее
              </Button>
            </nav>
          )}
        </section>
      )}
    </div>
  )
}

export function PhaseDossierPage() {
  const { phaseId = 'implementation' } = useParams()
  const [selectedSpaceKey, setSelectedSpaceKey, spacesQuery] = useSelectedSpaceKey()
  const phaseQuery = usePhase(phaseId, selectedSpaceKey)
  const linkDocument = useLinkPhaseDocument()
  const [documentId, setDocumentId] = useState('')
  const [linkMessage, setLinkMessage] = useState('')
  const phase = phaseQuery.data

  function handleLinkDocument(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const trimmedDocumentId = documentId.trim()
    if (!trimmedDocumentId || linkDocument.isPending) return
    setLinkMessage('')
    linkDocument.mutate(
      {
        spaceKey: selectedSpaceKey,
        phaseKey: phaseId,
        body: { document_id: trimmedDocumentId },
      },
      {
        onSuccess: () => {
          setDocumentId('')
          setLinkMessage('Документ привязан к фазе')
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
  if (phaseQuery.isLoading) return <LoadingState message="Загружаем фазу" />
  if (phaseQuery.isError || !phase) {
    return (
      <ErrorState
        message={formatApiErrorForUser(phaseQuery.error, 'Не удалось открыть фазу')}
        onRetry={() => phaseQuery.refetch()}
      />
    )
  }

  return (
    <div className="space-y-5">
      <section className="space-y-3">
        <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
          <div className="min-w-0">
            <Link
              to={scopedPath('/phases', selectedSpaceKey)}
              className="inline-flex min-h-10 items-center text-sm text-accent hover:text-accent-hover"
            >
              К фазам
            </Link>
            <h1 className="mt-2 break-words text-2xl font-bold">
              {phase.title ?? phase.phase_key}
            </h1>
            {phase.title && phase.title !== phase.phase_key && (
              <p className="mt-1 break-words text-sm text-text-muted">{phase.phase_key}</p>
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
          <span>Документы: {phase.document_count}</span>
          <span>Материалы: {phase.evidence_count}</span>
        </div>
      </section>

      {spacesQuery.isError && (
        <ErrorState
          message={formatApiErrorForUser(spacesQuery.error, 'Не удалось загрузить пространства')}
          onRetry={() => spacesQuery.refetch()}
        />
      )}

      <div className="grid gap-6 xl:grid-cols-2">
        <section className="min-w-0 space-y-3" aria-labelledby="phase-documents-title">
          <h2 id="phase-documents-title" className="text-base font-semibold">
            Документы фазы
          </h2>
          <form
            onSubmit={handleLinkDocument}
            className="border-y border-border bg-surface-raised p-3"
            aria-label="Привязать документ к фазе"
          >
            <div className="grid gap-3 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-end">
              <div className="space-y-1.5">
                <Label htmlFor="phase-document-link">Документ для фазы</Label>
                <Input
                  id="phase-document-link"
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
          {phase.documents.length === 0 ? (
            <EmptyState message="С фазой пока не связан ни один документ" />
          ) : (
            <ul className="divide-y divide-border border-y border-border">
              {phase.documents.map((document) => (
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

        <section className="min-w-0 space-y-3" aria-labelledby="phase-evidence-title">
          <h2 id="phase-evidence-title" className="text-base font-semibold">
            Материалы
          </h2>
          {phase.evidence.length === 0 ? (
            <EmptyState message="Материалы пока не прикреплены" />
          ) : (
            <ul className="divide-y divide-border border-y border-border">
              {phase.evidence.map((item) => (
                <li key={item.id}>
                  <Link
                    to={evidencePhasePath(selectedSpaceKey, phase.phase_key, item.id)}
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
    </div>
  )
}
