import {
  PageHeader,
  Prose,
  LoadingState,
  NamespaceLink as Link,
  ErrorState,
  TextLink,
} from '@pdlc/ui/ui'
import { useParams } from 'react-router'
import { withNamespaceLocation } from '@pdlc/ui/lib'
import { useDocument, useDocumentRevision } from '@/shared/api/hooks'
import { useNamespaceContext } from '@/widgets/namespace-context'
import { TaskRevisionLink } from '@/features/managed-links/TaskRevisionLink'

export function DocumentRevisionPage() {
  const { documentId = '', revisionId = '' } = useParams()
  const document = useDocument(documentId)
  const revision = useDocumentRevision(documentId, revisionId)
  const { ref, malformed, query: context } = useNamespaceContext()
  if (document.isPending || revision.isPending || (ref && context.isPending))
    return <LoadingState message="Загружаем опубликованную ревизию…" />
  if (
    malformed ||
    document.isError ||
    revision.isError ||
    !document.data ||
    !revision.data ||
    (ref && (context.isError || context.data?.resource_key !== document.data.space_key))
  )
    return <ErrorState message="Ревизия недоступна в выбранном проекте." />
  return (
    <article className="space-y-5">
      <TextLink asChild>
        <Link to={withNamespaceLocation(`/documents/${documentId}`, ref)}>
          Все версии документа
        </Link>
      </TextLink>
      <PageHeader
        title={revision.data.title}
        context={
          <>
            Опубликованная ревизия
            {revision.data.version}·{revision.data.published_at}
          </>
        }
      />
      <Prose
        className="page-readable"
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
