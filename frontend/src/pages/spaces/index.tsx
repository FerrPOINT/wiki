import { Link } from 'react-router'
import { BookOpenText, FileText, FolderOpen, Users } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'

const spaces = [
  {
    key: 'ENG',
    name: 'Инженерия',
    description: 'Требования, ADR, инструкции и технические решения.',
    documents: 9,
    members: 6,
    updated: 'сегодня',
    tree: ['Требования к Wiki', 'Архитектурные решения', 'API и CLI'],
  },
  {
    key: 'SDLC',
    name: 'SDLC workflow',
    description: 'Документы и материалы по фазам workflow.',
    documents: 7,
    members: 4,
    updated: 'вчера',
    tree: ['SDLC-42', 'Фазы реализации', 'Материалы проверки'],
  },
  {
    key: 'OPS',
    name: 'Эксплуатация',
    description: 'Релизы, проверки, мониторинг и заметки по инцидентам.',
    documents: 2,
    members: 3,
    updated: '2 дня назад',
    tree: ['Чеклист релиза', 'Incident notes'],
  },
]

export function SpacesPage() {
  return (
    <div className="space-y-5">
      <section className="flex flex-col gap-2">
        <h1 className="text-2xl font-bold">Пространства</h1>
        <p className="max-w-3xl text-sm text-text-muted">
          Пространства группируют документы по продуктам, командам и workflow-контекстам.
        </p>
      </section>

      <section className="grid gap-4 lg:grid-cols-3">
        {spaces.map((space) => (
          <Card key={space.key}>
            <CardHeader>
              <div className="flex items-start justify-between gap-3">
                <CardTitle className="flex items-center gap-2 text-base">
                  <FolderOpen className="h-4 w-4 text-accent" />
                  {space.key} · {space.name}
                </CardTitle>
                <span className="rounded bg-surface-raised px-2 py-1 text-xs text-text-muted">
                  {space.updated}
                </span>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              <p className="text-sm text-text-muted">{space.description}</p>
              <div className="flex gap-3 text-xs text-text-secondary">
                <span className="inline-flex items-center gap-1">
                  <FileText className="h-3.5 w-3.5" />
                  {space.documents}
                </span>
                <span className="inline-flex items-center gap-1">
                  <Users className="h-3.5 w-3.5" />
                  {space.members}
                </span>
              </div>
              <div className="space-y-2 rounded-md border border-border p-3">
                <div className="flex items-center gap-2 text-xs font-medium uppercase text-text-muted">
                  <BookOpenText className="h-3.5 w-3.5" />
                  Дерево
                </div>
                <div className="space-y-1.5">
                  {space.tree.map((document) => (
                    <Link
                      key={document}
                      to={`/documents/${space.key.toLowerCase()}-home`}
                      className="block truncate text-sm text-text-secondary hover:text-accent"
                    >
                      {document}
                    </Link>
                  ))}
                </div>
              </div>
              <Link
                to={`/documents/${space.key.toLowerCase()}-home`}
                className="inline-flex text-sm text-accent hover:text-accent-hover"
              >
                Открыть дерево документов
              </Link>
            </CardContent>
          </Card>
        ))}
      </section>
    </div>
  )
}
