import { ContentSection } from '@pdlc/ui/ui'
import { useProjectRouteRef } from '@/widgets/namespace-context'
import { FormEvent, useEffect, useMemo, useRef, useState } from 'react'
import { useLocation, useNavigate, useSearchParams } from 'react-router'
import { wikiSearchScope } from '@/shared/lib/search-scope'
import { SearchResultLink } from '@/widgets/search-result-link'
import { FileCheck2, FileText, GitBranch, ListFilter } from 'lucide-react'
import { useWikiSearch } from '@/shared/api/hooks'
import {
  ActionGroup,
  PageNavigation,
  PageHeader,
  SectionHeader,
  Select,
  Button,
  EmptyState,
  ErrorState,
  Input,
  LoadingState,
  ResourceList,
  SearchInput,
  FormField,
} from '@pdlc/ui/ui'
import { formatApiErrorForUser } from '@/shared/lib/api-error'
import { formatDateTime, formatDocumentType } from '@/shared/lib/wiki-format'
import type { SearchParams, SearchResult } from '@/api/wiki'

const resultTypes = [
  { label: 'Все', value: 'all' },
  { label: 'Документы', value: 'document' },
  { label: 'Материалы', value: 'evidence' },
] as const
const documentTypes = [
  'all',
  'page',
  'requirements',
  'research_note',
  'implementation_note',
  'test_plan',
  'release_note',
] as const
type ResultTypeFilter = (typeof resultTypes)[number]['value']
type DocumentTypeFilter = (typeof documentTypes)[number]
const pageSize = 20

function resultIcon(type: string) {
  if (type === 'evidence') return FileCheck2
  if (type === 'phase') return GitBranch
  return FileText
}

function resultLabel(type: string) {
  if (type === 'document') return 'документ'
  if (type === 'evidence') return 'материал'
  if (type === 'phase') return 'фаза'
  return type
}

function optional(value: string) {
  return value.trim() || undefined
}

function resultTypeFrom(value: string | null): ResultTypeFilter {
  return resultTypes.some((item) => item.value === value) ? (value as ResultTypeFilter) : 'all'
}

function documentTypeFrom(value: string | null): DocumentTypeFilter {
  return documentTypes.includes(value as DocumentTypeFilter) ? (value as DocumentTypeFilter) : 'all'
}

function setOptionalParam(params: URLSearchParams, key: string, value: string) {
  const normalized = value.trim()
  if (normalized) params.set(key, normalized)
  else params.delete(key)
}

export function WikiSearchPage() {
  const namespace = useProjectRouteRef()
  const [searchParams, setSearchParams] = useSearchParams()
  const location = useLocation()
  const navigate = useNavigate()
  const searchScope = wikiSearchScope(location.search, Boolean(namespace.ref))
  const projectSearch = searchScope.scope === 'project'
  const scopeError = searchScope.malformed || namespace.malformed
  const appliedQuery = searchParams.get('q')?.trim() ?? ''
  const appliedSpace = searchParams.get('space')?.trim().toUpperCase() ?? ''
  const appliedTask = searchParams.get('task_key')?.trim() ?? ''
  const appliedPhase = searchParams.get('phase_key')?.trim() ?? ''
  const resultTypeFilter = resultTypeFrom(searchParams.get('result_type'))
  const documentTypeFilter =
    resultTypeFilter === 'evidence' ? 'all' : documentTypeFrom(searchParams.get('document_type'))
  const [query, setQuery] = useState(appliedQuery)
  const [spaceFilter, setSpaceFilter] = useState(appliedSpace)
  const [taskFilter, setTaskFilter] = useState(appliedTask)
  const [phaseFilter, setPhaseFilter] = useState(appliedPhase)
  const [documentTypeDraft, setDocumentTypeDraft] = useState<DocumentTypeFilter>(documentTypeFilter)
  const [showFilters, setShowFilters] = useState(false)
  const criteriaKey = JSON.stringify([
    appliedQuery,
    appliedSpace,
    appliedTask,
    appliedPhase,
    resultTypeFilter,
    documentTypeFilter,
  ])
  const [paging, setPaging] = useState({
    criteriaKey,
    cursors: [undefined] as (string | undefined)[],
  })
  const pageCursors = useMemo<(string | undefined)[]>(
    () => (paging.criteriaKey === criteriaKey ? paging.cursors : [undefined]),
    [criteriaKey, paging.criteriaKey, paging.cursors],
  )
  const currentPage = pageCursors.length
  const resultsSection = useRef<HTMLElement>(null)

  useEffect(() => {
    setQuery(appliedQuery)
  }, [appliedQuery])

  useEffect(() => {
    setSpaceFilter(appliedSpace)
    setTaskFilter(appliedTask)
    setPhaseFilter(appliedPhase)
    setDocumentTypeDraft(documentTypeFilter)
  }, [appliedSpace, appliedTask, appliedPhase, documentTypeFilter])

  useEffect(() => {
    if (paging.criteriaKey !== criteriaKey) {
      setPaging({ criteriaKey, cursors: [undefined] })
    }
  }, [criteriaKey, paging.criteriaKey])

  useEffect(() => {
    const rawResultType = searchParams.get('result_type')
    const rawDocumentType = searchParams.get('document_type')
    const invalidResultType = rawResultType !== null && resultTypeFilter === 'all'
    const invalidDocumentType =
      rawDocumentType !== null && documentTypeFrom(rawDocumentType) === 'all'
    const incompatibleDocumentType = resultTypeFilter === 'evidence' && rawDocumentType !== null
    if (!invalidResultType && !invalidDocumentType && !incompatibleDocumentType) return

    setSearchParams(
      (current) => {
        const next = new URLSearchParams(current)
        if (invalidResultType) next.delete('result_type')
        if (invalidDocumentType || incompatibleDocumentType) next.delete('document_type')
        return next
      },
      { replace: true },
    )
  }, [resultTypeFilter, searchParams, setSearchParams])

  useEffect(() => {
    const normalizedQuery = query.trim()
    if (normalizedQuery === appliedQuery) return
    const timeout = window.setTimeout(() => {
      setSearchParams(
        (current) => {
          const next = new URLSearchParams(current)
          setOptionalParam(next, 'q', normalizedQuery)
          return next
        },
        { replace: true },
      )
    }, 300)
    return () => window.clearTimeout(timeout)
  }, [query, appliedQuery, setSearchParams])

  const requestParams: SearchParams = useMemo(
    () => ({
      q: appliedQuery,
      space: optional(appliedSpace),
      task_key: optional(appliedTask),
      phase_key: optional(appliedPhase),
      document_type: documentTypeFilter === 'all' ? undefined : documentTypeFilter,
      result_type: resultTypeFilter === 'all' ? undefined : resultTypeFilter,
      limit: pageSize,
      cursor: pageCursors[currentPage - 1],
    }),
    [
      appliedPhase,
      appliedQuery,
      appliedSpace,
      appliedTask,
      documentTypeFilter,
      resultTypeFilter,
      pageCursors,
      currentPage,
    ],
  )
  const searchQuery = useWikiSearch(requestParams, !scopeError)
  const results = searchQuery.data?.results ?? []
  const nextCursor = searchQuery.data?.next_cursor
  const activeFilterCount =
    [appliedSpace, appliedTask, appliedPhase].filter((value) => value.trim() !== '').length +
    (documentTypeFilter === 'all' ? 0 : 1)

  function submitSearch(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setSearchParams((current) => {
      const next = new URLSearchParams(current)
      setOptionalParam(next, 'q', query)
      return next
    })
  }

  function applyFilters(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setSearchParams((current) => {
      const next = new URLSearchParams(current)
      setOptionalParam(next, 'q', query)
      setOptionalParam(next, 'space', spaceFilter.toUpperCase())
      setOptionalParam(next, 'task_key', taskFilter)
      setOptionalParam(next, 'phase_key', phaseFilter)
      if (documentTypeDraft === 'all') next.delete('document_type')
      else next.set('document_type', documentTypeDraft)
      if (documentTypeDraft !== 'all') next.set('result_type', 'document')
      return next
    })
  }

  function selectResultType(value: ResultTypeFilter) {
    setSearchParams((current) => {
      const next = new URLSearchParams(current)
      if (value === 'all') next.delete('result_type')
      else next.set('result_type', value)
      if (value === 'evidence') next.delete('document_type')
      return next
    })
  }

  function changePage(direction: 'previous' | 'next') {
    if (direction === 'previous') {
      setPaging({ criteriaKey, cursors: pageCursors.slice(0, -1) })
    } else if (nextCursor) {
      setPaging({ criteriaKey, cursors: [...pageCursors, nextCursor] })
    }
    resultsSection.current?.scrollIntoView?.({ block: 'start' })
  }

  function changeScope(scope: string) {
    const next = new URLSearchParams(searchParams)
    setOptionalParam(next, 'q', query)
    next.set('search_scope', scope)
    next.delete('space')
    next.delete('cursor')
    setPaging({ criteriaKey: '', cursors: [undefined] })
    navigate({ pathname: '/search', search: '?' + next })
  }

  if (scopeError)
    return <ErrorState message="Некорректная область поиска. Проект не выбран автоматически." />

  return (
    <div className="space-y-4">
      <PageHeader title={'Поиск'} />
      <FormField id="search-scope" label={'Область поиска'}>
        {(baseFieldAttrs) => (
          <Select
            {...baseFieldAttrs}
            className="w-full"
            value={searchScope.scope}
            onValueChange={(nextValue) => changeScope(nextValue)}
            options={[
              { value: 'project', label: 'Текущий проект', disabled: !namespace.ref },
              { value: 'all', label: 'Все доступные пространства' },
            ]}
          />
        )}
      </FormField>

      <section aria-label="Параметры поиска" className="space-y-3">
        <form role="search" onSubmit={submitSearch} className="flex min-w-0 gap-2">
          <div className="min-w-0 flex-1">
            <SearchInput
              aria-label="Поисковый запрос"
              value={query}
              onChange={(event) => {
                setQuery(event.target.value)
              }}
              placeholder="Название, текст или ключ"
            />
          </div>
          <Button
            type="button"
            size="sm"
            variant="outline"
            className="shrink-0"
            aria-expanded={showFilters}
            aria-controls="search-filters"
            onClick={() => setShowFilters((value) => !value)}
          >
            <ListFilter className="h-4 w-4" aria-hidden />
            Фильтры{activeFilterCount > 0 ? ` (${activeFilterCount})` : ''}
          </Button>
        </form>

        {showFilters && (
          <ContentSection asChild dividers="both" density="compact">
            <form
              id="search-filters"
              onSubmit={applyFilters}
              className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4"
            >
              <FormField id="search-space" label={'Пространство'}>
                {(baseFieldAttrs) => (
                  <Input
                    {...baseFieldAttrs}
                    readOnly={projectSearch}

                    value={spaceFilter}
                    onChange={(event) => {
                      setSpaceFilter(event.target.value.toUpperCase())
                    }}
                    placeholder="BASE"
                  />
                )}
              </FormField>
              <FormField id="search-task" label={'Задача'}>
                {(baseFieldAttrs) => (
                  <Input
                    {...baseFieldAttrs}
                    value={taskFilter}
                    onChange={(event) => {
                      setTaskFilter(event.target.value)
                    }}
                    placeholder="BASE-42"
                  />
                )}
              </FormField>
              <FormField id="search-phase" label={'Фаза'}>
                {(baseFieldAttrs) => (
                  <Input
                    {...baseFieldAttrs}
                    value={phaseFilter}
                    onChange={(event) => {
                      setPhaseFilter(event.target.value)
                    }}
                    placeholder="implementation"
                  />
                )}
              </FormField>
              <FormField id="search-document-type" label={'Тип документа'}>
                {(baseFieldAttrs) => (
                  <Select
                    {...baseFieldAttrs}
                    className="w-full"
                    value={documentTypeDraft}
                    onValueChange={(nextValue) => {
                      setDocumentTypeDraft(nextValue as DocumentTypeFilter)
                    }}
                    options={[
                      ...(documentTypes.map((type) => ({
                        value: String(type),
                        label: String(type === 'all' ? 'Любой тип' : formatDocumentType(type)),
                      })) ?? []),
                    ]}
                  />
                )}
              </FormField>
              <ActionGroup align="end" className="sm:col-span-2 xl:col-span-4">
                {(activeFilterCount > 0 ||
                  spaceFilter ||
                  taskFilter ||
                  phaseFilter ||
                  documentTypeDraft !== 'all') && (
                  <Button
                    type="button"
                    size="sm"
                    variant="ghost"

                    onClick={() => {
                      setSpaceFilter('')
                      setTaskFilter('')
                      setPhaseFilter('')
                      setDocumentTypeDraft('all')
                      setSearchParams((current) => {
                        const next = new URLSearchParams(current)
                        for (const key of [
                          'space',
                          'task_key',
                          'phase_key',
                          'document_type',
                          'result_type',
                        ]) {
                          next.delete(key)
                        }
                        return next
                      })
                    }}
                  >
                    Сбросить фильтры
                  </Button>
                )}
                <Button type="submit" size="sm">
                  Применить
                </Button>
              </ActionGroup>
            </form>
          </ContentSection>
        )}
      </section>

      <section
        ref={resultsSection}
        aria-labelledby="search-results-title"
        className="scroll-mt-16 space-y-3"
      >
        <SectionHeader
          headingId="search-results-title"
          title={<>Результаты</>}
          context={
            <>
              {!searchQuery.isLoading && !searchQuery.isError && (
                <p role="status" className="text-sm text-text-muted">
                  {results.length === 0
                    ? '0 результатов'
                    : `Показано ${(currentPage - 1) * pageSize + 1}–${(currentPage - 1) * pageSize + results.length}`}
                </p>
              )}
            </>
          }
        />
        <div role="group" aria-label="Тип результата" className="flex flex-wrap gap-2">
          {resultTypes.map((item) => (
            <Button
              key={item.value}
              type="button"
              size="sm"
              variant={resultTypeFilter === item.value ? 'secondary' : 'outline'}

              aria-pressed={resultTypeFilter === item.value}
              onClick={() => selectResultType(item.value)}
            >
              {item.label}
            </Button>
          ))}
        </div>

        {searchQuery.isLoading && <LoadingState message="Ищем" />}
        {searchQuery.isError && (
          <ErrorState
            message={formatApiErrorForUser(searchQuery.error, 'Не удалось выполнить поиск')}
            onRetry={() => searchQuery.refetch()}
          />
        )}
        {!searchQuery.isLoading && !searchQuery.isError && results.length === 0 && (
          <EmptyState message="Ничего не найдено" />
        )}
        {!searchQuery.isLoading && !searchQuery.isError && results.length > 0 && (
          <ResourceList asChild>
            <div>
              {results.map((result: SearchResult) => {
                const Icon = resultIcon(result.result_type)
                return (
                  <SearchResultLink
                    key={`${result.result_type}-${result.id}`}
                    result={result}
                    allSpaces={!projectSearch}
                  >
                    <Icon className="mt-0.5 h-4 w-4 shrink-0 text-accent" aria-hidden />
                    <span className="min-w-0 flex-1">
                      <span className="block break-words text-sm font-medium text-text-primary">
                        {result.title}
                      </span>
                      <span className="mt-0.5 block text-xs text-text-muted">
                        {resultLabel(result.result_type)} · {result.space_key} ·{' '}
                        {formatDateTime(result.updated_at)}
                      </span>
                      {result.snippet && (
                        <span className="mt-1 block break-words text-sm text-text-secondary line-clamp-1">
                          {result.snippet}
                        </span>
                      )}
                    </span>
                  </SearchResultLink>
                )
              })}
            </div>
          </ResourceList>
        )}
        {(currentPage > 1 || nextCursor) && !searchQuery.isLoading && !searchQuery.isError && (
          <PageNavigation
            page={currentPage}
            hasNext={!!nextCursor}
            onPageChange={(nextPage) => changePage(nextPage < currentPage ? 'previous' : 'next')}
            label="Страницы результатов"
            previousLabel="Назад"
            nextLabel="Далее"
            summary={<>Страница {currentPage}</>}
          />
        )}
      </section>
    </div>
  )
}
