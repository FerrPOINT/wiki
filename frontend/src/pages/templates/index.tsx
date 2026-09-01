import { FormEvent, useState } from 'react'
import { Link } from 'react-router'
import { ClipboardCheck, FileText, Plus, ShieldCheck } from 'lucide-react'
import { useCreateTemplate, useTemplates } from '@/shared/api/hooks'
import { EmptyState, ErrorState, LoadingState } from '@/shared/ui/async-states'
import { Button } from '@/shared/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'
import { Input } from '@/shared/ui/input'
import { Label } from '@/shared/ui/label'
import { Textarea } from '@/shared/ui/textarea'
import { formatApiErrorForUser } from '@/shared/lib/api-error'
import { formatDocumentType } from '@/shared/lib/wiki-format'

const typeOptions = [
  'page',
  'requirements',
  'research_note',
  'implementation_note',
  'test_plan',
  'release_note',
]

const selectClassName =
  'flex h-9 w-full rounded-md border border-border-strong bg-surface px-3 py-1 text-sm text-text-primary shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent disabled:cursor-not-allowed disabled:opacity-50'

function templateIcon(documentType: string) {
  if (documentType === 'requirements') return ClipboardCheck
  if (documentType === 'test_plan' || documentType === 'release_note') return ShieldCheck
  return FileText
}

export function TemplatesPage() {
  const templatesQuery = useTemplates()
  const createTemplate = useCreateTemplate()
  const templates = templatesQuery.data?.templates ?? []
  const [name, setName] = useState('План проверки')
  const [documentType, setDocumentType] = useState('test_plan')
  const [body, setBody] = useState('# План проверки\n\n## Сценарии\n\n## Риски\n\n## Материалы\n')

  function handleCreateTemplate(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    createTemplate.mutate(
      {
        name: name.trim(),
        document_type: documentType,
        body_markdown: body.trim(),
      },
      {
        onSuccess: () => {
          setName('')
          setDocumentType('requirements')
          setBody('')
        },
      },
    )
  }

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

      <form
        onSubmit={handleCreateTemplate}
        className="rounded-md border border-border bg-surface p-3"
      >
        <div className="grid gap-3 lg:grid-cols-[1fr_14rem]">
          <div className="space-y-1.5">
            <Label htmlFor="template-name">Название шаблона</Label>
            <Input
              id="template-name"
              value={name}
              onChange={(event) => setName(event.target.value)}
              required
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="template-type">Тип документа</Label>
            <select
              id="template-type"
              className={selectClassName}
              value={documentType}
              onChange={(event) => setDocumentType(event.target.value)}
            >
              {typeOptions.map((type) => (
                <option key={type} value={type}>
                  {formatDocumentType(type)}
                </option>
              ))}
            </select>
          </div>
        </div>
        <div className="mt-3 space-y-2">
          <Label htmlFor="template-body">Markdown шаблона</Label>
          <Textarea
            id="template-body"
            className="min-h-32 font-mono text-sm"
            value={body}
            onChange={(event) => setBody(event.target.value)}
            required
          />
          <div className="flex justify-end">
            <Button disabled={createTemplate.isPending}>
              <Plus className="h-4 w-4" />
              {createTemplate.isPending ? 'Создаём...' : 'Создать шаблон'}
            </Button>
          </div>
        </div>
        {createTemplate.isError && (
          <p className="mt-2 text-sm text-danger">
            {formatApiErrorForUser(createTemplate.error, 'Не удалось создать шаблон')}
          </p>
        )}
      </form>

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
