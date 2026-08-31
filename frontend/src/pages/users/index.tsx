import { FormEvent, useState } from 'react'
import { ShieldCheck, UserPlus, UsersRound } from 'lucide-react'
import { useCreateUser, useUsers } from '@/shared/api/hooks'
import { EmptyState, ErrorState, LoadingState } from '@/shared/ui/async-states'
import { Button } from '@/shared/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'
import { Input } from '@/shared/ui/input'
import { Label } from '@/shared/ui/label'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/shared/ui/table'

function roleLabel(role: string | null | undefined, isSystemAdmin: boolean | undefined): string {
  if (isSystemAdmin) return 'системный администратор'
  if (role === 'admin') return 'администратор'
  if (role === 'editor') return 'редактор'
  return 'читатель'
}

export function UsersPage() {
  const usersQuery = useUsers()
  const createUser = useCreateUser()
  const users = usersQuery.data?.users ?? []
  const [email, setEmail] = useState('editor@example.test')
  const [username, setUsername] = useState('editor')
  const [displayName, setDisplayName] = useState('Редактор')
  const [password, setPassword] = useState('')
  const [role, setRole] = useState('viewer')
  const activeUsers = users.filter((user) => user.active !== false)
  const adminUsers = users.filter((user) => user.is_system_admin || user.role === 'admin')

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    createUser.mutate(
      {
        email: email.trim(),
        username: username.trim(),
        display_name: displayName.trim(),
        password,
        role,
      },
      {
        onSuccess: () => {
          setEmail('')
          setUsername('')
          setDisplayName('')
          setPassword('')
          setRole('viewer')
        },
      },
    )
  }

  return (
    <div className="space-y-5">
      <section>
        <h1 className="text-2xl font-bold">Пользователи</h1>
        <p className="mt-1 max-w-3xl text-sm text-text-muted">
          Пользователи, роли и доступы к пространствам, документам и административным настройкам.
        </p>
      </section>

      <form onSubmit={handleSubmit} className="rounded-md border border-border bg-surface p-3">
        <div className="grid gap-3 lg:grid-cols-[1fr_1fr_1fr_10rem]">
          <div className="space-y-1.5">
            <Label htmlFor="user-email">Email</Label>
            <Input
              id="user-email"
              type="email"
              value={email}
              onChange={(event) => setEmail(event.target.value)}
              required
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="user-username">Логин</Label>
            <Input
              id="user-username"
              value={username}
              onChange={(event) => setUsername(event.target.value)}
              required
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="user-name">Имя</Label>
            <Input
              id="user-name"
              value={displayName}
              onChange={(event) => setDisplayName(event.target.value)}
              required
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="user-role">Роль</Label>
            <Input
              id="user-role"
              list="user-role-options"
              value={role}
              onChange={(event) => setRole(event.target.value)}
              required
            />
            <datalist id="user-role-options">
              <option value="viewer">читатель</option>
              <option value="editor">редактор</option>
              <option value="admin">администратор</option>
            </datalist>
          </div>
        </div>
        <div className="mt-3 grid gap-3 md:grid-cols-[1fr_auto]">
          <Input
            type="password"
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            placeholder="Пароль"
            aria-label="Пароль нового пользователя"
            required
          />
          <Button disabled={createUser.isPending}>
            <UserPlus className="h-4 w-4" />
            {createUser.isPending ? 'Создаём...' : 'Создать пользователя'}
          </Button>
        </div>
        {createUser.isError && (
          <p className="mt-2 text-sm text-danger">
            {createUser.error?.message ?? 'Не удалось создать пользователя'}
          </p>
        )}
      </form>

      <section className="grid gap-4 sm:grid-cols-2">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Активные пользователи</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-3">
              <UsersRound className="h-5 w-5 text-accent" />
              <span className="text-2xl font-semibold">{activeUsers.length}</span>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Администраторы</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-3">
              <ShieldCheck className="h-5 w-5 text-success" />
              <span className="text-2xl font-semibold">{adminUsers.length}</span>
            </div>
          </CardContent>
        </Card>
      </section>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Матрица доступа</CardTitle>
        </CardHeader>
        <CardContent>
          {usersQuery.isLoading && <LoadingState message="Загружаем пользователей" />}
          {usersQuery.isError && (
            <ErrorState
              message="Не удалось загрузить пользователей"
              onRetry={() => usersQuery.refetch()}
            />
          )}
          {!usersQuery.isLoading && !usersQuery.isError && users.length === 0 && (
            <EmptyState message="Пользователи пока не созданы" />
          )}
          {!usersQuery.isLoading && !usersQuery.isError && users.length > 0 && (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Пользователь</TableHead>
                  <TableHead>Email</TableHead>
                  <TableHead>Роль</TableHead>
                  <TableHead>Статус</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {users.map((user) => (
                  <TableRow key={user.id}>
                    <TableCell className="font-medium">
                      {user.display_name ?? user.username ?? user.email}
                    </TableCell>
                    <TableCell>{user.email}</TableCell>
                    <TableCell>{roleLabel(user.role, user.is_system_admin)}</TableCell>
                    <TableCell>
                      <span className="rounded bg-surface-raised px-2 py-1 text-xs text-text-secondary">
                        {user.active === false ? 'выключен' : 'активен'}
                      </span>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
