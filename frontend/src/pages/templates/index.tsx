import { Link } from 'react-router'
import { ClipboardCheck, FileText, ShieldCheck } from 'lucide-react'
import { useTemplates } from '@/shared/api/hooks'
import { EmptyState, ErrorState, LoadingState } from '@/shared/ui/async-states'
import { Button } from '@/shared/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'
import { formatApiErrorForUser } from '@/shared/lib/api-error'
import { formatDocumentType } from '@/shared/lib/wiki-format'

function templateIcon(documentType: string) {
  if (documentType === 'requirements') return ClipboardCheck
  if (documentType === 'test_plan' || documentType === 'release_note') return ShieldCheck
  return FileText
}

export function TemplatesPage() {
  const templatesQuery = useTemplates()
  const templates = templatesQuery.data?.templates ?? []

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

      {templatesQuery.isLoading && <LoadingState message="Загружаем шаблоны" />}
      {templatesQuery.isError && (
        <ErrorState
          message={formatApiErrorForUser(templatesQuery.error, 'Не удалось загрузить шаблоны')}
          onRetry={() => templatesQuery.refetch()}
        />
      )}
      {!templatesQuery.isLoading && !templatesQuery.isError && templates.length === 0 && (
        <EmptyState message="Шаблоны ещё не созданы" />
      )}
      {!templatesQuery.isLoading && !templatesQuery.isError && templates.length > 0 && (
        <section className="grid gap-4 lg:grid-cols-2">
          {templates.map((template) => {
            const Icon = templateIcon(template.document_type)
            return (
              <Card key={template.id}>
                <CardHeader>
                  <CardTitle className="flex items-center gap-2 text-base">
                    <Icon className="h-4 w-4 text-accent" />
                    {template.name}
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-3">
                  <p className="text-sm text-text-secondary">
                    {formatDocumentType(template.document_type)}
                  </p>
                  <pre className="max-h-40 overflow-auto whitespace-pre-wrap rounded-md border border-border bg-background p-3 text-xs text-text-muted">
                    {template.body_markdown}
                  </pre>
                  <div className="flex items-center justify-between gap-3 text-xs text-text-muted">
                    <span>{template.id}</span>
                    <Link
                      to={`/documents/new?template=${template.id}`}
                      className="text-sm text-accent hover:text-accent-hover"
                    >
                      Использовать
                    </Link>
                  </div>
                </CardContent>
              </Card>
            )
          })}
        </section>
      )}
    </div>
  )
}
