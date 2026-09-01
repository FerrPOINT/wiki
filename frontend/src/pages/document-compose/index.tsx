import { FormEvent, useEffect, useState } from 'react'
import { Link, useNavigate, useSearchParams } from 'react-router'
import { FileText, GitBranch, Save, Tag } from 'lucide-react'
import { defaultSpaceKey, useCreateDocument, useSpaces, useTemplates } from '@/shared/api/hooks'
import { ErrorState, LoadingState } from '@/shared/ui/async-states'
import { Button } from '@/shared/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'
import { Input } from '@/shared/ui/input'
import { Label } from '@/shared/ui/label'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/shared/ui/tabs'
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

function normalizeOptional(value: string) {
  const trimmed = value.trim()
  return trimmed.length > 0 ? trimmed : null
}

export function DocumentComposePage() {
  const navigate = useNavigate()
  const [searchParams] = useSearchParams()
  const initialSpace = (searchParams.get('space') ?? defaultSpaceKey).toUpperCase()
  const requestedTemplateId = searchParams.get('template')?.trim() ?? ''
  const [title, setTitle] = useState('')
  const [body, setBody] = useState('')
  const [spaceKey, setSpaceKey] = useState(initialSpace)
  const [documentType, setDocumentType] = useState('page')
  const [slug, setSlug] = useState('')
  const [taskKey, setTaskKey] = useState('')
  const [phaseKey, setPhaseKey] = useState('')
  const [appliedTemplateId, setAppliedTemplateId] = useState('')
  const spacesQuery = useSpaces()
  const templatesQuery = useTemplates()
  const createDocument = useCreateDocument()

  function applyTemplate(templateId: string) {
    const template = templatesQuery.data?.templates.find((item) => item.id === templateId)
    if (!template) return
    setDocumentType(template.document_type)
    setBody(template.body_markdown)
    setAppliedTemplateId(template.id)
  }

  useEffect(() => {
    if (!requestedTemplateId || appliedTemplateId === requestedTemplateId) return
    const template = templatesQuery.data?.templates.find((item) => item.id === requestedTemplateId)
    if (!template) return
    setDocumentType(template.document_type)
    setBody(template.body_markdown)
    setAppliedTemplateId(template.id)
  }, [appliedTemplateId, requestedTemplateId, templatesQuery.data?.templates])

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    createDocument.mutate(
      {
        spaceKey: spaceKey.trim().toUpperCase() || defaultSpaceKey,
        body: {
          title: title.trim(),
          slug: normalizeOptional(slug),
          document_type: documentType,
          parent_id: null,
          content_markdown: body,
          task_key: normalizeOptional(taskKey),
          phase_key: normalizeOptional(phaseKey),
        },
      },
      {
        onSuccess: (document) => navigate(`/documents/${document.slug}`),
      },
    )
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-5">
      <section className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold">Новый документ</h1>
          <p className="mt-1 max-w-3xl text-sm text-text-muted">
            Черновик можно связать с задачей, фазой процесса и материалами проверки.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button type="button" size="sm" variant="secondary">
            <FileText className="h-4 w-4" />
            Предпросмотр
          </Button>
          <Button size="sm" disabled={createDocument.isPending}>
            <Save className="h-4 w-4" />
            {createDocument.isPending ? 'Сохраняем...' : 'Сохранить черновик'}
          </Button>
        </div>
      </section>

      {createDocument.isError && (
        <ErrorState
          message={formatApiErrorForUser(createDocument.error, 'Не удалось сохранить документ')}
        />
      )}

      <section className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_20rem]">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Редактор</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid gap-4 md:grid-cols-[1fr_12rem]">
              <div className="space-y-1.5">
                <Label htmlFor="document-title">Название</Label>
                <Input
                  id="document-title"
                  value={title}
                  onChange={(event) => setTitle(event.target.value)}
                  placeholder="Название документа"
                  required
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="document-space">Пространство</Label>
                <Input
                  id="document-space"
                  list="document-space-options"
                  value={spaceKey}
                  onChange={(event) => setSpaceKey(event.target.value.toUpperCase())}
                  required
                />
                <datalist id="document-space-options">
                  {(spacesQuery.data?.spaces ?? []).map((space) => (
                    <option key={space.key} value={space.key}>
                      {space.name}
                    </option>
                  ))}
                </datalist>
              </div>
            </div>

            <div className="flex flex-wrap gap-2">
              {templatesQuery.isLoading && <LoadingState message="Загружаем шаблоны" />}
              {(templatesQuery.data?.templates ?? []).map((template) => (
                <button
                  key={template.id}
                  type="button"
                  className="rounded-md border border-border px-2.5 py-1.5 text-xs text-text-secondary hover:bg-surface-raised hover:text-text-primary"
                  onClick={() => applyTemplate(template.id)}
                >
                  {template.name}
                </button>
              ))}
            </div>

            <Tabs defaultValue="write">
              <TabsList>
                <TabsTrigger value="write">Markdown</TabsTrigger>
                <TabsTrigger value="preview">Просмотр</TabsTrigger>
              </TabsList>
              <TabsContent value="write">
                <Textarea
                  className="min-h-96 font-mono text-sm"
                  aria-label="Markdown документа"
                  value={body}
                  onChange={(event) => setBody(event.target.value)}
                  placeholder="Начните писать Markdown или примените шаблон выше"
                  required
                />
              </TabsContent>
              <TabsContent value="preview">
                <div className="min-h-96 rounded-md border border-border bg-background p-4">
                  <h2 className="text-xl font-semibold">{title}</h2>
                  <pre className="mt-4 whitespace-pre-wrap text-sm leading-6 text-text-secondary">
                    {body}
                  </pre>
                </div>
              </TabsContent>
            </Tabs>
          </CardContent>
        </Card>

        <div className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Связи</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3 text-sm">
              <div className="space-y-1.5">
                <Label htmlFor="document-task">Задача</Label>
                <Input
                  id="document-task"
                  value={taskKey}
                  onChange={(event) => setTaskKey(event.target.value)}
                  placeholder="Ключ задачи"
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="document-phase">Фаза</Label>
                <Input
                  id="document-phase"
                  value={phaseKey}
                  onChange={(event) => setPhaseKey(event.target.value)}
                  placeholder="Ключ фазы"
                />
              </div>
              {taskKey && (
                <Link
                  to={`/tasks/${taskKey}`}
                  className="flex items-center gap-2 rounded-md border border-border p-3 hover:bg-surface-raised"
                >
                  <FileText className="h-4 w-4 text-accent" />
                  Задача {taskKey}
                </Link>
              )}
              {phaseKey && (
                <Link
                  to={`/phases/${phaseKey}`}
                  className="flex items-center gap-2 rounded-md border border-border p-3 hover:bg-surface-raised"
                >
                  <GitBranch className="h-4 w-4 text-accent" />
                  Фаза {phaseKey}
                </Link>
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-base">Метаданные</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <div className="space-y-1.5">
                <Label htmlFor="document-type">Тип</Label>
                <Input
                  id="document-type"
                  list="document-type-options"
                  value={documentType}
                  onChange={(event) => setDocumentType(event.target.value)}
                />
                <datalist id="document-type-options">
                  {typeOptions.map((type) => (
                    <option key={type} value={type}>
                      {formatDocumentType(type)}
                    </option>
                  ))}
                </datalist>
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="document-slug">Адрес (slug)</Label>
                <Input
                  id="document-slug"
                  value={slug}
                  onChange={(event) => setSlug(event.target.value)}
                  placeholder="создастся автоматически"
                />
              </div>
              <div className="flex items-center gap-2 rounded-md border border-border px-3 py-2 text-sm text-text-secondary">
                <Tag className="h-4 w-4 text-accent" />
                Markdown, ревизии, связи
              </div>
            </CardContent>
          </Card>
        </div>
      </section>
    </form>
  )
}
