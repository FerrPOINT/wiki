import { SpaceSelector, useSelectedSpaceKey } from '@/widgets/dossier-space'
import { type FormEvent, useRef, useState } from 'react'
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
  SearchInput,
  PageNavigation,
  PageHeader,
  Notice,
  TextLink,
  FormField,
} from '@pdlc/ui/ui'
import { useParams } from 'react-router'
import { FileCheck2, FileText, Link2 } from 'lucide-react'
import { useLinkPhaseDocument, usePhase, usePhaseSummaries } from '@/shared/api/hooks'

import { formatApiErrorForUser } from '@/shared/lib/api-error'
import { pathWithSearchParams, useDossierCatalogUrl } from '@/shared/lib/dossier-catalog-url'
import {
  formatDateTime,
  formatDocumentStatus,
  formatDocumentType,
  formatEvidenceType,
} from '@/shared/lib/wiki-format'

const pageSize = 12

function evidencePhasePath(spaceKey: string, phaseKey: string, evidenceId: string): string {
  const params = new URLSearchParams({ space: spaceKey, phase_key: phaseKey, id: evidenceId })
  return `/evidence?${params.toString()}`
}

function PhaseCatalog({ spaceKey }: { spaceKey: string }) {
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
  const phasesQuery = usePhaseSummaries(spaceKey, {
    limit: pageSize,
    cursor,
    q: appliedSearch || undefined,
  })
  const phases = phasesQuery.data?.phases ?? []
  const nextCursor = phasesQuery.data?.next_cursor

  function navigatePage(direction: 'previous' | 'next') {
    changePage(direction, direction === 'next' ? (nextCursor ?? undefined) : undefined)
    listRef.current?.scrollIntoView?.({ block: 'start' })
  }

  return (
    <section ref={listRef} aria-label="Список фаз" className="scroll-mt-16 space-y-3">
      <div className="max-w-md">
        <SearchInput
          aria-label="Найти фазу"
          placeholder="Ключ, название или документ"
          value={search}
          onChange={(event) => setSearch(event.target.value)}
        />
      </div>

      {phasesQuery.isLoading && <LoadingState message="Загружаем фазы" />}
      {phasesQuery.isError && (
        <ErrorState
          message={formatApiErrorForUser(phasesQuery.error, 'Не удалось загрузить фазы')}
          onRetry={() => phasesQuery.refetch()}
        />
      )}
      {!phasesQuery.isLoading && !phasesQuery.isError && phases.length === 0 && (
        <EmptyState
          message={
            cursor
              ? 'На этой странице фаз больше нет'
              : appliedSearch
                ? 'По запросу фазы не найдены'
                : 'Документы и материалы ещё не связаны с фазами'
          }
        />
      )}
      {!phasesQuery.isLoading && !phasesQuery.isError && phases.length > 0 && (
        <>
          <p role="status" className="text-xs text-text-muted">
            Фазы: {phases.length} из {phasesQuery.data?.total}
          </p>
          <ResourceList asChild>
            <ul>
              {phases.map((phase) => (
                <li key={phase.phase_key}>
                  <ResourceListRow asChild interactive>
                    <Link
                      to={pathWithSearchParams(`/phases/${phase.phase_key}`, searchParams)}
                      className="flex min-w-0 flex-wrap items-center justify-between gap-x-4 gap-y-1"
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
                  </ResourceListRow>
                </li>
              ))}
            </ul>
          </ResourceList>
        </>
      )}
      {(hasPreviousPage || nextCursor) && (
        <PageNavigation
          label="Страницы фаз"
          page={currentPage}
          hasNext={Boolean(nextCursor) && !phasesQuery.isError}
          pending={phasesQuery.isLoading || phasesQuery.isFetching}
          previousLabel="Назад"
          nextLabel="Далее"
          summary={`Страница ${currentPage}`}
          onPageChange={(page) => navigatePage(page < currentPage ? 'previous' : 'next')}
        />
      )}
    </section>
  )
}

export function PhaseDossiersPage() {
  const [selectedSpaceKey, setSelectedSpaceKey, spacesQuery] = useSelectedSpaceKey()

  return (
    <div className="space-y-5">
      <PageHeader
        title="Фазы процесса"
        context="Каждая завершённая фаза должна иметь документы и материалы, достаточные для аудита."
        actions={
          <SpaceSelector
            id="phase-space-selector"
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
      {selectedSpaceKey && <PhaseCatalog key={selectedSpaceKey} spaceKey={selectedSpaceKey} />}
    </div>
  )
}

export function PhaseDossierPage() {
  const { phaseId = 'implementation' } = useParams()
  const [selectedSpaceKey, setSelectedSpaceKey, spacesQuery, searchParams] = useSelectedSpaceKey()
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
        <div className="space-y-2">
          <TextLink asChild>
            <Link
              to={pathWithSearchParams('/phases', searchParams)}
              className="inline-flex min-h-10 items-center text-sm"
            >
              К фазам
            </Link>
          </TextLink>
          <PageHeader
            title={phase.title ?? phase.phase_key}
            context={
              phase.title &&
              phase.title !== phase.phase_key && (
                <p className="mt-1 break-words text-sm text-text-muted">{phase.phase_key}</p>
              )
            }
            actions={
              <>
                <SpaceSelector
                  id="phase-space-selector"
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

      <div
        className="grid min-w-0 items-start gap-6 xl:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]"
        data-dossier-layout="parallel-content"
      >
        <section className="min-w-0 space-y-3" aria-labelledby="phase-documents-title">
          <SectionHeader headingId="phase-documents-title" title={<>Документы фазы</>} />
          <form
            onSubmit={handleLinkDocument}
            className="border-y border-border bg-surface-raised p-3"
            aria-label="Привязать документ к фазе"
          >
            <div className="grid gap-3 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-end">
              <FormField id="phase-document-link" label={'Документ для фазы'}>
                {(baseFieldAttrs) => (
                  <Input
                    {...baseFieldAttrs}
                    value={documentId}
                    onChange={(event) => setDocumentId(event.target.value)}
                    placeholder="product-requirements"
                    disabled={linkDocument.isPending}
                  />
                )}
              </FormField>
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
          {phase.documents.length === 0 ? (
            <EmptyState message="С фазой пока не связан ни один документ" />
          ) : (
            <ResourceList asChild>
              <ul>
                {phase.documents.map((document) => (
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

        <section className="min-w-0 space-y-3" aria-labelledby="phase-evidence-title">
          <SectionHeader headingId="phase-evidence-title" title={<>Материалы</>} />
          {phase.evidence.length === 0 ? (
            <EmptyState message="Материалы пока не прикреплены" />
          ) : (
            <ResourceList asChild>
              <ul>
                {phase.evidence.map((item) => (
                  <li key={item.id}>
                    <ResourceListRow asChild interactive>
                      <Link
                        to={evidencePhasePath(selectedSpaceKey, phase.phase_key, item.id)}
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
    </div>
  )
}
