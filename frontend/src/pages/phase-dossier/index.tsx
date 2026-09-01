import { Link, useParams } from 'react-router'
import { CheckCircle2, CircleDashed, FileCheck2, FileText, GitBranch } from 'lucide-react'
import { defaultSpaceKey, usePhase, usePhases } from '@/shared/api/hooks'
import { EmptyState, ErrorState, LoadingState } from '@/shared/ui/async-states'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'
import { Progress } from '@/shared/ui/progress'
import { formatApiErrorForUser } from '@/shared/lib/api-error'
import {
  formatDateTime,
  formatDocumentStatus,
  formatDocumentType,
  formatEvidenceType,
} from '@/shared/lib/wiki-format'
import type { PhasePage } from '@/api/wiki'

function readiness(phase: Pick<PhasePage, 'document_count' | 'evidence_count'>): number {
  const score = phase.document_count * 45 + phase.evidence_count * 35
  return Math.max(0, Math.min(100, score))
}

export function PhaseDossiersPage() {
  const phasesQuery = usePhases(defaultSpaceKey)
  const phases = phasesQuery.data?.phases ?? []

  return (
    <div className="space-y-5">
      <section>
        <h1 className="text-2xl font-bold">Фазы процесса</h1>
        <p className="mt-1 max-w-3xl text-sm text-text-muted">
          Каждая завершённая фаза должна иметь документы и материалы, достаточные для аудита.
        </p>
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
                  <Link to={`/phases/${phase.phase_key}`} className="text-sm text-accent">
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
  const phaseQuery = usePhase(phaseId, defaultSpaceKey)
  const phase = phaseQuery.data
  const value = phase ? readiness(phase) : 0

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
            Документы и материалы, привязанные к фазе процесса.
          </p>
        </div>
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
                to="/evidence"
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
