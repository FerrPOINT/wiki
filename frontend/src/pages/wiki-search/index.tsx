import { useMemo, useState } from 'react'
import { Link } from 'react-router'
import { FileCheck2, FileText, GitBranch, Search } from 'lucide-react'
import { defaultSpaceKey, useWikiSearch } from '@/shared/api/hooks'
import { EmptyState, ErrorState, LoadingState } from '@/shared/ui/async-states'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'
import { Input } from '@/shared/ui/input'
import { formatDateTime } from '@/shared/lib/wiki-format'
import type { SearchResult } from '@/api/wiki'

const filters = [
  { label: 'Все', value: 'all' },
  { label: 'Документы', value: 'document' },
  { label: 'Материалы', value: 'evidence' },
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

export function WikiSearchPage() {
  const [query, setQuery] = useState('')
  const [filter, setFilter] = useState('all')
  const searchQuery = useWikiSearch({ q: query, space: defaultSpaceKey, limit: 25 })
  const results = useMemo(() => searchQuery.data?.results ?? [], [searchQuery.data?.results])
  const filteredResults = useMemo(() => {
    if (filter === 'all') return results
    return results.filter((result) => result.result_type === filter)
  }, [filter, results])

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
            className="pl-9"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="релиз, SDLC-42, требования..."
          />
        </div>
        <div className="flex flex-wrap gap-2">
          {filters.map((item) => (
            <button
              key={item.value}
              type="button"
              className={`rounded-md border px-3 py-1.5 text-xs ${
                filter === item.value
                  ? 'border-accent bg-accent text-white'
                  : 'border-border text-text-secondary hover:bg-surface-raised'
              }`}
              onClick={() => setFilter(item.value)}
            >
              {item.label}
            </button>
          ))}
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
                message="Не удалось выполнить поиск"
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
