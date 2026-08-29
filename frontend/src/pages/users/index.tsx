import { ShieldCheck, UserPlus, UsersRound } from 'lucide-react'
import { Button } from '@/shared/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/shared/ui/table'

const users = [
  {
    name: 'Администратор',
    email: 'admin@example.test',
    role: 'системный администратор',
    spaces: 'все',
    status: 'активен',
  },
  {
    name: 'Техлид',
    email: 'tech.lead@example.test',
    role: 'владелец пространства',
    spaces: 'ENG, SDLC',
    status: 'активен',
  },
  {
    name: 'QA',
    email: 'qa@example.test',
    role: 'редактор',
    spaces: 'SDLC',
    status: 'активен',
  },
]

export function UsersPage() {
  return (
    <div className="space-y-5">
      <section className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold">Пользователи</h1>
          <p className="mt-1 max-w-3xl text-sm text-text-muted">
            Пользователи, роли и доступы к пространствам, документам и административным настройкам.
          </p>
        </div>
        <Button size="sm">
          <UserPlus className="h-4 w-4" />
          Пригласить
        </Button>
      </section>

      <section className="grid gap-4 sm:grid-cols-2">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Активные пользователи</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-3">
              <UsersRound className="h-5 w-5 text-accent" />
              <span className="text-2xl font-semibold">3</span>
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
              <span className="text-2xl font-semibold">1</span>
            </div>
          </CardContent>
        </Card>
      </section>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Матрица доступа</CardTitle>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Пользователь</TableHead>
                <TableHead>Email</TableHead>
                <TableHead>Роль</TableHead>
                <TableHead>Пространства</TableHead>
                <TableHead>Статус</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {users.map((user) => (
                <TableRow key={user.email}>
                  <TableCell className="font-medium">{user.name}</TableCell>
                  <TableCell>{user.email}</TableCell>
                  <TableCell>{user.role}</TableCell>
                  <TableCell>{user.spaces}</TableCell>
                  <TableCell>
                    <span className="rounded bg-surface-raised px-2 py-1 text-xs text-text-secondary">
                      {user.status}
                    </span>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  )
}
