import { useState } from 'react'
import { ExternalLink, Search } from 'lucide-react'
import {
  Button,
  EmptyState,
  ErrorState,
  Input,
  LoadingState,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  usePlatformServices,
} from '@sdlc/ui/ui'
import { useUsers } from '@/shared/api/hooks'
import { formatApiErrorForUser } from '@/shared/lib/api-error'

export function UsersPage() {
  const usersQuery = useUsers()
  const { services } = usePlatformServices()
  const [search, setSearch] = useState('')
  const adminUrl = services.find((service) => service.key === 'admin-panel')?.ui_url
  const users = usersQuery.data?.users ?? []
  const query = search.trim().toLocaleLowerCase()
  const filtered = query
    ? users.filter((user) =>
        [user.display_name, user.username, user.email].some((value) =>
          value?.toLocaleLowerCase().includes(query),
        ),
      )
    : users

  return (
    <div className="min-w-0 space-y-5">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="text-2xl font-bold">Профили Wiki</h1>
          {!usersQuery.isLoading && !usersQuery.isError && (
            <p className="text-sm text-text-muted">{users.length} всего</p>
          )}
        </div>
        {adminUrl && (
          <Button asChild variant="secondary">
            <a href={`${adminUrl.replace(/\/$/, '')}/users`}>
              <ExternalLink className="h-4 w-4" />
              Учётные записи
            </a>
          </Button>
        )}
      </div>

      <div className="relative max-w-xl">
        <Search
          aria-hidden
          className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-text-muted"
        />
        <Input
          aria-label="Поиск пользователей"
          placeholder="Имя или email"
          value={search}
          onChange={(event) => setSearch(event.target.value)}
          className="pl-9"
        />
      </div>

      {usersQuery.isLoading && <LoadingState message="Загружаем пользователей" />}
      {usersQuery.isError && (
        <ErrorState
          message={formatApiErrorForUser(usersQuery.error, 'Не удалось загрузить пользователей')}
          onRetry={() => usersQuery.refetch()}
        />
      )}
      {!usersQuery.isLoading && !usersQuery.isError && filtered.length === 0 && (
        <EmptyState message={users.length === 0 ? 'Пользователей пока нет' : 'Ничего не найдено'} />
      )}
      {!usersQuery.isLoading && !usersQuery.isError && filtered.length > 0 && (
        <>
          <div className="divide-y divide-border md:hidden">
            {filtered.map((user) => (
              <div key={user.id} className="min-w-0 space-y-1 py-3">
                <div className="flex items-start justify-between gap-3">
                  <span className="min-w-0 break-words font-medium">
                    {user.display_name ?? user.username ?? user.email}
                  </span>
                  <span className="shrink-0 text-xs text-text-secondary">
                    {user.active === false ? 'Профиль отключён' : 'Профиль доступен'}
                  </span>
                </div>
                <div className="break-all text-sm text-text-muted">{user.email}</div>
              </div>
            ))}
          </div>
          <Table className="hidden table-fixed md:table">
            <TableHeader>
              <TableRow>
                <TableHead>Имя</TableHead>
                <TableHead>Email</TableHead>
                <TableHead className="w-44">Профиль Wiki</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {filtered.map((user) => (
                <TableRow key={user.id}>
                  <TableCell className="break-words font-medium">
                    {user.display_name ?? user.username ?? user.email}
                  </TableCell>
                  <TableCell className="break-all">{user.email}</TableCell>
                  <TableCell>
                    {user.active === false ? 'Профиль отключён' : 'Профиль доступен'}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </>
      )}
    </div>
  )
}
