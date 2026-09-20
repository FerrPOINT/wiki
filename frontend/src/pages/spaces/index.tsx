import { FormEvent, useState } from 'react'
import { Link } from 'react-router'
import {
  Archive,
  BookOpenText,
  ChevronDown,
  FileText,
  FolderOpen,
  Plus,
  Save,
  Search,
  UserMinus,
  UserPlus,
  Users,
} from 'lucide-react'
import {
  defaultSpaceKey,
  useArchiveSpace,
  useCreateSpace,
  useCurrentUser,
  useDeleteSpaceMember,
  useSpaceMembers,
  useSpaceTree,
  useSpaces,
  useUpdateSpace,
  useUpsertSpaceMember,
  useUsers,
} from '@/shared/api/hooks'
import { ConfirmDialog, EmptyState, ErrorState, LoadingState } from '@sdlc/ui/ui'
import { Button } from '@sdlc/ui/ui'
import { Input } from '@sdlc/ui/ui'
import { Label } from '@sdlc/ui/ui'
import { Textarea } from '@sdlc/ui/ui'
import { formatApiErrorForUser } from '@/shared/lib/api-error'
import { formatDateTime, formatDocumentType, shortText } from '@/shared/lib/wiki-format'
import type { Space, SpaceMember, SpaceTreeNode, User } from '@/api/wiki'

const spaceRoleOptions = [
  { value: 'viewer', label: 'читатель' },
  { value: 'editor', label: 'редактор' },
  { value: 'admin', label: 'администратор' },
]
const selectClassName =
  'flex min-h-10 w-full rounded-md border border-border-strong bg-surface px-3 py-1 text-sm text-text-primary shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent disabled:cursor-not-allowed disabled:opacity-50'

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

function roleLabel(role: string): string {
  return spaceRoleOptions.find((option) => option.value === role)?.label ?? role
}

function statusLabel(status: string): string {
  return status === 'archived' ? 'архивировано' : 'активно'
}

function userOptionLabel(user: User): string {
  return `${user.display_name ?? user.username ?? user.email} · ${user.email}`
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
    <form onSubmit={handleSubmit} className="rounded-md border border-border bg-surface p-3">
      <div className="grid gap-3 lg:grid-cols-[10rem_1fr_1.5fr_auto] lg:items-end">
        <div className="space-y-1.5">
          <Label htmlFor="space-key">Ключ</Label>
          <Input
            id="space-key"
            value={key}
            onChange={(event) => setKey(event.target.value)}
            placeholder="KEY"
          />
        </div>
        <div className="space-y-1.5">
          <Label htmlFor="space-name">Название</Label>
          <Input
            id="space-name"
            value={name}
            onChange={(event) => setName(event.target.value)}
            placeholder="Название пространства"
            required
          />
        </div>
        <div className="space-y-1.5">
          <Label htmlFor="space-description">Описание</Label>
          <Input
            id="space-description"
            value={description}
            onChange={(event) => setDescription(event.target.value)}
            placeholder="Описание пространства"
          />
        </div>
        <Button disabled={createSpace.isPending || !key.trim() || !name.trim()}>
          <Plus className="h-4 w-4" />
          {createSpace.isPending ? 'Создаём...' : 'Создать пространство'}
        </Button>
      </div>
      {createSpace.isError && (
        <p className="mt-2 text-sm text-danger">
          {formatApiErrorForUser(createSpace.error, 'Не удалось создать пространство')}
        </p>
      )}
    </form>
  )
}

function SpaceMembers({
  canManage,
  members,
  spaceKey,
  users,
}: {
  canManage: boolean
  members: SpaceMember[]
  spaceKey: string
  users: User[]
}) {
  const upsertMember = useUpsertSpaceMember()
  const deleteMember = useDeleteSpaceMember()
  const [userId, setUserId] = useState('')
  const [role, setRole] = useState('viewer')
  const hasUserOptions = users.length > 0

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    upsertMember.mutate(
      {
        spaceKey,
        userId: userId.trim(),
        body: { role },
      },
      {
        onSuccess: () => {
          setUserId('')
          setRole('viewer')
        },
      },
    )
  }

  return (
    <div className="space-y-3 rounded-md border border-border p-3">
      <div className="flex items-center gap-2 text-xs font-medium uppercase text-text-muted">
        <Users className="h-3.5 w-3.5" />
        Участники
      </div>

      {members.length === 0 ? (
        <EmptyState message="Участники пока не назначены" />
      ) : (
        <div className="space-y-2">
          {members.map((member) => (
            <div
              key={member.user_id}
              className="grid gap-2 rounded-md border border-border bg-surface-raised p-2 text-sm md:grid-cols-[1fr_auto_auto] md:items-center"
            >
              <div className="min-w-0">
                <div className="truncate font-medium">{member.display_name}</div>
                <div className="truncate text-xs text-text-muted">{member.email}</div>
              </div>
              <span className="text-xs text-text-secondary">{roleLabel(member.role)}</span>
              {canManage && (
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={() => deleteMember.mutate({ spaceKey, userId: member.user_id })}
                  disabled={deleteMember.isPending}
                >
                  <UserMinus className="h-3.5 w-3.5" />
                  Удалить
                </Button>
              )}
            </div>
          ))}
        </div>
      )}

      {canManage && (
        <form onSubmit={handleSubmit} className="grid gap-2 md:grid-cols-[1fr_10rem_auto]">
          <div className="space-y-1.5">
            <Label htmlFor={`${spaceKey}-member-user`}>Пользователь</Label>
            {hasUserOptions ? (
              <select
                id={`${spaceKey}-member-user`}
                className={selectClassName}
                value={userId}
                onChange={(event) => setUserId(event.target.value)}
                required
              >
                <option value="">Выберите пользователя</option>
                {users.map((user) => (
                  <option key={user.id} value={user.id}>
                    {userOptionLabel(user)}
                  </option>
                ))}
              </select>
            ) : (
              <Input
                id={`${spaceKey}-member-user`}
                value={userId}
                onChange={(event) => setUserId(event.target.value)}
                placeholder="ID пользователя"
                required
              />
            )}
          </div>
          <div className="space-y-1.5">
            <Label htmlFor={`${spaceKey}-member-role`}>Роль</Label>
            <select
              id={`${spaceKey}-member-role`}
              className={selectClassName}
              value={role}
              onChange={(event) => setRole(event.target.value)}
              required
            >
              {spaceRoleOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>
          <Button className="self-end" disabled={upsertMember.isPending || !userId.trim()}>
            <UserPlus className="h-4 w-4" />
            {upsertMember.isPending ? 'Назначаем...' : 'Назначить'}
          </Button>
        </form>
      )}

      {upsertMember.isError && (
        <p className="text-sm text-danger">
          {formatApiErrorForUser(upsertMember.error, 'Не удалось назначить участника')}
        </p>
      )}
      {deleteMember.isError && (
        <p className="text-sm text-danger">
          {formatApiErrorForUser(deleteMember.error, 'Не удалось удалить участника')}
        </p>
      )}
    </div>
  )
}

function SpaceDetails({
  currentUserId,
  isSystemAdmin,
  onArchived,
  space,
  users,
}: {
  currentUserId?: string
  isSystemAdmin: boolean
  onArchived: (space: Space) => void
  space: Space
  users: User[]
}) {
  const membersQuery = useSpaceMembers(space.key)
  const updateSpace = useUpdateSpace()
  const archiveSpace = useArchiveSpace()
  const members = membersQuery.data?.members ?? []
  const canManage =
    space.status !== 'archived' &&
    (isSystemAdmin ||
      members.some((member) => member.user_id === currentUserId && member.role === 'admin'))
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
        <form onSubmit={handleUpdate} className="space-y-3 rounded-md border border-border p-3">
          <div className="grid gap-3 md:grid-cols-2">
            <div className="space-y-1.5">
              <Label htmlFor={`${space.key}-name`}>Название пространства</Label>
              <Input
                id={`${space.key}-name`}
                value={name}
                onChange={(event) => setName(event.target.value)}
                required
              />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor={`${space.key}-description`}>Описание</Label>
              <Textarea
                id={`${space.key}-description`}
                value={description}
                onChange={(event) => setDescription(event.target.value)}
              />
            </div>
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
            <p className="text-sm text-danger">
              {formatApiErrorForUser(updateSpace.error, 'Не удалось обновить пространство')}
            </p>
          )}
        </form>
      )}

      <ConfirmDialog
        open={archiveOpen}
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

      <div className="space-y-2 rounded-md border border-border p-3">
        <div className="flex items-center gap-2 text-xs font-medium uppercase text-text-muted">
          <BookOpenText className="h-3.5 w-3.5" />
          Дерево
        </div>
        <SpaceTreePreview spaceKey={space.key} />
      </div>

      {membersQuery.isLoading && <LoadingState message="Загружаем участников" />}
      {membersQuery.isError && (
        <ErrorState
          message={formatApiErrorForUser(
            membersQuery.error,
            'Участников может смотреть только администратор пространства',
          )}
          onRetry={() => membersQuery.refetch()}
        />
      )}
      {!membersQuery.isLoading && !membersQuery.isError && (
        <SpaceMembers canManage={canManage} members={members} spaceKey={space.key} users={users} />
      )}
    </div>
  )
}

export function SpacesPage() {
  const currentUserQuery = useCurrentUser()
  const spacesQuery = useSpaces()
  const isSystemAdmin = currentUserQuery.data?.is_system_admin === true
  const usersQuery = useUsers(isSystemAdmin)
  const users = usersQuery.data?.users ?? []
  const [recentlyArchived, setRecentlyArchived] = useState<Space | null>(null)
  const spaces = (spacesQuery.data?.spaces ?? []).map((space) =>
    space.key === recentlyArchived?.key ? recentlyArchived : space,
  )
  const [showCreate, setShowCreate] = useState(false)
  const [search, setSearch] = useState('')
  const [status, setStatus] = useState('all')
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
      <section className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold">Пространства</h1>
          <p className="max-w-3xl text-sm text-text-muted">
            Пространства группируют документы по продуктам, командам и контекстам процесса.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          {isSystemAdmin && (
            <Button
              type="button"
              size="sm"
              variant="outline"
              className="min-h-10"
              aria-expanded={showCreate}
              onClick={() => setShowCreate((value) => !value)}
            >
              <Plus className="h-4 w-4" /> Создать пространство
            </Button>
          )}
          <Button asChild size="sm" className="min-h-10">
            <Link to="/documents/new">Новый документ</Link>
          </Button>
        </div>
      </section>

      {isSystemAdmin && showCreate && <CreateSpaceForm onCreated={() => setShowCreate(false)} />}

      {recentlyArchived && (
        <p role="status" className="border-l-2 border-success bg-surface-raised px-3 py-2 text-sm">
          Пространство «{recentlyArchived.name}» архивировано и остаётся доступным для чтения.
        </p>
      )}

      {spacesQuery.isLoading && <LoadingState message="Загружаем пространства" />}
      {spacesQuery.isError && (
        <ErrorState
          message={formatApiErrorForUser(spacesQuery.error, 'Не удалось загрузить пространства')}
          onRetry={() => spacesQuery.refetch()}
        />
      )}
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
        <section aria-label="Список пространств" className="space-y-3">
          <div className="grid gap-2 sm:grid-cols-[minmax(0,1fr)_11rem_11rem_auto]">
            <div className="relative min-w-0">
              <Search
                className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-text-muted"
                aria-hidden
              />
              <Input
                type="search"
                aria-label="Найти пространство"
                placeholder="Название, ключ или описание"
                value={search}
                onChange={(event) => {
                  setSearch(event.target.value)
                  resetListPosition()
                }}
                className="min-h-10 pl-9"
              />
            </div>
            <select
              aria-label="Статус пространства"
              className={selectClassName}
              value={status}
              onChange={(event) => {
                setStatus(event.target.value)
                resetListPosition()
              }}
            >
              <option value="all">Все статусы</option>
              <option value="active">Активные</option>
              <option value="archived">Архивные</option>
            </select>
            <select
              aria-label="Сортировка пространств"
              className={selectClassName}
              value={sort}
              onChange={(event) => {
                setSort(event.target.value)
                resetListPosition()
              }}
            >
              <option value="updated">Сначала обновлённые</option>
              <option value="name">По названию</option>
            </select>
            {currentUserQuery.data && (
              <label className="flex min-h-10 cursor-pointer items-center gap-2 whitespace-nowrap text-sm text-text-secondary">
                <input
                  type="checkbox"
                  checked={mineOnly}
                  onChange={(event) => {
                    setMineOnly(event.target.checked)
                    resetListPosition()
                  }}
                  className="h-4 w-4 accent-accent"
                />
                Только мои
              </label>
            )}
          </div>
          <p className="text-xs text-text-muted">
            Показано {visibleSpaces.length} из {filteredSpaces.length} пространств
          </p>
          {filteredSpaces.length === 0 ? (
            <p role="status" className="border-y border-border py-6 text-sm text-text-muted">
              По заданным условиям пространства не найдены.
            </p>
          ) : (
            <ul className="divide-y divide-border border-y border-border">
              {visibleSpaces.map((space) => {
                const expanded = expandedKey === space.key
                const owner = users.find((user) => user.id === space.owner_id)
                return (
                  <li key={space.key}>
                    <button
                      type="button"
                      aria-expanded={expanded}
                      aria-controls={`space-details-${space.key}`}
                      onClick={() => setExpandedKey(expanded ? null : space.key)}
                      className="flex min-h-16 w-full min-w-0 items-start gap-3 py-2 text-left transition-colors hover:bg-surface-raised focus-visible:outline-2 focus-visible:outline-accent"
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
                          <span className="inline-flex items-center gap-1">
                            <Users className="h-3.5 w-3.5" aria-hidden />
                            {space.member_count} участников
                          </span>
                          {owner && <span>Владелец: {owner.display_name ?? owner.username}</span>}
                          <span>Обновлено {formatDateTime(space.updated_at)}</span>
                        </span>
                      </span>
                      <ChevronDown
                        className={`mt-1 h-4 w-4 shrink-0 text-text-muted transition-transform ${expanded ? 'rotate-180' : ''}`}
                        aria-hidden
                      />
                    </button>
                    {expanded && (
                      <div id={`space-details-${space.key}`}>
                        <SpaceDetails
                          currentUserId={currentUserQuery.data?.id}
                          isSystemAdmin={isSystemAdmin}
                          onArchived={handleArchived}
                          space={space}
                          users={users}
                        />
                      </div>
                    )}
                  </li>
                )
              })}
            </ul>
          )}
          {filteredSpaces.length > 12 && (
            <nav aria-label="Страницы пространств" className="flex items-center justify-end gap-2">
              <Button
                type="button"
                size="sm"
                variant="outline"
                className="min-h-10"
                disabled={currentPage === 1}
                onClick={() => {
                  setPage(currentPage - 1)
                  setExpandedKey(null)
                }}
              >
                Назад
              </Button>
              <span className="text-sm tabular-nums text-text-muted">
                {currentPage} / {pageCount}
              </span>
              <Button
                type="button"
                size="sm"
                variant="outline"
                className="min-h-10"
                disabled={currentPage === pageCount}
                onClick={() => {
                  setPage(currentPage + 1)
                  setExpandedKey(null)
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
