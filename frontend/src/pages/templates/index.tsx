import { useApplicationAccess } from '@pdlc/ui/auth'
import { FormEvent, useEffect, useRef, useState } from 'react'
import {
  ActionGroup,
  PageNavigation,
  PageHeader,
  Select,
  CodeBlock,
  NamespaceLink as Link,
  ResourceList,
  ResourceListRow,
  Button,
  EmptyState,
  ErrorState,
  Input,
  FormField,
  LoadingState,
  Textarea,
  SearchInput,
  Notice,
} from '@pdlc/ui/ui'
import { useSearchParams } from 'react-router'
import { ChevronDown, ClipboardCheck, FileText, Plus, ShieldCheck, X } from 'lucide-react'
import { useCreateTemplate, useTemplates } from '@/shared/api/hooks'

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
const searchDebounceMs = 300

function templateIcon(documentType: string) {
  if (documentType === 'requirements') return ClipboardCheck
  if (documentType === 'test_plan' || documentType === 'release_note') return ShieldCheck
  return FileText
}

function normalized(value: string | null): string {
  return value?.trim() ?? ''
}

function parsePage(value: string | null): number {
  if (!value || !/^[1-9]\d*$/.test(value)) return 1
  const parsed = Number(value)
  return Number.isSafeInteger(parsed) ? parsed : 1
}

function setOptionalParam(params: URLSearchParams, name: string, value: string): void {
  if (value) params.set(name, value)
  else params.delete(name)
}

function useTemplateCatalogUrl() {
  const [searchParams, setSearchParams] = useSearchParams()
  const appliedSearch = normalized(searchParams.get('q'))
  const requestedType = normalized(searchParams.get('type'))
  const typeFilter = typeOptions.includes(requestedType) ? requestedType : 'all'
  const requestedPage = parsePage(searchParams.get('page'))
  const [search, setSearch] = useState(appliedSearch)

  useEffect(() => {
    setSearch(appliedSearch)
  }, [appliedSearch])

  useEffect(() => {
    const canonicalType = typeFilter === 'all' ? '' : typeFilter
    const canonicalPage = requestedPage === 1 ? '' : String(requestedPage)
    const shouldCanonicalize =
      searchParams.getAll('q').length !== (appliedSearch ? 1 : 0) ||
      searchParams.get('q') !== (appliedSearch || null) ||
      searchParams.getAll('type').length !== (canonicalType ? 1 : 0) ||
      searchParams.get('type') !== (canonicalType || null) ||
      searchParams.getAll('page').length !== (canonicalPage ? 1 : 0) ||
      searchParams.get('page') !== (canonicalPage || null)
    if (!shouldCanonicalize) return

    setSearchParams(
      (current) => {
        const next = new URLSearchParams(current)
        setOptionalParam(next, 'q', appliedSearch)
        setOptionalParam(next, 'type', canonicalType)
        setOptionalParam(next, 'page', canonicalPage)
        return next
      },
      { replace: true },
    )
  }, [appliedSearch, requestedPage, searchParams, setSearchParams, typeFilter])

  useEffect(() => {
    const normalizedSearch = search.trim()
    if (normalizedSearch === appliedSearch) return
    const timeout = window.setTimeout(() => {
      setSearchParams(
        (current) => {
          const next = new URLSearchParams(current)
          setOptionalParam(next, 'q', normalizedSearch)
          next.delete('page')
          return next
        },
        { replace: true },
      )
    }, searchDebounceMs)
    return () => window.clearTimeout(timeout)
  }, [appliedSearch, search, setSearchParams])

  function setTypeFilter(value: string) {
    const nextType = typeOptions.includes(value) ? value : 'all'
    setSearchParams((current) => {
      const next = new URLSearchParams(current)
      setOptionalParam(next, 'type', nextType === 'all' ? '' : nextType)
      next.delete('page')
      return next
    })
  }

  function setCatalogPage(nextPage: number, replace = false) {
    setSearchParams(
      (current) => {
        const next = new URLSearchParams(current)
        setOptionalParam(next, 'page', nextPage > 1 ? String(nextPage) : '')
        return next
      },
      { replace },
    )
  }

  return {
    requestedPage,
    search,
    setCatalogPage,
    setSearch,
    setTypeFilter,
    typeFilter,
  }
}

export function TemplatesPage() {
  const templatesQuery = useTemplates()
  const createTemplate = useCreateTemplate()
  const isSystemAdmin = useApplicationAccess()?.permits([], true) === true
  const templates = templatesQuery.data?.templates ?? []
  const [showCreate, setShowCreate] = useState(false)
  const [createdTemplate, setCreatedTemplate] = useState<Template | null>(null)
  const [submitAttempted, setSubmitAttempted] = useState(false)
  const [name, setName] = useState('')
  const [documentType, setDocumentType] = useState('requirements')
  const [body, setBody] = useState('')
  const { requestedPage, search, setCatalogPage, setSearch, setTypeFilter, typeFilter } =
    useTemplateCatalogUrl()
  const [expandedId, setExpandedId] = useState<string | null>(null)
  const nameRef = useRef<HTMLInputElement>(null)
  const bodyRef = useRef<HTMLTextAreaElement>(null)
  const nameInvalid = submitAttempted && !name.trim()
  const bodyInvalid = submitAttempted && !body.trim()
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
  const currentPage = Math.min(requestedPage, totalPages)
  const visibleTemplates = filteredTemplates.slice(
    (currentPage - 1) * pageSize,
    currentPage * pageSize,
  )

  useEffect(() => {
    if (templatesQuery.isLoading || templatesQuery.isError || requestedPage === currentPage) return
    setCatalogPage(currentPage, true)
  }, [currentPage, requestedPage, setCatalogPage, templatesQuery.isError, templatesQuery.isLoading])

  function handleCreateTemplate(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (!isSystemAdmin || createTemplate.isPending) return
    setSubmitAttempted(true)
    const normalizedName = name.trim()
    const normalizedBody = body.trim()
    if (!normalizedName || !normalizedBody) {
      if (!normalizedName) nameRef.current?.focus()
      else bodyRef.current?.focus()
      return
    }
    setCreatedTemplate(null)
    createTemplate.mutate(
      {
        name: normalizedName,
        document_type: documentType,
        body_markdown: normalizedBody,
      },
      {
        onSuccess: (template) => {
          setName('')
          setDocumentType('requirements')
          setBody('')
          setShowCreate(false)
          setSubmitAttempted(false)
          setCreatedTemplate(template)
        },
      },
    )
  }

  return (
    <div className="space-y-5">
      <PageHeader
        title={'Шаблоны'}
        context={
          <>
            <p className="mt-1 text-sm text-text-muted">Стартовые структуры документов.</p>
          </>
        }
        actions={
          <>
            {isSystemAdmin && (
              <Button
                type="button"
                size="sm"

                aria-expanded={showCreate}
                aria-controls={showCreate ? 'template-create' : undefined}
                disabled={createTemplate.isPending}
                onClick={() => {
                  if (!showCreate) {
                    createTemplate.reset()
                    setSubmitAttempted(false)
                  }
                  setShowCreate((value) => !value)
                }}
              >
                <Plus className="h-4 w-4" aria-hidden />
                {showCreate ? 'Свернуть форму' : 'Новый шаблон'}
              </Button>
            )}
            <Button asChild size="sm" variant="secondary" className="md:hidden">
              <Link to="/documents/new">
                <FileText className="h-4 w-4" aria-hidden />
                Создать документ
              </Link>
            </Button>
          </>
        }
      />

      {createdTemplate && (
        <Notice
          role="status"
          tone="success"
          actions={
            <>
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
            </>
          }
        >
          Шаблон «{createdTemplate.name}» создан
        </Notice>
      )}

      {isSystemAdmin && showCreate && (
        <form
          id="template-create"
          aria-label="Создание шаблона"
          onSubmit={handleCreateTemplate}
          noValidate
          className="space-y-3 border-y border-border py-4"
        >
          <div className="grid gap-3 lg:grid-cols-[minmax(0,1fr)_14rem]">
            <FormField
              id="template-name"
              label="Название шаблона"
              required
              error={nameInvalid ? 'Введите название шаблона.' : undefined}
            >
              {(attrs) => (
                <Input
                  {...attrs}
                  ref={nameRef}

                  value={name}
                  onChange={(event) => setName(event.target.value)}
                  disabled={createTemplate.isPending}
                />
              )}
            </FormField>
            <FormField id="template-type" label="Тип документа">
              {(attrs) => (
                <Select
                  {...attrs}
                  className="w-full"
                  value={documentType}
                  onValueChange={(nextValue) => setDocumentType(nextValue)}
                  disabled={createTemplate.isPending}
                  options={[
                    ...(typeOptions.map((type) => ({
                      value: String(type),
                      label: String(formatDocumentType(type)),
                    })) ?? []),
                  ]}
                />
              )}
            </FormField>
          </div>
          <FormField
            id="template-body"
            label="Markdown шаблона"
            required
            error={bodyInvalid ? 'Введите содержимое шаблона.' : undefined}
          >
            {(attrs) => (
              <Textarea
                {...attrs}
                ref={bodyRef}
                className="min-h-32 font-mono text-sm"
                value={body}
                onChange={(event) => setBody(event.target.value)}
                disabled={createTemplate.isPending}
              />
            )}
          </FormField>
          {createTemplate.isError && (
            <Notice role="alert" appearance="inline" tone="danger">
              {formatApiErrorForUser(createTemplate.error, 'Не удалось создать шаблон')}
            </Notice>
          )}
          <ActionGroup align="end">
            <Button
              type="button"
              size="sm"
              variant="outline"

              disabled={createTemplate.isPending}
              onClick={() => setShowCreate(false)}
            >
              Свернуть
            </Button>
            <Button
              type="submit"
              size="sm"

              disabled={createTemplate.isPending}
            >
              {createTemplate.isPending ? 'Создаём...' : 'Создать шаблон'}
            </Button>
          </ActionGroup>
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
            <div className="min-w-0">
              <SearchInput
                aria-label="Найти шаблон"
                placeholder="Название или содержимое"
                value={search}
                onChange={(event) => {
                  setSearch(event.target.value)
                  setExpandedId(null)
                }}
              />
            </div>
            <Select
              aria-label="Тип шаблона"
              className="w-full"
              value={typeFilter}
              onValueChange={(nextValue) => {
                setTypeFilter(nextValue)
                setExpandedId(null)
              }}
              options={[
                { value: 'all', label: 'Все типы' },
                ...(typeOptions.map((type) => ({
                  value: String(type),
                  label: String(formatDocumentType(type)),
                })) ?? []),
              ]}
            />
          </div>
          <p className="text-xs text-text-muted">
            Показано {visibleTemplates.length} из {filteredTemplates.length} шаблонов
          </p>
          {filteredTemplates.length === 0 ? (
            <div role="status">
              <EmptyState message="По заданным условиям шаблоны не найдены." />
            </div>
          ) : (
            <ResourceList asChild>
              <ul>
                {visibleTemplates.map((template) => {
                  const Icon = templateIcon(template.document_type)
                  const expanded = expandedId === template.id
                  return (
                    <li key={template.id}>
                      <ResourceListRow asChild>
                        <div className="flex min-w-0 items-center gap-2">
                          <Button
                            variant="ghost"
                            type="button"
                            aria-expanded={expanded}
                            aria-controls={`template-body-${template.id}`}
                            aria-label={`${expanded ? 'Скрыть' : 'Показать'} содержимое шаблона ${template.name}`}
                            className="min-w-0 flex-1 gap-3 text-left"
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
                          </Button>
                          <Button asChild variant="link" className="shrink-0">
                            <Link
                              to={`/documents/new?template=${encodeURIComponent(template.id)}`}
                              aria-label={`Использовать шаблон ${template.name}`}
                            >
                              Использовать
                            </Link>
                          </Button>
                        </div>
                      </ResourceListRow>
                      {expanded && (
                        <CodeBlock id={`template-body-${template.id}`} aria-label={template.name}>
                          {template.body_markdown}
                        </CodeBlock>
                      )}
                    </li>
                  )
                })}
              </ul>
            </ResourceList>
          )}
          {filteredTemplates.length > pageSize && (
            <PageNavigation
              page={currentPage}
              pageCount={totalPages}
              onPageChange={(nextPage) => {
                setCatalogPage(nextPage)
                setExpandedId(null)
              }}
              label="Страницы шаблонов"
              previousLabel="Назад"
              nextLabel="Далее"
            />
          )}
        </section>
      )}
    </div>
  )
}
