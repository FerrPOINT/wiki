import { Link } from 'react-router'
import { FileCheck2, FileText, GitBranch, Search } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'
import { Input } from '@/shared/ui/input'

const filters = ['Все', 'Документы', 'Задачи', 'Фазы', 'Материалы']

const results = [
  {
    type: 'документ',
    title: 'Требования к Wiki',
    location: 'ENG / Требования',
    href: '/documents/product-requirements',
    icon: FileText,
  },
  {
    type: 'задача',
    title: 'SDLC-42',
    location: 'Трекер / SDLC-42',
    href: '/tasks/SDLC-42',
    icon: FileText,
  },
  {
    type: 'материал',
    title: 'backend-tests #1842',
    location: 'Фаза реализации',
    href: '/evidence',
    icon: FileCheck2,
  },
  {
    type: 'фаза',
    title: 'Реализация',
    location: 'SDLC-42 / workflow',
    href: '/phases/implementation',
    icon: GitBranch,
  },
]

export function WikiSearchPage() {
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
          <Input className="pl-9" placeholder="релиз, SDLC-42, требования..." />
        </div>
        <div className="flex flex-wrap gap-2">
          {filters.map((filter) => (
            <button
              key={filter}
              type="button"
              className={`rounded-md border px-3 py-1.5 text-xs ${
                filter === 'Все'
                  ? 'border-accent bg-accent text-white'
                  : 'border-border text-text-secondary hover:bg-surface-raised'
              }`}
            >
              {filter}
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
            {results.map((result) => {
              const Icon = result.icon
              return (
                <Link
                  key={`${result.type}-${result.title}`}
                  to={result.href}
                  className="block rounded-md border border-border p-3 hover:bg-surface-raised"
                >
                  <div className="flex items-start gap-3">
                    <Icon className="mt-0.5 h-4 w-4 text-accent" />
                    <div className="min-w-0">
                      <div className="text-sm font-medium text-text-primary">{result.title}</div>
                      <div className="mt-1 text-xs text-text-muted">
                        {result.type} · {result.location}
                      </div>
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
              <span>12</span>
            </div>
            <div className="flex justify-between gap-3">
              <span>Задачи</span>
              <span>4</span>
            </div>
            <div className="flex justify-between gap-3">
              <span>Фазы</span>
              <span>6</span>
            </div>
            <div className="flex justify-between gap-3">
              <span>Материалы</span>
              <span>9</span>
            </div>
          </CardContent>
        </Card>
      </section>
    </div>
  )
}
