import { Link, useParams } from 'react-router'
import {
  CheckCircle2,
  Clock3,
  FileCheck2,
  FilePenLine,
  FileText,
  GitBranch,
  History,
  Link2,
} from 'lucide-react'
import { Button } from '@/shared/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'

const revisions = [
  { version: 3, summary: 'Уточнены границы MVP', author: 'Техлид', time: 'сегодня' },
  { version: 2, summary: 'Добавлены связи с фазами', author: 'Редактор', time: 'вчера' },
  {
    version: 1,
    summary: 'Базовая версия требований',
    author: 'Администратор',
    time: '2 дня назад',
  },
]

const relatedDocuments = ['Архитектура Wiki', 'План проверки', 'Заметка к релизу']
const relatedMaterials = ['PR: wiki-mvp-docs', 'Smoke-проверка frontend', 'Скриншоты страниц']

export function DocumentPage() {
  const { documentId = 'product-requirements' } = useParams()

  return (
    <article className="space-y-5">
      <section className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
        <div>
          <div className="text-sm text-text-muted">ENG / Требования / {documentId}</div>
          <h1 className="mt-2 text-2xl font-bold">Требования к Wiki</h1>
          <div className="mt-3 flex flex-wrap gap-2 text-xs text-text-secondary">
            <span className="inline-flex items-center gap-1 rounded bg-surface-raised px-2 py-1">
              <CheckCircle2 className="h-3.5 w-3.5 text-success" />
              опубликован
            </span>
            <span className="inline-flex items-center gap-1 rounded bg-surface-raised px-2 py-1">
              <Clock3 className="h-3.5 w-3.5" />
              ревизия 3
            </span>
            <span className="rounded bg-surface-raised px-2 py-1">обновлено сегодня</span>
          </div>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button asChild size="sm" variant="secondary">
            <Link to="/documents/new">
              <FilePenLine className="h-4 w-4" />
              Создать черновик
            </Link>
          </Button>
          <Button asChild size="sm" variant="outline">
            <Link to="/evidence">
              <FileCheck2 className="h-4 w-4" />
              Материалы
            </Link>
          </Button>
        </div>
      </section>

      <section className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_22rem]">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Содержимое</CardTitle>
          </CardHeader>
          <CardContent className="space-y-5 text-sm leading-6 text-text-secondary">
            <section className="space-y-2">
              <h2 className="text-lg font-semibold text-text-primary">Назначение</h2>
              <p>
                Wiki хранит важные документы по задаче и по каждой фазе выполненного workflow:
                требования, архитектурные решения, заметки реализации, планы проверки, материалы и
                заметки к релизу.
              </p>
            </section>
            <section className="space-y-2">
              <h2 className="text-lg font-semibold text-text-primary">MVP</h2>
              <ul className="list-disc space-y-1 pl-5">
                <li>Markdown-документы с черновиком, публикацией и неизменяемыми ревизиями.</li>
                <li>Пространства, дерево страниц и базовые роли доступа.</li>
                <li>Связи документов с внешним ключом задачи и ключом фазы.</li>
                <li>URL-ссылки и файлы с метаданными, поиском и аудитом.</li>
              </ul>
            </section>
            <section className="space-y-2">
              <h2 className="text-lg font-semibold text-text-primary">Граница продукта</h2>
              <p>
                UI и CLI работают через один публичный API. Wiki не исполняет workflow, не заменяет
                трекер задач и не добавляет отдельный контекст исполнения.
              </p>
            </section>
          </CardContent>
        </Card>

        <div className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Связанные объекты</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3 text-sm">
              <Link
                to="/tasks/SDLC-42"
                className="flex items-center gap-2 rounded-md border border-border p-3 hover:bg-surface-raised"
              >
                <FileText className="h-4 w-4 text-accent" />
                SDLC-42
              </Link>
              <Link
                to="/phases/implementation"
                className="flex items-center gap-2 rounded-md border border-border p-3 hover:bg-surface-raised"
              >
                <GitBranch className="h-4 w-4 text-accent" />
                Реализация
              </Link>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-base">Ревизии</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              {revisions.map((revision) => (
                <div key={revision.version} className="rounded-md border border-border p-3">
                  <div className="flex items-center justify-between gap-3 text-sm font-medium">
                    <span className="inline-flex items-center gap-2">
                      <History className="h-4 w-4 text-accent" />
                      Ревизия {revision.version}
                    </span>
                    <span className="text-xs text-text-muted">{revision.time}</span>
                  </div>
                  <p className="mt-1 text-xs text-text-secondary">{revision.summary}</p>
                  <p className="mt-1 text-xs text-text-muted">{revision.author}</p>
                </div>
              ))}
            </CardContent>
          </Card>
        </div>
      </section>

      <section className="grid gap-4 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Связанные документы</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            {relatedDocuments.map((title) => (
              <Link
                key={title}
                to="/documents/product-requirements"
                className="flex items-center gap-2 rounded-md border border-border p-3 text-sm hover:bg-surface-raised"
              >
                <FileText className="h-4 w-4 text-accent" />
                {title}
              </Link>
            ))}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">Связанные материалы</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            {relatedMaterials.map((title) => (
              <Link
                key={title}
                to="/evidence"
                className="flex items-center gap-2 rounded-md border border-border p-3 text-sm hover:bg-surface-raised"
              >
                <Link2 className="h-4 w-4 text-accent" />
                {title}
              </Link>
            ))}
          </CardContent>
        </Card>
      </section>
    </article>
  )
}
