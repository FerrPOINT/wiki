import { useActiveSpaces } from '@/widgets/active-spaces'
import { useNamespaceContext } from '@/widgets/namespace-context'
import { documentPath } from '@/api/workspaces'
import { FormEvent, useEffect, useState } from 'react'
import Markdown from 'react-markdown'
import {
  PageHeader,
  Badge,
  Prose,
  CodeBlock,
  NamespaceLink as Link,
  Button,
  useNamespaceNavigate as useNavigate,
  ErrorState,
  FormField,
  LoadingState,
  SearchableSelect,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  Input,
  Label,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  Textarea,
  UnsavedChangesGuard,
  Notice,
} from '@pdlc/ui/ui'

import { useSearchParams } from 'react-router'
import { FileText, GitBranch, Save, Tag } from 'lucide-react'
import { useCreateDocument, useTemplates } from '@/shared/api/hooks'
import { formatApiErrorForUser } from '@/shared/lib/api-error'
import { formatDocumentType } from '@/shared/lib/wiki-format'
import { resolveSpaceKey } from '@/shared/lib/space-selection'

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
  const namespace = useNamespaceContext()
  const [searchParams] = useSearchParams()
  const initialSpace = searchParams.get('space')?.trim().toUpperCase() ?? ''
  const requestedTemplateId = searchParams.get('template')?.trim() ?? ''
  const [title, setTitle] = useState('')
  const [body, setBody] = useState('')
  const [spaceKey, setSpaceKey] = useState(initialSpace)
  const [spaceChosen, setSpaceChosen] = useState(false)
  const [spaceSearch, setSpaceSearch] = useState('')
  const [spaceOffset, setSpaceOffset] = useState(0)
  const [documentType, setDocumentType] = useState('page')
  const [slug, setSlug] = useState('')
  const [taskKey, setTaskKey] = useState('')
  const [phaseKey, setPhaseKey] = useState('')
  const [templateInitialized, setTemplateInitialized] = useState(false)
  const [editorTab, setEditorTab] = useState<'write' | 'preview'>('write')
  const [createdDocumentPath, setCreatedDocumentPath] = useState<string | null>(null)
  const spacesQuery = useActiveSpaces()
  const templatesQuery = useTemplates()
  const createDocument = useCreateDocument()
  const availableSpaces = (spacesQuery.data?.spaces ?? []).filter(
    (space) => space.status === 'active',
  )
  const selectedSpace = availableSpaces.find(
    (space) => space.key.toUpperCase() === spaceKey.toUpperCase(),
  )
  const filteredSpaces = availableSpaces.filter((space) =>
    `${space.name} ${space.key}`
      .toLocaleLowerCase()
      .includes(spaceSearch.trim().toLocaleLowerCase()),
  )
  const canSave =
    !namespace.malformed &&
    !namespace.conflict &&
    (namespace.ref
      ? !namespace.query.isPending &&
        !namespace.query.isError &&
        namespace.query.data?.status === 'active'
      : !spacesQuery.isLoading && !spacesQuery.isError && Boolean(selectedSpace))
  const spaceProblem = namespace.malformed
    ? 'Некорректная ссылка на пространство Namespace'
    : !namespace.ref && !spacesQuery.isLoading && !spacesQuery.isError && spaceKey && !selectedSpace
      ? 'Выбранное пространство недоступно. Выберите доступное пространство.'
      : undefined
  useEffect(() => {
    if (
      initialSpace ||
      spaceKey ||
      spaceChosen ||
      namespace.ref ||
      spacesQuery.isLoading ||
      spacesQuery.isError
    )
      return
    setSpaceKey(
      resolveSpaceKey(
        null,
        (spacesQuery.data?.spaces ?? []).filter((space) => space.status === 'active'),
      ),
    )
  }, [
    initialSpace,
    spaceKey,
    spaceChosen,
    namespace.ref,
    spacesQuery.isLoading,
    spacesQuery.isError,
    spacesQuery.data,
  ])

  function applyTemplate(templateId: string) {
    const template = templatesQuery.data?.templates.find((item) => item.id === templateId)
    if (!template) return
    setDocumentType(template.document_type)
    setBody(template.body_markdown)
    setTemplateInitialized(true)
  }

  useEffect(() => {
    if (!requestedTemplateId || templateInitialized) return
    const template = templatesQuery.data?.templates.find((item) => item.id === requestedTemplateId)
    if (!template) return
    setDocumentType(template.document_type)
    setBody(template.body_markdown)
    setTemplateInitialized(true)
  }, [templateInitialized, requestedTemplateId, templatesQuery.data?.templates])

  useEffect(() => {
    if (createdDocumentPath) navigate(createdDocumentPath)
  }, [createdDocumentPath, navigate])

  const hasUnsavedChanges =
    createdDocumentPath === null &&
    (title.trim().length > 0 ||
      body.trim().length > 0 ||
      (spaceChosen && spaceKey !== initialSpace) ||
      documentType !== 'page' ||
      slug.trim().length > 0 ||
      taskKey.trim().length > 0 ||
      phaseKey.trim().length > 0)

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (!canSave || createDocument.isPending) return
    createDocument.mutate(
      {
        spaceKey: namespace.ref ? namespace.query.data!.resource_key : selectedSpace!.key,
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
        onSuccess: (document) =>
          setCreatedDocumentPath(documentPath(document.id, searchParams.toString())),
      },
    )
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-5">
      <UnsavedChangesGuard when={hasUnsavedChanges} />
      <PageHeader
        title={'Новый документ'}
        context={
          <>
            <p className="mt-1 max-w-3xl text-sm text-text-muted">
              Черновик можно связать с задачей, фазой процесса и материалами проверки.
            </p>
          </>
        }
        actions={
          <>
            <Button
              type="button"
              size="sm"
              variant="secondary"
              onClick={() => setEditorTab('preview')}
            >
              <FileText className="h-4 w-4" />
              Предпросмотр
            </Button>
            <Button size="sm" disabled={createDocument.isPending || !canSave}>
              <Save className="h-4 w-4" />
              {createDocument.isPending ? 'Сохраняем...' : 'Сохранить черновик'}
            </Button>
          </>
        }
      />

      {createDocument.isError && (
        <ErrorState
          message={formatApiErrorForUser(createDocument.error, 'Не удалось сохранить документ')}
        />
      )}

      <section className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_20rem]">
        <Card className="min-w-0">
          <CardHeader>
            <CardTitle className="text-base">Редактор</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid gap-4 md:grid-cols-[1fr_12rem]">
              <FormField id="document-title" label={'Название'}>
                {(baseFieldAttrs) => (
                  <Input
                    {...baseFieldAttrs}
                    value={title}
                    disabled={createDocument.isPending}
                    onChange={(event) => setTitle(event.target.value)}
                    placeholder="Название документа"
                    required
                  />
                )}
              </FormField>
              <div className="space-y-1.5">
                <FormField id="document-space" label="Пространство" error={spaceProblem}>
                  {(field) =>
                    namespace.ref ? (
                      <Input
                        id={field.id}
                        aria-invalid={field['aria-invalid']}
                        aria-describedby={field['aria-describedby']}
                        value={namespace.query.data?.resource_key ?? ''}
                        readOnly
                        disabled
                      />
                    ) : (
                      <SearchableSelect
                        id={field.id}
                        aria-invalid={field['aria-invalid']}
                        aria-describedby={field['aria-describedby']}
                        label="Пространство"
                        value={spaceKey}
                        disabled={createDocument.isPending}
                        selectedLabel={
                          selectedSpace ? `${selectedSpace.name} (${selectedSpace.key})` : undefined
                        }
                        placeholder="Выберите пространство"
                        allowEmpty={false}
                        options={filteredSpaces
                          .slice(spaceOffset, spaceOffset + 50)
                          .map((space) => ({
                            value: space.key,
                            label: `${space.name} (${space.key})`,
                          }))}
                        search={spaceSearch}
                        onSearch={(value) => {
                          setSpaceSearch(value)
                          setSpaceOffset(0)
                        }}
                        offset={spaceOffset}
                        pageSize={50}
                        total={filteredSpaces.length}
                        onPage={setSpaceOffset}
                        onChange={(value) => {
                          setSpaceKey(value)
                          setSpaceChosen(true)
                        }}
                        loading={spacesQuery.isLoading}
                        error={
                          spacesQuery.isError ? 'Не удалось загрузить пространства' : undefined
                        }
                        onRetry={() => {
                          void spacesQuery.refetch()
                        }}
                      />
                    )
                  }
                </FormField>
                {namespace.ref && !canSave && (
                  <Notice role="status" appearance="inline" tone="neutral">
                    Привязка пространства не подтверждена.
                  </Notice>
                )}
                {!namespace.ref &&
                  !spacesQuery.isLoading &&
                  !spacesQuery.isError &&
                  !selectedSpace &&
                  !spaceProblem && (
                    <Notice role="status" appearance="inline" tone="neutral">
                      {availableSpaces.length
                        ? 'Выбранное пространство недоступно. Выберите доступное пространство.'
                        : 'Нет доступных пространств. Создайте пространство или получите доступ.'}{' '}
                      <Link to="/spaces">К пространствам</Link>
                    </Notice>
                  )}
              </div>
            </div>

            <div className="flex flex-wrap gap-2">
              {templatesQuery.isLoading && <LoadingState message="Загружаем шаблоны" />}
              {(templatesQuery.data?.templates ?? []).map((template) => (
                <Button
                  variant="outline"
                  key={template.id}
                  type="button"
                  disabled={createDocument.isPending}
                  size="sm"
                  onClick={() => applyTemplate(template.id)}
                >
                  {template.name}
                </Button>
              ))}
            </div>

            <Tabs
              value={editorTab}
              onValueChange={(value) => setEditorTab(value as 'write' | 'preview')}
            >
              <TabsList>
                <TabsTrigger value="write">Markdown</TabsTrigger>
                <TabsTrigger value="preview">Просмотр</TabsTrigger>
              </TabsList>
              <TabsContent value="write">
                <Textarea
                  className="min-h-96 font-mono text-sm"
                  aria-label="Markdown документа"
                  value={body}
                  disabled={createDocument.isPending}
                  onChange={(event) => {
                    setBody(event.target.value)
                    setTemplateInitialized(true)
                  }}
                  placeholder="Начните писать Markdown или примените шаблон выше"
                  required
                />
              </TabsContent>
              <TabsContent value="preview">
                <Card className="min-h-96">
                  <CardContent>
                    <Prose>
                      <h2>{title}</h2>
                      <Markdown
                        skipHtml
                        components={{
                          pre: ({ children }) => (
                            <CodeBlock tabIndex={0} wrap={false}>
                              {children}
                            </CodeBlock>
                          ),
                        }}
                      >
                        {body}
                      </Markdown>
                    </Prose>
                  </CardContent>
                </Card>
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
              <FormField id="document-task" label={'Задача'}>
                {(baseFieldAttrs) => (
                  <Input
                    {...baseFieldAttrs}
                    value={taskKey}
                    disabled={createDocument.isPending}
                    onChange={(event) => setTaskKey(event.target.value)}
                    placeholder="Ключ задачи"
                  />
                )}
              </FormField>
              <FormField id="document-phase" label={'Фаза'}>
                {(baseFieldAttrs) => (
                  <Input
                    {...baseFieldAttrs}
                    value={phaseKey}
                    disabled={createDocument.isPending}
                    onChange={(event) => setPhaseKey(event.target.value)}
                    placeholder="Ключ фазы"
                  />
                )}
              </FormField>
              {taskKey && (
                <Card asChild interactive>
                  <Link to={`/tasks/${taskKey}`}>
                    <CardContent className="flex items-center gap-2">
                      <FileText className="h-4 w-4 text-accent" />
                      Задача {taskKey}
                    </CardContent>
                  </Link>
                </Card>
              )}
              {phaseKey && (
                <Card asChild interactive>
                  <Link to={`/phases/${phaseKey}`}>
                    <CardContent className="flex items-center gap-2">
                      <GitBranch className="h-4 w-4 text-accent" />
                      Фаза {phaseKey}
                    </CardContent>
                  </Link>
                </Card>
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
                  disabled={createDocument.isPending}
                  onChange={(event) => {
                    setDocumentType(event.target.value)
                    setTemplateInitialized(true)
                  }}
                />
                <datalist id="document-type-options">
                  {typeOptions.map((type) => (
                    <option key={type} value={type}>
                      {formatDocumentType(type)}
                    </option>
                  ))}
                </datalist>
              </div>
              <FormField id="document-slug" label={'Адрес (slug)'}>
                {(baseFieldAttrs) => (
                  <Input
                    {...baseFieldAttrs}
                    value={slug}
                    disabled={createDocument.isPending}
                    onChange={(event) => setSlug(event.target.value)}
                    placeholder="создастся автоматически"
                  />
                )}
              </FormField>
              <Badge tone="info">
                <Tag aria-hidden="true" className="h-4 w-4" />
                Markdown, ревизии, связи
              </Badge>
            </CardContent>
          </Card>
        </div>
      </section>
    </form>
  )
}
