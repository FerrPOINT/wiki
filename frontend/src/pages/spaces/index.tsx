import { Link } from 'react-router'
import { BookOpenText, FileText, FolderOpen, Users } from 'lucide-react'
import { defaultSpaceKey, useSpaceTree, useSpaces } from '@/shared/api/hooks'
import { EmptyState, ErrorState, LoadingState } from '@/shared/ui/async-states'
import { Button } from '@/shared/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'
import { formatDateTime, formatDocumentType, shortText } from '@/shared/lib/wiki-format'
import type { Space, SpaceTreeNode } from '@/api/wiki'

function flattenTree(nodes: SpaceTreeNode[], limit = 5): SpaceTreeNode[] {
  const result: SpaceTreeNode[] = []
  const visit = (items: SpaceTreeNode[]) => {
    for (const item of items) {
      if (result.length >= limit) return
      result.push(item)
      visit(item.children)
    }
  }
  visit(nodes)
  return result
}

function SpaceTreePreview({ spaceKey }: { spaceKey: string }) {
  const treeQuery = useSpaceTree(spaceKey)
  const documents = flattenTree(treeQuery.data?.documents ?? [])

  if (treeQuery.isLoading) return <LoadingState message="Загружаем дерево" />
  if (treeQuery.isError) return <ErrorState message="Не удалось загрузить дерево" />
  if (documents.length === 0) return <EmptyState message="В пространстве пока нет документов" />

  return (
    <div className="space-y-1.5">
      {documents.map((document) => (
        <Link
          key={document.id}
          to={`/documents/${document.slug}`}
          className="block truncate text-sm text-text-secondary hover:text-accent"
        >
          {document.title}
          <span className="ml-2 text-xs text-text-muted">
            {formatDocumentType(document.document_type)}
          </span>
        </Link>
      ))}
    </div>
  )
}

function SpaceCard({ space }: { space: Space }) {
  return (
    <Card>
      <CardHeader>
        <div className="flex items-start justify-between gap-3">
          <CardTitle className="flex items-center gap-2 text-base">
            <FolderOpen className="h-4 w-4 text-accent" />
            {space.key} · {space.name}
          </CardTitle>
          <span className="rounded bg-surface-raised px-2 py-1 text-xs text-text-muted">
            {formatDateTime(space.updated_at)}
          </span>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <p className="text-sm text-text-muted">{shortText(space.description)}</p>
        <div className="flex gap-3 text-xs text-text-secondary">
          <span className="inline-flex items-center gap-1">
            <FileText className="h-3.5 w-3.5" />
            {space.document_count}
          </span>
          <span className="inline-flex items-center gap-1">
            <Users className="h-3.5 w-3.5" />
            {space.member_count}
          </span>
        </div>
        <div className="space-y-2 rounded-md border border-border p-3">
          <div className="flex items-center gap-2 text-xs font-medium uppercase text-text-muted">
            <BookOpenText className="h-3.5 w-3.5" />
            Дерево
          </div>
          <SpaceTreePreview spaceKey={space.key} />
        </div>
      </CardContent>
    </Card>
  )
}

export function SpacesPage() {
  const spacesQuery = useSpaces()
  const spaces = spacesQuery.data?.spaces ?? []

  return (
    <div className="space-y-5">
      <section className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold">Пространства</h1>
          <p className="max-w-3xl text-sm text-text-muted">
            Пространства группируют документы по продуктам, командам и контекстам процесса.
          </p>
        </div>
        <Button asChild size="sm">
          <Link to="/documents/new">Новый документ</Link>
        </Button>
      </section>

      {spacesQuery.isLoading && <LoadingState message="Загружаем пространства" />}
      {spacesQuery.isError && <ErrorState message="Не удалось загрузить пространства" />}
      {!spacesQuery.isLoading && !spacesQuery.isError && spaces.length === 0 && (
        <EmptyState
          message="Пространства ещё не созданы"
          action={
            <Button asChild size="sm" variant="secondary">
              <Link to={`/documents/new?space=${defaultSpaceKey}`}>Создать первый документ</Link>
            </Button>
          }
        />
      )}
      {!spacesQuery.isLoading && !spacesQuery.isError && spaces.length > 0 && (
        <section className="grid gap-4 lg:grid-cols-3">
          {spaces.map((space) => (
            <SpaceCard key={space.key} space={space} />
          ))}
        </section>
      )}
    </div>
  )
}
