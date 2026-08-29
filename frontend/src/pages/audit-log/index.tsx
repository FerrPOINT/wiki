import { FileCheck2, History, LockKeyhole, UserRound } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/shared/ui/table'

const auditEvents = [
  {
    event: 'document.published',
    actor: 'tech.lead@example.test',
    target: 'Требования v2',
    scope: 'SDLC-42',
    time: '2026-08-27 10:12',
  },
  {
    event: 'evidence.linked',
    actor: 'qa@example.test',
    target: 'Отчёт smoke-проверки',
    scope: 'Проверка',
    time: '2026-08-27 09:41',
  },
  {
    event: 'space.permission_changed',
    actor: 'admin@example.test',
    target: 'OPS',
    scope: 'Пространство',
    time: '2026-08-26 18:05',
  },
]

export function AuditLogPage() {
  return (
    <div className="space-y-5">
      <section>
        <h1 className="text-2xl font-bold">Аудит</h1>
        <p className="mt-1 max-w-3xl text-sm text-text-muted">
          Неизменяемая история действий с документами, материалами, пользователями и правами.
        </p>
      </section>

      <section className="grid gap-4 sm:grid-cols-3">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">События документов</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-3">
              <FileCheck2 className="h-5 w-5 text-success" />
              <span className="text-2xl font-semibold">34</span>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">События доступа</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-3">
              <LockKeyhole className="h-5 w-5 text-warning" />
              <span className="text-2xl font-semibold">7</span>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Участники</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-3">
              <UserRound className="h-5 w-5 text-accent" />
              <span className="text-2xl font-semibold">6</span>
            </div>
          </CardContent>
        </Card>
      </section>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-base">
            <History className="h-4 w-4 text-accent" />
            Последние события
          </CardTitle>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Событие</TableHead>
                <TableHead>Участник</TableHead>
                <TableHead>Объект</TableHead>
                <TableHead>Область</TableHead>
                <TableHead>Время</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {auditEvents.map((event) => (
                <TableRow key={`${event.event}-${event.time}`}>
                  <TableCell className="font-mono text-xs">{event.event}</TableCell>
                  <TableCell>{event.actor}</TableCell>
                  <TableCell>{event.target}</TableCell>
                  <TableCell>{event.scope}</TableCell>
                  <TableCell className="whitespace-nowrap text-xs text-text-muted">
                    {event.time}
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
