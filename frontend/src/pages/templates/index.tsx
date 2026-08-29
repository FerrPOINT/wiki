import { Link } from 'react-router'
import { ClipboardCheck, FileText, ShieldCheck } from 'lucide-react'
import { Button } from '@/shared/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'

const templates = [
  {
    name: 'Требования',
    description: 'Цели, границы, критерии приёмки, NFR и трассировка для задачи.',
    icon: ClipboardCheck,
    space: 'ENG',
  },
  {
    name: 'Исследование',
    description: 'Контекст, варианты решения, выводы и ссылки на материалы.',
    icon: FileText,
    space: 'ENG',
  },
  {
    name: 'Заметки реализации',
    description: 'Что изменено, какие документы затронуты и какие риски остаются.',
    icon: FileText,
    space: 'SDLC',
  },
  {
    name: 'План проверки',
    description: 'Сценарии проверки, ожидаемые результаты и ссылки на материалы.',
    icon: ShieldCheck,
    space: 'SDLC',
  },
  {
    name: 'Заметка к релизу',
    description: 'Изменения, влияние, порядок выкладки и ссылки на проверку.',
    icon: ShieldCheck,
    space: 'OPS',
  },
]

export function TemplatesPage() {
  return (
    <div className="space-y-5">
      <section className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold">Шаблоны</h1>
          <p className="mt-1 max-w-3xl text-sm text-text-muted">
            Стартовые структуры документов, чтобы задачи и фазы были описаны единообразно.
          </p>
        </div>
        <Button asChild size="sm">
          <Link to="/documents/new">
            <FileText className="h-4 w-4" />
            Создать документ
          </Link>
        </Button>
      </section>

      <section className="grid gap-4 lg:grid-cols-2">
        {templates.map((template) => {
          const Icon = template.icon
          return (
            <Card key={template.name}>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base">
                  <Icon className="h-4 w-4 text-accent" />
                  {template.name}
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                <p className="text-sm text-text-secondary">{template.description}</p>
                <div className="flex items-center justify-between gap-3 text-xs text-text-muted">
                  <span>Пространство по умолчанию: {template.space}</span>
                  <Link to="/documents/new" className="text-sm text-accent hover:text-accent-hover">
                    Использовать
                  </Link>
                </div>
              </CardContent>
            </Card>
          )
        })}
      </section>
    </div>
  )
}
