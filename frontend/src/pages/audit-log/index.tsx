import { FileCheck2, History, LockKeyhole, UserRound } from 'lucide-react'
import { useAuditLog } from '@/shared/api/hooks'
import { EmptyState, ErrorState, LoadingState } from '@/shared/ui/async-states'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/shared/ui/table'
import { formatApiErrorForUser } from '@/shared/lib/api-error'
import { formatDateTime } from '@/shared/lib/wiki-format'

function countBy(entries: { action: string }[], needle: string): number {
  return entries.filter((entry) => entry.action.includes(needle)).length
}

export function AuditLogPage() {
  const auditQuery = useAuditLog()
  const entries = auditQuery.data?.entries ?? []
  const documentEvents = countBy(entries, 'document')
  const accessEvents = entries.filter((entry) =>
    ['member', 'role', 'space'].some((needle) => entry.action.includes(needle)),
  ).length
  const userEvents = entries.filter((entry) =>
    ['auth', 'user'].some((needle) => entry.action.includes(needle)),
  ).length

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
              <span className="text-2xl font-semibold">{documentEvents}</span>
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
              <span className="text-2xl font-semibold">{accessEvents}</span>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Пользователи</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-3">
              <UserRound className="h-5 w-5 text-accent" />
              <span className="text-2xl font-semibold">{userEvents}</span>
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
          {auditQuery.isLoading && <LoadingState message="Загружаем аудит" />}
          {auditQuery.isError && (
            <ErrorState
              message={formatApiErrorForUser(auditQuery.error, 'Не удалось загрузить аудит')}
              onRetry={() => auditQuery.refetch()}
            />
          )}
          {!auditQuery.isLoading && !auditQuery.isError && entries.length === 0 && (
            <EmptyState message="Событий аудита пока нет" />
          )}
          {!auditQuery.isLoading && !auditQuery.isError && entries.length > 0 && (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Событие</TableHead>
                  <TableHead>Участник</TableHead>
                  <TableHead>Объект</TableHead>
                  <TableHead>Тип</TableHead>
                  <TableHead>Время</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {entries.map((event) => (
                  <TableRow key={event.id}>
                    <TableCell className="font-mono text-xs">{event.action}</TableCell>
                    <TableCell>{event.actor_id}</TableCell>
                    <TableCell>{event.entity_id}</TableCell>
                    <TableCell>{event.entity_type}</TableCell>
                    <TableCell className="whitespace-nowrap text-xs text-text-muted">
                      {formatDateTime(event.created_at)}
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
