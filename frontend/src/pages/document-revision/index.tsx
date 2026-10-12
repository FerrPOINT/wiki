import { NamespaceLink as Link } from '@sdlc/ui/ui'
import { useParams } from 'react-router'
import { withNamespaceLocation } from '@sdlc/ui/lib'
import { useDocument, useDocumentRevision } from '@/shared/api/hooks'
import { useNamespaceContext } from '@/widgets/namespace-context'
import { TaskRevisionLink } from '@/features/managed-links/TaskRevisionLink'

export function DocumentRevisionPage() {
  const { documentId = '', revisionId = '' } = useParams()
  const document = useDocument(documentId)
  const revision = useDocumentRevision(documentId, revisionId)
  const { ref, malformed, query: context } = useNamespaceContext()
  if (document.isPending || revision.isPending || (ref && context.isPending))
    return <p role="status">Загружаем опубликованную ревизию…</p>
  if (
    malformed ||
    document.isError ||
    revision.isError ||
    !document.data ||
    !revision.data ||
    (ref && (context.isError || context.data?.resource_key !== document.data.space_key))
  )
    return (
      <p role="alert" className="text-danger">
        Ревизия недоступна в выбранном проекте.
      </p>
    )
  return (
    <article className="space-y-5">
      <Link className="text-accent" to={withNamespaceLocation(`/documents/${documentId}`, ref)}>
        Все версии документа
      </Link>
      <h1 className="text-xl font-semibold">{revision.data.title}</h1>
      <p className="text-sm text-text-muted">
        Опубликованная ревизия {revision.data.version} · {revision.data.published_at}
      </p>
      <div
        className="wiki-rendered page-readable"
        dangerouslySetInnerHTML={{ __html: revision.data.body_html }}
      />
      <TaskRevisionLink
        documentId={documentId}
        revisionId={revisionId}
        spaceKey={document.data.space_key}
      />
    </article>
  )
}
