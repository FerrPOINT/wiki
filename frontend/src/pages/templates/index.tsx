import { FormEvent, useState } from 'react'
import { Link } from 'react-router'
import {
  CheckCircle2,
  ChevronDown,
  ClipboardCheck,
  FileText,
  Plus,
  Search,
  ShieldCheck,
  X,
} from 'lucide-react'
import { useCreateTemplate, useCurrentUser, useTemplates } from '@/shared/api/hooks'
import { Button, EmptyState, ErrorState, Input, Label, LoadingState, Textarea } from '@sdlc/ui/ui'
import { formatApiErrorForUser } from '@/shared/lib/api-error'
import { formatDocumentType } from '@/shared/lib/wiki-format'
import type { Template } from '@/api/wiki'

const typeOptions = [
  'page',
  'requirements',
  'research_note',
  'implementation_note',
  'test_plan',
  'release_note',
]
const pageSize = 12
const selectClassName =
  'min-h-10 w-full rounded-md border border-border-strong bg-surface px-3 text-sm text-text-primary focus-visible:outline-2 focus-visible:outline-accent'

function templateIcon(documentType: string) {
  if (documentType === 'requirements') return ClipboardCheck
  if (documentType === 'test_plan' || documentType === 'release_note') return ShieldCheck
  return FileText
}

export function TemplatesPage() {
  const templatesQuery = useTemplates()
  const currentUserQuery = useCurrentUser()
  const createTemplate = useCreateTemplate()
  const isSystemAdmin = currentUserQuery.data?.is_system_admin === true
  const templates = templatesQuery.data?.templates ?? []
  const [showCreate, setShowCreate] = useState(false)
  const [createdTemplate, setCreatedTemplate] = useState<Template | null>(null)
  const [name, setName] = useState('')
  const [documentType, setDocumentType] = useState('requirements')
  const [body, setBody] = useState('')
  const [search, setSearch] = useState('')
  const [typeFilter, setTypeFilter] = useState('all')
  const [page, setPage] = useState(1)
  const [expandedId, setExpandedId] = useState<string | null>(null)
  const normalizedSearch = search.trim().toLocaleLowerCase('ru')
  const filteredTemplates = templates
    .filter((template) => typeFilter === 'all' || template.document_type === typeFilter)
    .filter((template) =>
      normalizedSearch
        ? `${template.name} ${template.body_markdown}`
            .toLocaleLowerCase('ru')
            .includes(normalizedSearch)
        : true,
    )
    .sort((a, b) => a.name.localeCompare(b.name, 'ru'))
  const totalPages = Math.max(1, Math.ceil(filteredTemplates.length / pageSize))
  const currentPage = Math.min(page, totalPages)
  const visibleTemplates = filteredTemplates.slice(
    (currentPage - 1) * pageSize,
    currentPage * pageSize,
  )

  function handleCreateTemplate(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (!isSystemAdmin || createTemplate.isPending) return
    setCreatedTemplate(null)
    createTemplate.mutate(
      {
        name: name.trim(),
        document_type: documentType,
        body_markdown: body.trim(),
      },
      {
        onSuccess: (template) => {
          setName('')
          setDocumentType('requirements')
          setBody('')
          setShowCreate(false)
          setCreatedTemplate(template)
        },
      },
    )
  }

  return (
    <div className="space-y-5">
      <header className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold">Шаблоны</h1>
          <p className="mt-1 text-sm text-text-muted">Стартовые структуры документов.</p>
        </div>
        <div className="flex flex-wrap gap-2">
          {isSystemAdmin && (
            <Button
              type="button"
              size="sm"
              className="min-h-10 sm:min-h-10"
              aria-expanded={showCreate}
              aria-controls={showCreate ? 'template-create' : undefined}
              disabled={createTemplate.isPending}
              onClick={() => setShowCreate((value) => !value)}
            >
              <Plus className="h-4 w-4" aria-hidden />
              {showCreate ? 'Свернуть форму' : 'Новый шаблон'}
            </Button>
          )}
          <Button asChild size="sm" variant="secondary" className="min-h-10 sm:min-h-10">
            <Link to="/documents/new">
              <FileText className="h-4 w-4" aria-hidden />
              Создать документ
            </Link>
          </Button>
        </div>
      </header>

      {createdTemplate && (
        <div
          role="status"
          className="flex flex-wrap items-center gap-2 border-y border-border py-3 text-sm text-text-secondary"
        >
          <CheckCircle2 className="h-4 w-4 text-success" aria-hidden />
          <span className="mr-auto min-w-0 break-words">
            Шаблон «{createdTemplate.name}» создан
          </span>
          <Button asChild size="sm" variant="outline">
            <Link to={`/documents/new?template=${encodeURIComponent(createdTemplate.id)}`}>
              Использовать
            </Link>
          </Button>
          <Button
            type="button"
            size="icon"
            variant="ghost"
            aria-label="Закрыть уведомление"
            title="Закрыть уведомление"
            onClick={() => setCreatedTemplate(null)}
          >
            <X className="h-4 w-4" aria-hidden />
          </Button>
        </div>
      )}

      {isSystemAdmin && showCreate && (
        <form
          id="template-create"
          aria-label="Создание шаблона"
          onSubmit={handleCreateTemplate}
          className="space-y-3 border-y border-border py-4"
        >
          <div className="grid gap-3 lg:grid-cols-[minmax(0,1fr)_14rem]">
            <div className="space-y-1.5">
              <Label htmlFor="template-name">Название шаблона</Label>
              <Input
                id="template-name"
                className="min-h-10"
                value={name}
                onChange={(event) => setName(event.target.value)}
                disabled={createTemplate.isPending}
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
                disabled={createTemplate.isPending}
              >
                {typeOptions.map((type) => (
                  <option key={type} value={type}>
                    {formatDocumentType(type)}
                  </option>
                ))}
              </select>
            </div>
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="template-body">Markdown шаблона</Label>
            <Textarea
              id="template-body"
              className="min-h-32 font-mono text-sm"
              value={body}
              onChange={(event) => setBody(event.target.value)}
              disabled={createTemplate.isPending}
              required
            />
          </div>
          {createTemplate.isError && (
            <p role="alert" className="text-sm text-danger">
              {formatApiErrorForUser(createTemplate.error, 'Не удалось создать шаблон')}
            </p>
          )}
          <div className="flex flex-wrap justify-end gap-2">
            <Button
              type="button"
              size="sm"
              variant="outline"
              className="min-h-10 sm:min-h-10"
              disabled={createTemplate.isPending}
              onClick={() => setShowCreate(false)}
            >
              Свернуть
            </Button>
            <Button
              type="submit"
              size="sm"
              className="min-h-10 sm:min-h-10"
              disabled={createTemplate.isPending}
            >
              {createTemplate.isPending ? 'Создаём...' : 'Создать шаблон'}
            </Button>
          </div>
        </form>
      )}

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
        <section aria-label="Список шаблонов" className="space-y-3">
          <div className="grid gap-2 sm:grid-cols-[minmax(0,1fr)_14rem]">
            <div className="relative min-w-0">
              <Search
                className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-text-muted"
                aria-hidden
              />
              <Input
                type="search"
                aria-label="Найти шаблон"
                placeholder="Название или содержимое"
                className="min-h-10 pl-9"
                value={search}
                onChange={(event) => {
                  setSearch(event.target.value)
                  setPage(1)
                  setExpandedId(null)
                }}
              />
            </div>
            <select
              aria-label="Тип шаблона"
              className={selectClassName}
              value={typeFilter}
              onChange={(event) => {
                setTypeFilter(event.target.value)
                setPage(1)
                setExpandedId(null)
              }}
            >
              <option value="all">Все типы</option>
              {typeOptions.map((type) => (
                <option key={type} value={type}>
                  {formatDocumentType(type)}
                </option>
              ))}
            </select>
          </div>
          <p className="text-xs text-text-muted">
            Показано {visibleTemplates.length} из {filteredTemplates.length} шаблонов
          </p>
          {filteredTemplates.length === 0 ? (
            <p role="status" className="border-y border-border py-6 text-sm text-text-muted">
              По заданным условиям шаблоны не найдены.
            </p>
          ) : (
            <ul className="divide-y divide-border border-y border-border">
              {visibleTemplates.map((template) => {
                const Icon = templateIcon(template.document_type)
                const expanded = expandedId === template.id
                return (
                  <li key={template.id}>
                    <div className="flex min-w-0 items-center gap-2">
                      <button
                        type="button"
                        aria-expanded={expanded}
                        aria-controls={`template-body-${template.id}`}
                        aria-label={`${expanded ? 'Скрыть' : 'Показать'} содержимое шаблона ${template.name}`}
                        className="flex min-h-14 min-w-0 flex-1 items-center gap-3 py-2 text-left hover:bg-surface-raised focus-visible:outline-2 focus-visible:outline-accent"
                        onClick={() => setExpandedId(expanded ? null : template.id)}
                      >
                        <Icon className="h-4 w-4 shrink-0 text-accent" aria-hidden />
                        <span className="min-w-0 flex-1">
                          <span className="block truncate text-sm font-medium text-text-primary">
                            {template.name}
                          </span>
                          <span className="block truncate text-xs text-text-muted">
                            {formatDocumentType(template.document_type)}
                          </span>
                        </span>
                        <ChevronDown
                          className={`h-4 w-4 shrink-0 text-text-muted transition-transform ${expanded ? 'rotate-180' : ''}`}
                          aria-hidden
                        />
                      </button>
                      <Link
                        to={`/documents/new?template=${encodeURIComponent(template.id)}`}
                        aria-label={`Использовать шаблон ${template.name}`}
                        className="inline-flex min-h-10 shrink-0 items-center px-2 text-sm text-accent hover:underline"
                      >
                        Использовать
                      </Link>
                    </div>
                    {expanded && (
                      <pre
                        id={`template-body-${template.id}`}
                        className="whitespace-pre-wrap break-words border-t border-border bg-surface-raised p-3 font-mono text-xs text-text-secondary"
                      >
                        {template.body_markdown}
                      </pre>
                    )}
                  </li>
                )
              })}
            </ul>
          )}
          {filteredTemplates.length > pageSize && (
            <nav aria-label="Страницы шаблонов" className="flex items-center justify-end gap-2">
              <Button
                type="button"
                size="sm"
                variant="outline"
                className="min-h-10 sm:min-h-10"
                disabled={currentPage === 1}
                onClick={() => {
                  setPage(currentPage - 1)
                  setExpandedId(null)
                }}
              >
                Назад
              </Button>
              <span className="text-sm text-text-muted">
                {currentPage} / {totalPages}
              </span>
              <Button
                type="button"
                size="sm"
                variant="outline"
                className="min-h-10 sm:min-h-10"
                disabled={currentPage === totalPages}
                onClick={() => {
                  setPage(currentPage + 1)
                  setExpandedId(null)
                }}
              >
                Далее
              </Button>
            </nav>
          )}
        </section>
      )}
    </div>
  )
}
