import { FormEvent, useEffect, useMemo, useRef, useState } from 'react'
import { Link } from 'react-router'
import { FileCheck2, FileText, GitBranch, ListFilter, Search } from 'lucide-react'
import { useWikiSearch } from '@/shared/api/hooks'
import { Button, EmptyState, ErrorState, Input, Label, LoadingState } from '@sdlc/ui/ui'
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
const pageSize = 20
const selectClassName =
  'min-h-10 w-full rounded-md border border-border-strong bg-surface px-3 text-sm text-text-primary focus-visible:outline-2 focus-visible:outline-accent'

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

export function WikiSearchPage() {
  const [query, setQuery] = useState('')
  const [appliedQuery, setAppliedQuery] = useState('')
  const [spaceFilter, setSpaceFilter] = useState('')
  const [taskFilter, setTaskFilter] = useState('')
  const [phaseFilter, setPhaseFilter] = useState('')
  const [appliedFilters, setAppliedFilters] = useState({
    space: '',
    task: '',
    phase: '',
    documentType: 'all',
  })
  const [resultTypeFilter, setResultTypeFilter] =
    useState<(typeof resultTypes)[number]['value']>('all')
  const [documentTypeFilter, setDocumentTypeFilter] = useState<string>('all')
  const [showFilters, setShowFilters] = useState(false)
  const [pageCursors, setPageCursors] = useState<(string | undefined)[]>([undefined])
  const currentPage = pageCursors.length
  const resultsSection = useRef<HTMLElement>(null)

  useEffect(() => {
    const timeout = window.setTimeout(() => {
      if (query.trim() !== appliedQuery) {
        setAppliedQuery(query.trim())
        setPageCursors([undefined])
      }
    }, 300)
    return () => window.clearTimeout(timeout)
  }, [query, appliedQuery])

  const searchParams: SearchParams = useMemo(
    () => ({
      q: appliedQuery,
      space: optional(appliedFilters.space),
      task_key: optional(appliedFilters.task),
      phase_key: optional(appliedFilters.phase),
      document_type:
        appliedFilters.documentType === 'all' ? undefined : appliedFilters.documentType,
      result_type: resultTypeFilter === 'all' ? undefined : resultTypeFilter,
      limit: pageSize,
      cursor: pageCursors[currentPage - 1],
    }),
    [appliedFilters, appliedQuery, resultTypeFilter, pageCursors, currentPage],
  )
  const searchQuery = useWikiSearch(searchParams)
  const results = searchQuery.data?.results ?? []
  const nextCursor = searchQuery.data?.next_cursor
  const activeFilterCount =
    [appliedFilters.space, appliedFilters.task, appliedFilters.phase].filter(
      (value) => value.trim() !== '',
    ).length + (appliedFilters.documentType === 'all' ? 0 : 1)

  function submitSearch(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setAppliedQuery(query.trim())
    setPageCursors([undefined])
  }

  function applyFilters(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setAppliedFilters({
      space: spaceFilter.trim(),
      task: taskFilter.trim(),
      phase: phaseFilter.trim(),
      documentType: documentTypeFilter,
    })
    if (documentTypeFilter !== 'all') setResultTypeFilter('document')
    setPageCursors([undefined])
  }

  function selectResultType(value: (typeof resultTypes)[number]['value']) {
    setResultTypeFilter(value)
    if (value === 'evidence') {
      setDocumentTypeFilter('all')
      setAppliedFilters((filters) => ({ ...filters, documentType: 'all' }))
    }
    setPageCursors([undefined])
  }

  function changePage(direction: 'previous' | 'next') {
    if (direction === 'previous') setPageCursors((cursors) => cursors.slice(0, -1))
    else if (nextCursor) setPageCursors((cursors) => [...cursors, nextCursor])
    resultsSection.current?.scrollIntoView?.({ block: 'start' })
  }

  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-bold">Поиск</h1>

      <section aria-label="Параметры поиска" className="space-y-3">
        <form role="search" onSubmit={submitSearch} className="flex min-w-0 gap-2">
          <div className="relative min-w-0 flex-1">
            <Search
              className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-text-muted"
              aria-hidden
            />
            <Input
              type="search"
              aria-label="Поисковый запрос"
              className="min-h-10 pl-9"
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
            className="min-h-10 shrink-0 sm:min-h-10"
            aria-expanded={showFilters}
            aria-controls="search-filters"
            onClick={() => setShowFilters((value) => !value)}
          >
            <ListFilter className="h-4 w-4" aria-hidden />
            Фильтры{activeFilterCount > 0 ? ` (${activeFilterCount})` : ''}
          </Button>
        </form>

        {showFilters && (
          <form
            id="search-filters"
            onSubmit={applyFilters}
            className="grid gap-3 border-y border-border py-3 sm:grid-cols-2 xl:grid-cols-4"
          >
            <div className="space-y-1.5">
              <Label htmlFor="search-space">Пространство</Label>
              <Input
                id="search-space"
                className="min-h-10"
                value={spaceFilter}
                onChange={(event) => {
                  setSpaceFilter(event.target.value.toUpperCase())
                }}
                placeholder="BASE"
              />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="search-task">Задача</Label>
              <Input
                id="search-task"
                className="min-h-10"
                value={taskFilter}
                onChange={(event) => {
                  setTaskFilter(event.target.value)
                }}
                placeholder="BASE-42"
              />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="search-phase">Фаза</Label>
              <Input
                id="search-phase"
                className="min-h-10"
                value={phaseFilter}
                onChange={(event) => {
                  setPhaseFilter(event.target.value)
                }}
                placeholder="implementation"
              />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="search-document-type">Тип документа</Label>
              <select
                id="search-document-type"
                className={selectClassName}
                value={documentTypeFilter}
                onChange={(event) => {
                  setDocumentTypeFilter(event.target.value)
                }}
              >
                {documentTypes.map((type) => (
                  <option key={type} value={type}>
                    {type === 'all' ? 'Любой тип' : formatDocumentType(type)}
                  </option>
                ))}
              </select>
            </div>
            <div className="flex flex-wrap justify-end gap-2 sm:col-span-2 xl:col-span-4">
              {(activeFilterCount > 0 ||
                spaceFilter ||
                taskFilter ||
                phaseFilter ||
                documentTypeFilter !== 'all') && (
                <Button
                  type="button"
                  size="sm"
                  variant="ghost"
                  className="min-h-10 sm:min-h-10"
                  onClick={() => {
                    setSpaceFilter('')
                    setTaskFilter('')
                    setPhaseFilter('')
                    setDocumentTypeFilter('all')
                    setAppliedFilters({ space: '', task: '', phase: '', documentType: 'all' })
                    setResultTypeFilter('all')
                    setPageCursors([undefined])
                  }}
                >
                  Сбросить фильтры
                </Button>
              )}
              <Button type="submit" size="sm" className="min-h-10 sm:min-h-10">
                Применить
              </Button>
            </div>
          </form>
        )}
      </section>

      <section
        ref={resultsSection}
        aria-labelledby="search-results-title"
        className="scroll-mt-16 space-y-3"
      >
        <div className="flex flex-wrap items-center justify-between gap-2">
          <h2 id="search-results-title" className="text-base font-semibold">
            Результаты
          </h2>
          {!searchQuery.isLoading && !searchQuery.isError && (
            <p role="status" className="text-sm text-text-muted">
              {results.length === 0
                ? '0 результатов'
                : `Показано ${(currentPage - 1) * pageSize + 1}–${(currentPage - 1) * pageSize + results.length}`}
            </p>
          )}
        </div>
        <div role="group" aria-label="Тип результата" className="flex flex-wrap gap-2">
          {resultTypes.map((item) => (
            <Button
              key={item.value}
              type="button"
              size="sm"
              variant={resultTypeFilter === item.value ? 'secondary' : 'outline'}
              className="min-h-10 sm:min-h-10"
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
          <div className="divide-y divide-border border-y border-border">
            {results.map((result: SearchResult) => {
              const Icon = resultIcon(result.result_type)
              return (
                <Link
                  key={`${result.result_type}-${result.id}`}
                  to={result.url}
                  className="flex min-h-16 min-w-0 items-start gap-3 px-2 py-3 hover:bg-surface-raised focus-visible:outline-2 focus-visible:outline-accent"
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
                </Link>
              )
            })}
          </div>
        )}
        {(currentPage > 1 || nextCursor) && !searchQuery.isLoading && !searchQuery.isError && (
          <nav aria-label="Страницы результатов" className="flex items-center justify-end gap-2">
            <Button
              type="button"
              size="sm"
              variant="outline"
              className="min-h-10 sm:min-h-10"
              disabled={currentPage === 1}
              onClick={() => changePage('previous')}
            >
              Назад
            </Button>
            <span className="text-sm text-text-muted">Страница {currentPage}</span>
            <Button
              type="button"
              size="sm"
              variant="outline"
              className="min-h-10 sm:min-h-10"
              disabled={!nextCursor}
              onClick={() => changePage('next')}
            >
              Далее
            </Button>
          </nav>
        )}
      </section>
    </div>
  )
}
