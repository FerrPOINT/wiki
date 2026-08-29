import { Link } from 'react-router'
import { CheckCircle2, FilePlus2, FileText, GitBranch, Library, Search } from 'lucide-react'
import { Button } from '@/shared/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'

const recentDocuments = [
  {
    title: 'Требования к Wiki',
    href: '/documents/product-requirements',
    space: 'ENG',
    status: 'published',
    updated: 'сегодня',
  },
  {
    title: 'Материалы по фазе реализации',
    href: '/documents/implementation-evidence',
    space: 'SDLC',
    status: 'draft',
    updated: 'вчера',
  },
  {
    title: 'Чеклист релиза',
    href: '/documents/release-checklist',
    space: 'OPS',
    status: 'published',
    updated: '2 дня назад',
  },
]

const phaseGaps = [
  { phase: 'Анализ', task: 'SDLC-42', missing: 'Нет итогового документа требований' },
  { phase: 'Проверка', task: 'SDLC-39', missing: 'Не прикреплена ссылка на проверку' },
  { phase: 'Релиз', task: 'SDLC-37', missing: 'Нужна заметка к релизу' },
]

export function DashboardPage() {
  return (
    <div className="space-y-6">
      <section className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold">Wiki</h1>
          <p className="mt-1 max-w-3xl text-sm text-text-muted">
            База знаний для документов по задачам SDLC и фазам выполненного workflow.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button asChild size="sm">
            <Link to="/documents/new">
              <FilePlus2 className="h-4 w-4" />
              Новый документ
            </Link>
          </Button>
          <Button asChild size="sm" variant="secondary">
            <Link to="/search">
              <Search className="h-4 w-4" />
              Найти
            </Link>
          </Button>
        </div>
      </section>

      <section className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Пространства</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-3">
              <Library className="h-5 w-5 text-accent" />
              <span className="text-2xl font-semibold">3</span>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Документы</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-3">
              <FileText className="h-5 w-5 text-accent" />
              <span className="text-2xl font-semibold">18</span>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Фазы</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-3">
              <GitBranch className="h-5 w-5 text-accent" />
              <span className="text-2xl font-semibold">11</span>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Материалы</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-3">
              <CheckCircle2 className="h-5 w-5 text-success" />
              <span className="text-2xl font-semibold">27</span>
            </div>
          </CardContent>
        </Card>
      </section>

      <section className="grid gap-4 lg:grid-cols-[1.5fr_1fr]">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Последние документы</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            {recentDocuments.map((document) => (
              <Link
                key={document.href}
                to={document.href}
                className="flex flex-col gap-2 rounded-md border border-border p-3 hover:bg-surface-raised sm:flex-row sm:items-center sm:justify-between"
              >
                <span>
                  <span className="block text-sm font-medium text-text-primary">
                    {document.title}
                  </span>
                  <span className="mt-1 block text-xs text-text-muted">
                    {document.space} · {document.updated}
                  </span>
                </span>
                <span className="w-fit rounded bg-surface-raised px-2 py-1 text-xs text-text-secondary">
                  {document.status === 'published' ? 'Опубликован' : 'Черновик'}
                </span>
              </Link>
            ))}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">Нужно закрыть</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            {phaseGaps.map((gap) => (
              <Link
                key={`${gap.task}-${gap.phase}`}
                to={`/tasks/${gap.task}`}
                className="block rounded-md border border-border p-3 hover:bg-surface-raised"
              >
                <div className="text-sm font-medium text-text-primary">
                  {gap.task} · {gap.phase}
                </div>
                <div className="mt-1 text-xs text-text-muted">{gap.missing}</div>
              </Link>
            ))}
          </CardContent>
        </Card>
      </section>
    </div>
  )
}
