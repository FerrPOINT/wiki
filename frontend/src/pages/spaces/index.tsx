import { useApplicationAccess } from '@pdlc/ui/auth'
import { useActiveSpaces } from '@/widgets/active-spaces'
import { FormEvent, type RefObject, useRef, useState } from 'react'
// Spaces own documents independently of their project associations.
import { Link } from 'react-router'

import { Archive, BookOpenText, ChevronDown, FileText, FolderOpen, Plus, Save } from 'lucide-react'
import {
  useArchiveSpace,
  useCreateSpace,
  useCurrentUser,
  useSpaceMembers,
  useSpaceTree,
  useUpdateSpace,
  useUsers,
} from '@/shared/api/hooks'
import {
  PageNavigation,
  CardContent,
  Card,
  PageHeader,
  Select,
  ConfirmDialog,
  EmptyState,
  ErrorState,
  LoadingState,
  ResourceList,
  ResourceListRow,
  Checkbox,
  Button,
  Input,
  Textarea,
  SearchInput,
  Notice,
  FormField,
} from '@pdlc/ui/ui'

import { formatApiErrorForUser } from '@/shared/lib/api-error'
import { formatDateTime, formatDocumentType, shortText } from '@/shared/lib/wiki-format'
import type { Space, SpaceTreeNode } from '@/api/wiki'
import { DossierMenu } from '@/widgets/dossier-menu'
import { SpaceProjectsEditor } from '@/features/space-projects/SpaceProjectsEditor'

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

function nullableText(value: string): string | null {
  const trimmed = value.trim()
  return trimmed.length > 0 ? trimmed : null
}

function statusLabel(status: string): string {
  return status === 'archived' ? 'архивировано' : 'активно'
}

function SpaceTreePreview({ spaceKey }: { spaceKey: string }) {
  const treeQuery = useSpaceTree(spaceKey)
  const documents = flattenTree(treeQuery.data?.documents ?? [])

  if (treeQuery.isLoading) return <LoadingState message="Загружаем дерево" />
  if (treeQuery.isError) {
    return (
      <ErrorState
        message={formatApiErrorForUser(treeQuery.error, 'Не удалось загрузить дерево')}
        onRetry={() => treeQuery.refetch()}
      />
    )
  }
  if (documents.length === 0) return <EmptyState message="В пространстве пока нет документов" />

  return (
    <div className="space-y-1.5">
      {documents.map((document) => (
        <Link
          key={document.id}
          to={`/documents/${document.id}`}
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

function CreateSpaceForm({ onCreated }: { onCreated: () => void }) {
  const createSpace = useCreateSpace()
  const [key, setKey] = useState('')
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    createSpace.mutate(
      {
        key: key.trim(),
        name: name.trim(),
        description: nullableText(description),
      },
      {
        onSuccess: () => {
          setKey('')
          setName('')
          setDescription('')
          onCreated()
        },
      },
    )
  }

  return (
    <Card asChild>
      <form onSubmit={handleSubmit}>
        <CardContent>
          <div className="grid gap-3 lg:grid-cols-[10rem_1fr_1.5fr_auto] lg:items-end">
            <FormField id="space-key" label={'Ключ'}>
              {(baseFieldAttrs) => (
                <Input
                  {...baseFieldAttrs}
                  value={key}
                  onChange={(event) => setKey(event.target.value)}
                  placeholder="KEY"
                />
              )}
            </FormField>
            <FormField id="space-name" label={'Название'}>
              {(baseFieldAttrs) => (
                <Input
                  {...baseFieldAttrs}
                  value={name}
                  onChange={(event) => setName(event.target.value)}
                  placeholder="Название пространства"
                  required
                />
              )}
            </FormField>
            <FormField id="space-description" label={'Описание'}>
              {(baseFieldAttrs) => (
                <Input
                  {...baseFieldAttrs}
                  value={description}
                  onChange={(event) => setDescription(event.target.value)}
                  placeholder="Описание пространства"
                />
              )}
            </FormField>
            <Button disabled={createSpace.isPending || !key.trim() || !name.trim()}>
              <Plus className="h-4 w-4" />
              {createSpace.isPending ? 'Создаём...' : 'Создать пространство'}
            </Button>
          </div>
          {createSpace.isError && (
            <Notice role="alert" appearance="inline" tone="danger" className="mt-2">
              {formatApiErrorForUser(createSpace.error, 'Не удалось создать пространство')}
            </Notice>
          )}
        </CardContent>
      </form>
    </Card>
  )
}

function SpaceDetails({
  fallbackFocusRef,
  isSystemAdmin,
  onArchived,
  space,
}: {
  currentUserId?: string
  fallbackFocusRef: RefObject<HTMLButtonElement | null>
  isSystemAdmin: boolean
  onArchived: (space: Space) => void
  space: Space
}) {
  const membersQuery = useSpaceMembers(space.key, !isSystemAdmin)
  const updateSpace = useUpdateSpace()
  const archiveSpace = useArchiveSpace()
  const canManage = space.status !== 'archived' && isSystemAdmin
  const [name, setName] = useState(space.name)
  const [description, setDescription] = useState(space.description ?? '')
  const [archiveOpen, setArchiveOpen] = useState(false)
  const changed =
    name.trim() !== space.name || nullableText(description) !== (space.description ?? null)

  function handleUpdate(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    updateSpace.mutate({
      spaceKey: space.key,
      body: {
        name: name.trim(),
        description: nullableText(description),
      },
    })
  }

  return (
    <div className="space-y-4 border-t border-border bg-surface-raised/40 p-3">
      {canManage && (
        <Card asChild>
          <form onSubmit={handleUpdate}>
            <CardContent className="space-y-3">
              <div className="grid gap-3 md:grid-cols-2">
                <FormField id={`${space.key}-name`} label={'Название пространства'}>
                  {(baseFieldAttrs) => (
                    <Input
                      {...baseFieldAttrs}
                      value={name}
                      onChange={(event) => setName(event.target.value)}
                      required
                    />
                  )}
                </FormField>
                <FormField id={`${space.key}-description`} label={'Описание'}>
                  {(baseFieldAttrs) => (
                    <Textarea
                      {...baseFieldAttrs}
                      value={description}
                      onChange={(event) => setDescription(event.target.value)}
                    />
                  )}
                </FormField>
              </div>
              <div className="flex flex-wrap gap-2">
                <Button size="sm" disabled={updateSpace.isPending || !changed || !name.trim()}>
                  <Save className="h-3.5 w-3.5" />
                  {updateSpace.isPending ? 'Сохраняем...' : 'Сохранить'}
                </Button>
                <Button
                  type="button"
                  size="sm"
                  variant="destructive"
                  onClick={() => {
                    archiveSpace.reset()
                    setArchiveOpen(true)
                  }}
                  disabled={archiveSpace.isPending}
                >
                  <Archive className="h-3.5 w-3.5" />
                  Архивировать
                </Button>
              </div>
              {updateSpace.isError && (
                <Notice role="alert" appearance="inline" tone="danger">
                  {formatApiErrorForUser(updateSpace.error, 'Не удалось обновить пространство')}
                </Notice>
              )}
            </CardContent>
          </form>
        </Card>
      )}

      <SpaceProjectsEditor spaceKey={space.key} canManage={Boolean(canManage)} />

      <ConfirmDialog
        open={archiveOpen}
        fallbackFocusRef={fallbackFocusRef}
        onOpenChange={setArchiveOpen}
        title="Архивировать пространство?"
        description={`«${space.name}» (${space.key}) останется доступным для чтения, но изменения документов, материалов и связей будут заблокированы. Восстановление из интерфейса пока недоступно.`}
        onConfirm={() => {
          if (!canManage || archiveSpace.isPending) return
          archiveSpace.mutate(space.key, {
            onSuccess: (archived) => {
              setArchiveOpen(false)
              onArchived(archived)
            },
          })
        }}
        isPending={archiveSpace.isPending}
        error={
          archiveSpace.error
            ? formatApiErrorForUser(archiveSpace.error, 'Не удалось архивировать пространство')
            : null
        }
      />

      <Card>
        <CardContent className="space-y-2">
          <DossierMenu spaceKey={space.key} />
          <div className="flex items-center gap-2 text-xs font-medium uppercase text-text-muted">
            <BookOpenText className="h-3.5 w-3.5" />
            Дерево
          </div>
          <SpaceTreePreview spaceKey={space.key} />
        </CardContent>
      </Card>
      {!isSystemAdmin && membersQuery.isError && (
        <ErrorState
          message={formatApiErrorForUser(
            membersQuery.error,
            'Не удалось проверить право на изменение пространства',
          )}
          onRetry={() => membersQuery.refetch()}
        />
      )}
    </div>
  )
}

export function SpacesPage() {
  const expandedSpaceRef = useRef<HTMLButtonElement>(null)
  const currentUserQuery = useCurrentUser()
  const [status, setStatus] = useState('active')
  const spacesQuery = useActiveSpaces(status !== 'active')
  const isSystemAdmin = useApplicationAccess()?.permits([], true) === true
  const usersQuery = useUsers(isSystemAdmin)
  const users = usersQuery.data?.users ?? []
  const [recentlyArchived, setRecentlyArchived] = useState<Space | null>(null)
  const spaces = (spacesQuery.data?.spaces ?? []).map((space) =>
    space.key === recentlyArchived?.key ? recentlyArchived : space,
  )
  const [showCreate, setShowCreate] = useState(false)
  const [search, setSearch] = useState('')
  const [sort, setSort] = useState('updated')
  const [mineOnly, setMineOnly] = useState(false)
  const [page, setPage] = useState(1)
  const [expandedKey, setExpandedKey] = useState<string | null>(null)
  const query = search.trim().toLocaleLowerCase()
  const filteredSpaces = spaces
    .filter((space) => status === 'all' || space.status === status)
    .filter((space) => !mineOnly || space.owner_id === currentUserQuery.data?.id)
    .filter(
      (space) =>
        !query ||
        [space.key, space.name, space.description ?? ''].some((value) =>
          value.toLocaleLowerCase().includes(query),
        ),
    )
    .sort((a, b) => {
      if (sort === 'updated' && recentlyArchived) {
        if (a.key === recentlyArchived.key) return -1
        if (b.key === recentlyArchived.key) return 1
      }
      return sort === 'name'
        ? a.name.localeCompare(b.name, 'ru')
        : new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()
    })
  const pageCount = Math.max(1, Math.ceil(filteredSpaces.length / 12))
  const currentPage = Math.min(page, pageCount)
  const visibleSpaces = filteredSpaces.slice((currentPage - 1) * 12, currentPage * 12)

  function resetListPosition() {
    setPage(1)
    setExpandedKey(null)
  }

  function handleArchived(space: Space) {
    setRecentlyArchived(space)
    setStatus('all')
    setSort('updated')
    setPage(1)
    setExpandedKey(space.key)
  }

  return (
    <div className="space-y-5">
      <PageHeader
        title={'Пространства'}
        context={
          <>
            <p className="max-w-3xl text-sm text-text-muted">
              Пространства группируют документы по продуктам, командам и контекстам процесса.
            </p>
          </>
        }
        actions={
          <>
            {isSystemAdmin && spacesQuery.unfilteredCount > 0 && (
              <Button
                type="button"
                size="sm"
                variant="outline"

                aria-expanded={showCreate}
                onClick={() => setShowCreate((value) => !value)}
              >
                <Plus className="h-4 w-4" /> Создать пространство
              </Button>
            )}
            {spaces.some((space) => space.status === 'active') && (
              <Button asChild size="sm" className="md:hidden">
                <Link to="/documents/new">Новый документ</Link>
              </Button>
            )}
          </>
        }
      />

      {isSystemAdmin && showCreate && <CreateSpaceForm onCreated={() => setShowCreate(false)} />}

      {recentlyArchived && (
        <Notice role="status" appearance="inline" tone="success">
          Пространство «{recentlyArchived.name}» архивировано и остаётся доступным для чтения.
        </Notice>
      )}

      {spacesQuery.isLoading && <LoadingState message="Загружаем пространства" />}
      {spacesQuery.isError && (
        <ErrorState
          message={formatApiErrorForUser(spacesQuery.error, 'Не удалось загрузить пространства')}
          onRetry={() => spacesQuery.refetch()}
        />
      )}
      {!spacesQuery.isLoading && !spacesQuery.isError && spacesQuery.unfilteredCount === 0 && (
        <EmptyState
          message="Пространства ещё не созданы"
          action={
            isSystemAdmin ? (
              <Button
                type="button"
                size="sm"
                variant="secondary"
                onClick={() => setShowCreate(true)}
              >
                Создать пространство
              </Button>
            ) : (
              <p className="text-sm text-text-muted">
                Получите доступ к пространству у администратора.
              </p>
            )
          }
        />
      )}
      {!spacesQuery.isLoading && !spacesQuery.isError && spacesQuery.unfilteredCount > 0 && (
        <section aria-label="Список пространств" className="space-y-3">
          <div className="grid gap-2 sm:grid-cols-[minmax(0,1fr)_11rem_11rem_auto]">
            <div className="min-w-0">
              <SearchInput
                aria-label="Найти пространство"
                placeholder="Название, ключ или описание"
                value={search}
                onChange={(event) => {
                  setSearch(event.target.value)
                  resetListPosition()
                }}
              />
            </div>
            <Select
              aria-label="Статус пространства"
              className="w-full"
              value={status}
              onValueChange={(nextValue) => {
                setStatus(nextValue)
                resetListPosition()
              }}
              options={[
                { value: 'all', label: 'Все статусы' },
                { value: 'active', label: 'Активные' },
                { value: 'archived', label: 'Архивные' },
              ]}
            />
            <Select
              aria-label="Сортировка пространств"
              className="w-full"
              value={sort}
              onValueChange={(nextValue) => {
                setSort(nextValue)
                resetListPosition()
              }}
              options={[
                { value: 'updated', label: 'Сначала обновлённые' },
                { value: 'name', label: 'По названию' },
              ]}
            />
            {currentUserQuery.data && (
              <Checkbox
                className="whitespace-nowrap text-text-secondary"
                checked={mineOnly}
                onCheckedChange={(checked) => {
                  setMineOnly(checked === true)
                  resetListPosition()
                }}
              >
                Только мои
              </Checkbox>
            )}
          </div>
          <p className="text-xs text-text-muted">
            Показано {visibleSpaces.length} из {filteredSpaces.length} пространств
          </p>
          {filteredSpaces.length === 0 ? (
            <div role="status">
              <EmptyState message="По заданным условиям пространства не найдены." />
            </div>
          ) : (
            <ResourceList asChild>
              <ul>
                {visibleSpaces.map((space) => {
                  const expanded = expandedKey === space.key
                  const owner = users.find((user) => user.id === space.owner_id)
                  return (
                    <li key={space.key}>
                      <ResourceListRow asChild interactive>
                        <Button
                          variant="ghost"
                          ref={expanded ? expandedSpaceRef : undefined}
                          type="button"
                          aria-expanded={expanded}
                          aria-controls={`space-details-${space.key}`}
                          onClick={() => setExpandedKey(expanded ? null : space.key)}
                          className="flex w-full min-w-0 items-start gap-3 text-left transition-colors"
                        >
                          <FolderOpen className="mt-0.5 h-4 w-4 shrink-0 text-accent" aria-hidden />
                          <span className="min-w-0 flex-1 space-y-1">
                            <span className="flex min-w-0 items-center gap-2">
                              <span className="truncate text-sm font-semibold">{space.name}</span>
                              <span className="shrink-0 text-xs text-text-muted">{space.key}</span>
                              {space.status === 'archived' && (
                                <span className="shrink-0 text-xs text-warning">
                                  {statusLabel(space.status)}
                                </span>
                              )}
                            </span>
                            <span className="block truncate text-xs text-text-secondary">
                              {shortText(space.description)}
                            </span>
                            <span className="flex flex-wrap gap-x-3 gap-y-1 text-xs text-text-muted">
                              <span className="inline-flex items-center gap-1">
                                <FileText className="h-3.5 w-3.5" aria-hidden />
                                {space.document_count} документов
                              </span>
                              {owner && (
                                <span>Владелец: {owner.display_name ?? owner.username}</span>
                              )}
                              <span>Обновлено {formatDateTime(space.updated_at)}</span>
                            </span>
                          </span>
                          <ChevronDown
                            className={`mt-1 h-4 w-4 shrink-0 text-text-muted transition-transform ${expanded ? 'rotate-180' : ''}`}
                            aria-hidden
                          />
                        </Button>
                      </ResourceListRow>
                      {expanded && (
                        <div id={`space-details-${space.key}`}>
                          <SpaceDetails
                            currentUserId={currentUserQuery.data?.id}
                            fallbackFocusRef={expandedSpaceRef}
                            isSystemAdmin={isSystemAdmin}
                            onArchived={handleArchived}
                            space={space}
                          />
                        </div>
                      )}
                    </li>
                  )
                })}
              </ul>
            </ResourceList>
          )}
          {filteredSpaces.length > 12 && (
            <PageNavigation
              page={currentPage}
              pageCount={pageCount}
              onPageChange={(nextPage) => {
                setPage(nextPage)
                setExpandedKey(null)
              }}
              label="Страницы пространств"
              previousLabel="Назад"
              nextLabel="Далее"
            />
          )}
        </section>
      )}
    </div>
  )
}
