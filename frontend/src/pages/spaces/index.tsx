import { FormEvent, useState } from 'react'
import { Link } from 'react-router'
import {
  Archive,
  BookOpenText,
  FileText,
  FolderOpen,
  Plus,
  Save,
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
import { EmptyState, ErrorState, LoadingState } from '@/shared/ui/async-states'
import { Button } from '@/shared/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'
import { Input } from '@/shared/ui/input'
import { Label } from '@/shared/ui/label'
import { Textarea } from '@/shared/ui/textarea'
import { formatApiErrorForUser } from '@/shared/lib/api-error'
import { formatDateTime, formatDocumentType, shortText } from '@/shared/lib/wiki-format'
import type { Space, SpaceMember, SpaceTreeNode, User } from '@/api/wiki'

const spaceRoleOptions = [
  { value: 'viewer', label: 'читатель' },
  { value: 'editor', label: 'редактор' },
  { value: 'admin', label: 'администратор' },
]
const selectClassName =
  'flex h-9 w-full rounded-md border border-border-strong bg-surface px-3 py-1 text-sm text-text-primary shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent disabled:cursor-not-allowed disabled:opacity-50'

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

function CreateSpaceForm({ canCreate }: { canCreate: boolean }) {
  const createSpace = useCreateSpace()
  const [key, setKey] = useState('')
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')

  if (!canCreate) return null

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

function SpaceCard({
  currentUserId,
  isSystemAdmin,
  space,
  users,
}: {
  currentUserId?: string
  isSystemAdmin: boolean
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
    <Card>
      <CardHeader>
        <div className="flex items-start justify-between gap-3">
          <CardTitle className="flex items-center gap-2 text-base">
            <FolderOpen className="h-4 w-4 text-accent" />
            {space.key} · {space.name}
          </CardTitle>
          <span className="rounded bg-surface-raised px-2 py-1 text-xs text-text-muted">
            {statusLabel(space.status)} · {formatDateTime(space.updated_at)}
          </span>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <p className="text-sm text-text-muted">{shortText(space.description)}</p>
        <div className="flex gap-3 text-xs text-text-secondary">
          <span className="inline-flex items-center gap-1">
            <FileText className="h-3.5 w-3.5" />
            {space.document_count}
          </span>
          <span className="inline-flex items-center gap-1">
            <Users className="h-3.5 w-3.5" />
            {space.member_count}
          </span>
        </div>

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
                onClick={() => archiveSpace.mutate(space.key)}
                disabled={archiveSpace.isPending}
              >
                <Archive className="h-3.5 w-3.5" />
                {archiveSpace.isPending ? 'Архивируем...' : 'Архивировать'}
              </Button>
            </div>
            {updateSpace.isError && (
              <p className="text-sm text-danger">
                {formatApiErrorForUser(updateSpace.error, 'Не удалось обновить пространство')}
              </p>
            )}
            {archiveSpace.isError && (
              <p className="text-sm text-danger">
                {formatApiErrorForUser(archiveSpace.error, 'Не удалось архивировать пространство')}
              </p>
            )}
          </form>
        )}

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
          <SpaceMembers
            canManage={canManage}
            members={members}
            spaceKey={space.key}
            users={users}
          />
        )}
      </CardContent>
    </Card>
  )
}

export function SpacesPage() {
  const currentUserQuery = useCurrentUser()
  const spacesQuery = useSpaces()
  const isSystemAdmin = currentUserQuery.data?.is_system_admin === true
  const usersQuery = useUsers(isSystemAdmin)
  const users = usersQuery.data?.users ?? []
  const spaces = spacesQuery.data?.spaces ?? []

  return (
    <div className="space-y-5">
      <section className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold">Пространства</h1>
          <p className="max-w-3xl text-sm text-text-muted">
            Пространства группируют документы по продуктам, командам и контекстам процесса.
          </p>
        </div>
        <Button asChild size="sm">
          <Link to="/documents/new">Новый документ</Link>
        </Button>
      </section>

      <CreateSpaceForm canCreate={isSystemAdmin} />

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
        <section className="grid gap-4 xl:grid-cols-2">
          {spaces.map((space) => (
            <SpaceCard
              key={space.key}
              currentUserId={currentUserQuery.data?.id}
              isSystemAdmin={isSystemAdmin}
              space={space}
              users={users}
            />
          ))}
        </section>
      )}
    </div>
  )
}
