import { useMemo, useState } from 'react'
import { Link } from 'react-router'
import { FileCheck2, FileText, GitBranch, Search } from 'lucide-react'
import { defaultSpaceKey, useWikiSearch } from '@/shared/api/hooks'
import { EmptyState, ErrorState, LoadingState } from '@sdlc/ui/ui'
import { Card, CardContent, CardHeader, CardTitle } from '@sdlc/ui/ui'
import { Input } from '@sdlc/ui/ui'
import { Label } from '@sdlc/ui/ui'
import { formatApiErrorForUser } from '@/shared/lib/api-error'
import { formatDateTime } from '@/shared/lib/wiki-format'
import type { SearchParams, SearchResult } from '@/api/wiki'

const resultTypeFilters = [
  { label: 'Все', value: 'all' },
  { label: 'Документы', value: 'document' },
  { label: 'Материалы', value: 'evidence' },
]

const documentTypeFilters = [
  { label: 'Любой тип', value: 'all' },
  { label: 'Требования', value: 'requirements' },
  { label: 'Исследование', value: 'research_note' },
  { label: 'Реализация', value: 'implementation_note' },
  { label: 'План проверки', value: 'test_plan' },
  { label: 'Релиз', value: 'release_note' },
]

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

function countResults(results: SearchResult[], type: string) {
  return results.filter((result) => result.result_type === type).length
}

function optional(value: string) {
  const trimmed = value.trim()
  return trimmed === '' ? undefined : trimmed
}

export function WikiSearchPage() {
  const [query, setQuery] = useState('')
  const [spaceFilter, setSpaceFilter] = useState(defaultSpaceKey)
  const [taskFilter, setTaskFilter] = useState('')
  const [phaseFilter, setPhaseFilter] = useState('')
  const [resultTypeFilter, setResultTypeFilter] = useState('all')
  const [documentTypeFilter, setDocumentTypeFilter] = useState('all')
  const searchParams: SearchParams = useMemo(
    () => ({
      q: query,
      space: optional(spaceFilter) ?? defaultSpaceKey,
      task_key: optional(taskFilter),
      phase_key: optional(phaseFilter),
      document_type: documentTypeFilter === 'all' ? undefined : documentTypeFilter,
      limit: 25,
    }),
    [documentTypeFilter, phaseFilter, query, spaceFilter, taskFilter],
  )
  const searchQuery = useWikiSearch(searchParams)
  const results = useMemo(() => searchQuery.data?.results ?? [], [searchQuery.data?.results])
  const filteredResults = useMemo(() => {
    if (resultTypeFilter === 'all') return results
    return results.filter((result) => result.result_type === resultTypeFilter)
  }, [resultTypeFilter, results])

  return (
    <div className="space-y-5">
      <section>
        <h1 className="text-2xl font-bold">Поиск</h1>
        <p className="mt-1 max-w-3xl text-sm text-text-muted">
          Поиск по документам, задачам, фазам и материалам.
        </p>
      </section>

      <section className="space-y-3 rounded-md border border-border bg-surface p-3">
        <div className="relative">
          <Search className="pointer-events-none absolute left-3 top-2.5 h-4 w-4 text-text-muted" />
          <Input
            aria-label="Поисковый запрос"
            className="pl-9"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="релиз, SDLC-42, требования..."
          />
        </div>
        <div className="grid gap-3 md:grid-cols-3">
          <div className="space-y-1.5">
            <Label htmlFor="search-space">Пространство поиска</Label>
            <Input
              id="search-space"
              value={spaceFilter}
              onChange={(event) => setSpaceFilter(event.target.value.toUpperCase())}
              placeholder="SDLC"
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="search-task">Задача</Label>
            <Input
              id="search-task"
              value={taskFilter}
              onChange={(event) => setTaskFilter(event.target.value)}
              placeholder="SDLC-42"
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="search-phase">Фаза</Label>
            <Input
              id="search-phase"
              value={phaseFilter}
              onChange={(event) => setPhaseFilter(event.target.value)}
              placeholder="implementation"
            />
          </div>
        </div>
        <div className="space-y-2">
          <div className="flex flex-wrap gap-2">
            {resultTypeFilters.map((item) => (
              <button
                key={item.value}
                type="button"
                className={`min-h-8 rounded-md border px-3 py-1.5 text-xs ${
                  resultTypeFilter === item.value
                    ? 'border-accent bg-accent text-white'
                    : 'border-border text-text-secondary hover:bg-surface-raised'
                }`}
                onClick={() => setResultTypeFilter(item.value)}
              >
                {item.label}
              </button>
            ))}
          </div>
          <div className="flex flex-wrap gap-2">
            {documentTypeFilters.map((item) => (
              <button
                key={item.value}
                type="button"
                className={`min-h-8 rounded-md border px-3 py-1.5 text-xs ${
                  documentTypeFilter === item.value
                    ? 'border-accent bg-accent text-white'
                    : 'border-border text-text-secondary hover:bg-surface-raised'
                }`}
                onClick={() => setDocumentTypeFilter(item.value)}
              >
                {item.label}
              </button>
            ))}
          </div>
        </div>
      </section>

      <section className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_18rem]">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Результаты</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            {searchQuery.isLoading && <LoadingState message="Ищем" />}
            {searchQuery.isError && (
              <ErrorState
                message={formatApiErrorForUser(searchQuery.error, 'Не удалось выполнить поиск')}
                onRetry={() => searchQuery.refetch()}
              />
            )}
            {!searchQuery.isLoading && !searchQuery.isError && filteredResults.length === 0 && (
              <EmptyState message="Ничего не найдено" />
            )}
            {!searchQuery.isLoading &&
              !searchQuery.isError &&
              filteredResults.map((result) => {
                const Icon = resultIcon(result.result_type)
                return (
                  <Link
                    key={`${result.result_type}-${result.id}`}
                    to={result.url}
                    className="block rounded-md border border-border p-3 hover:bg-surface-raised"
                  >
                    <div className="flex items-start gap-3">
                      <Icon className="mt-0.5 h-4 w-4 text-accent" />
                      <div className="min-w-0">
                        <div className="text-sm font-medium text-text-primary">{result.title}</div>
                        <div className="mt-1 text-xs text-text-muted">
                          {resultLabel(result.result_type)} · {result.space_key} ·{' '}
                          {formatDateTime(result.updated_at)}
                        </div>
                        <p className="mt-2 line-clamp-2 text-sm text-text-secondary">
                          {result.snippet}
                        </p>
                      </div>
                    </div>
                  </Link>
                )
              })}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">Фасеты</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3 text-sm text-text-secondary">
            <div className="flex justify-between gap-3">
              <span>Документы</span>
              <span>{countResults(results, 'document')}</span>
            </div>
            <div className="flex justify-between gap-3">
              <span>Материалы</span>
              <span>{countResults(results, 'evidence')}</span>
            </div>
            <div className="flex justify-between gap-3">
              <span>Всего</span>
              <span>{results.length}</span>
            </div>
          </CardContent>
        </Card>
      </section>
    </div>
  )
}
