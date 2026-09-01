import { type FormEvent, useState } from 'react'
import { Link, useParams, useSearchParams } from 'react-router'
import { CheckCircle2, CircleDashed, FileCheck2, FileText, GitBranch, Link2 } from 'lucide-react'
import {
  defaultSpaceKey,
  useLinkPhaseDocument,
  usePhase,
  usePhases,
  useSpaces,
} from '@/shared/api/hooks'
import { EmptyState, ErrorState, LoadingState } from '@/shared/ui/async-states'
import { Button } from '@/shared/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'
import { Input } from '@/shared/ui/input'
import { Label } from '@/shared/ui/label'
import { Progress } from '@/shared/ui/progress'
import { formatApiErrorForUser } from '@/shared/lib/api-error'
import {
  formatDateTime,
  formatDocumentStatus,
  formatDocumentType,
  formatEvidenceType,
} from '@/shared/lib/wiki-format'
import type { PhasePage } from '@/api/wiki'

const selectClassName =
  'flex h-9 w-full rounded-md border border-border-strong bg-surface px-3 py-1 text-sm text-text-primary shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent disabled:cursor-not-allowed disabled:opacity-50'

type SpaceOption = {
  key: string
  name: string
}

function readiness(phase: Pick<PhasePage, 'document_count' | 'evidence_count'>): number {
  const score = phase.document_count * 45 + phase.evidence_count * 35
  return Math.max(0, Math.min(100, score))
}

function normalizeSpaceParam(value: string | null): string {
  const normalized = value?.trim().toUpperCase()
  return normalized || defaultSpaceKey
}

function useSelectedSpaceKey() {
  const [searchParams, setSearchParams] = useSearchParams()
  const selectedSpaceKey = normalizeSpaceParam(searchParams.get('space'))

  function setSelectedSpaceKey(spaceKey: string) {
    const normalized = normalizeSpaceParam(spaceKey)
    const nextParams = new URLSearchParams(searchParams)
    if (normalized === defaultSpaceKey) nextParams.delete('space')
    else nextParams.set('space', normalized)
    setSearchParams(nextParams, { replace: true })
  }

  return [selectedSpaceKey, setSelectedSpaceKey] as const
}

function scopedPath(path: string, spaceKey: string): string {
  return spaceKey === defaultSpaceKey ? path : `${path}?space=${encodeURIComponent(spaceKey)}`
}

function evidencePhasePath(spaceKey: string, phaseKey: string): string {
  const params = new URLSearchParams({ space: spaceKey, phase_key: phaseKey })
  return `/evidence?${params.toString()}`
}

function SpaceSelector({
  value,
  onChange,
}: {
  value: string
  onChange: (spaceKey: string) => void
}) {
  const spacesQuery = useSpaces()
  const spaces: SpaceOption[] = spacesQuery.data?.spaces ?? []
  const hasSelected = spaces.some((space) => space.key === value)
  const options = hasSelected ? spaces : [{ key: value, name: value }, ...spaces]

  return (
    <div className="w-full space-y-1.5 sm:w-64">
      <Label htmlFor="phase-space-selector">Пространство</Label>
      <select
        id="phase-space-selector"
        className={selectClassName}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        disabled={spacesQuery.isLoading}
      >
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
  const [selectedSpaceKey, setSelectedSpaceKey] = useSelectedSpaceKey()
  const phasesQuery = usePhases(selectedSpaceKey)
  const phases = phasesQuery.data?.phases ?? []

  return (
    <div className="space-y-5">
      <section className="flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold">Фазы процесса</h1>
          <p className="mt-1 max-w-3xl text-sm text-text-muted">
            Каждая завершённая фаза должна иметь документы и материалы, достаточные для аудита.
          </p>
        </div>
        <SpaceSelector value={selectedSpaceKey} onChange={setSelectedSpaceKey} />
      </section>

      {phasesQuery.isLoading && <LoadingState message="Загружаем фазы" />}
      {phasesQuery.isError && (
        <ErrorState
          message={formatApiErrorForUser(phasesQuery.error, 'Не удалось загрузить фазы')}
          onRetry={() => phasesQuery.refetch()}
        />
      )}
      {!phasesQuery.isLoading && !phasesQuery.isError && phases.length === 0 && (
        <EmptyState message="Документы и материалы ещё не связаны с фазами" />
      )}
      {!phasesQuery.isLoading && !phasesQuery.isError && phases.length > 0 && (
        <section className="grid gap-4 lg:grid-cols-4">
          {phases.map((phase) => {
            const value = readiness(phase)
            return (
              <Card key={phase.phase_key}>
                <CardHeader>
                  <CardTitle className="flex items-center justify-between gap-2 text-base">
                    {phase.title ?? phase.phase_key}
                    {phase.evidence_count > 0 ? (
                      <CheckCircle2 className="h-4 w-4 text-success" />
                    ) : (
                      <CircleDashed className="h-4 w-4 text-warning" />
                    )}
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="space-y-1.5">
                    <div className="flex justify-between text-xs text-text-muted">
                      <span>Заполненность</span>
                      <span>{value}%</span>
                    </div>
                    <Progress value={value} variant={value < 50 ? 'danger' : 'default'} />
                  </div>
                  <div className="text-xs text-text-muted">
                    Документы: {phase.document_count} · Материалы: {phase.evidence_count}
                  </div>
                  <Link
                    to={scopedPath(`/phases/${phase.phase_key}`, selectedSpaceKey)}
                    className="text-sm text-accent"
                  >
                    Открыть фазу
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

export function PhaseDossierPage() {
  const { phaseId = 'implementation' } = useParams()
  const [selectedSpaceKey, setSelectedSpaceKey] = useSelectedSpaceKey()
  const phaseQuery = usePhase(phaseId, selectedSpaceKey)
  const linkDocument = useLinkPhaseDocument()
  const [documentId, setDocumentId] = useState('')
  const [linkMessage, setLinkMessage] = useState('')
  const phase = phaseQuery.data
  const value = phase ? readiness(phase) : 0

  function handleLinkDocument(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const trimmedDocumentId = documentId.trim()
    if (!trimmedDocumentId) return
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
      <section className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
        <div>
          <div className="text-sm text-text-muted">карточка фазы</div>
          <h1 className="mt-2 text-2xl font-bold">{phase.title ?? phase.phase_key}</h1>
          <p className="mt-1 max-w-3xl text-sm text-text-muted">
            Документы и материалы, привязанные к фазе процесса в пространстве {selectedSpaceKey}.
          </p>
        </div>
        <SpaceSelector value={selectedSpaceKey} onChange={setSelectedSpaceKey} />
        <div className="min-w-72 rounded-md border border-border p-3">
          <div className="flex items-center justify-between text-sm">
            <span className="font-medium">Заполненность</span>
            <span className="text-text-muted">{value}%</span>
          </div>
          <Progress value={value} className="mt-3" />
          <p className="mt-2 text-xs text-text-muted">
            Документы: {phase.document_count} · Материалы: {phase.evidence_count}
          </p>
        </div>
      </section>

      <section className="grid gap-4 xl:grid-cols-[1fr_1fr]">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base">
              <FileText className="h-4 w-4 text-accent" />
              Документы фазы
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <form
              onSubmit={handleLinkDocument}
              className="rounded-md bg-surface-raised p-3"
              aria-label="Привязать документ к фазе"
            >
              <div className="grid gap-3 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-end">
                <div className="space-y-1.5">
                  <Label htmlFor="phase-document-link">Документ для фазы</Label>
                  <Input
                    id="phase-document-link"
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
            {phase.documents.length === 0 && (
              <EmptyState message="С фазой пока не связан ни один документ" />
            )}
            {phase.documents.map((document) => (
              <Link
                key={document.id}
                to={`/documents/${document.slug}`}
                className="flex items-center justify-between rounded-md border border-border p-3 text-sm hover:bg-surface-raised"
              >
                <span>
                  {document.title}
                  <span className="ml-2 text-xs text-text-muted">
                    {formatDocumentType(document.document_type)}
                  </span>
                </span>
                <span className="rounded bg-surface-raised px-2 py-1 text-xs text-text-secondary">
                  {formatDocumentStatus(document.status)}
                </span>
              </Link>
            ))}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base">
              <FileCheck2 className="h-4 w-4 text-accent" />
              Материалы
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-3 text-sm text-text-secondary">
            {phase.evidence.length === 0 && <EmptyState message="Материалы пока не прикреплены" />}
            {phase.evidence.map((item) => (
              <Link
                key={item.id}
                to={evidencePhasePath(selectedSpaceKey, phase.phase_key)}
                className="flex items-center justify-between rounded-md border border-border p-3 hover:bg-surface-raised"
              >
                <span className="inline-flex items-center gap-2">
                  <GitBranch className="h-4 w-4 text-accent" />
                  {item.title}
                </span>
                <span className="rounded bg-surface-raised px-2 py-1 text-xs text-text-muted">
                  {formatEvidenceType(item.evidence_type)} · {formatDateTime(item.created_at)}
                </span>
              </Link>
            ))}
          </CardContent>
        </Card>
      </section>
    </div>
  )
}
