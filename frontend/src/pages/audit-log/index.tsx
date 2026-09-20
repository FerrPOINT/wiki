import { useRef, useState } from 'react'
import { ChevronLeft, ChevronRight, History } from 'lucide-react'
import { useAuditLog } from '@/shared/api/hooks'
import { EmptyState, ErrorState, LoadingState } from '@sdlc/ui/ui'
import { Card, CardContent, CardHeader, CardTitle } from '@sdlc/ui/ui'
import { Button } from '@sdlc/ui/ui'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@sdlc/ui/ui'
import { formatApiErrorForUser } from '@/shared/lib/api-error'
import { formatDateTime } from '@/shared/lib/wiki-format'

function countBy(entries: { action: string }[], needle: string): number {
  return entries.filter((entry) => entry.action.includes(needle)).length
}

const PAGE_SIZE = 20

export function AuditLogPage() {
  const [cursors, setCursors] = useState<(string | null)[]>([null])
  const eventsRef = useRef<HTMLDivElement>(null)
  const cursor = cursors[cursors.length - 1] ?? undefined
  const auditQuery = useAuditLog({ limit: PAGE_SIZE, cursor })
  const entries = auditQuery.data?.entries ?? []
  const nextCursor = auditQuery.data?.next_cursor
  const documentEvents = countBy(entries, 'document')
  const accessEvents = entries.filter((entry) =>
    ['member', 'role', 'space'].some((needle) => entry.action.includes(needle)),
  ).length
  const userEvents = entries.filter((entry) =>
    ['auth', 'user'].some((needle) => entry.action.includes(needle)),
  ).length

  function changePage(nextCursors: (string | null)[]) {
    setCursors(nextCursors)
    eventsRef.current?.scrollIntoView?.({ block: 'start' })
  }

  return (
    <div className="space-y-5">
      <section>
        <h1 className="text-2xl font-bold">Аудит</h1>
      </section>

      {!auditQuery.isLoading && !auditQuery.isError && entries.length > 0 && (
        <section
          aria-label="Сводка текущей страницы"
          className="flex flex-wrap gap-x-5 gap-y-2 border-y border-border py-3 text-sm"
        >
          <span>
            Показано <strong>{entries.length}</strong> (лимит {PAGE_SIZE})
          </span>
          <span>Документы: {documentEvents}</span>
          <span>Доступ: {accessEvents}</span>
          <span>Пользователи: {userEvents}</span>
        </section>
      )}

      <Card ref={eventsRef} className="scroll-mt-16">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-base">
            <History className="h-4 w-4 text-accent" />
            События
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
            <>
              <ul className="divide-y divide-border xl:hidden">
                {entries.map((event) => (
                  <li key={event.id} className="py-3 first:pt-0 last:pb-0">
                    <div className="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1">
                      <span className="min-w-0 break-all font-mono text-xs">{event.action}</span>
                      <time className="text-xs text-text-muted">
                        {formatDateTime(event.created_at)}
                      </time>
                    </div>
                    <dl className="mt-2 grid grid-cols-[auto_minmax(0,1fr)] gap-x-3 gap-y-1 text-xs">
                      <dt className="text-text-muted">Участник</dt>
                      <dd className="min-w-0 break-all">{event.actor_id}</dd>
                      <dt className="text-text-muted">Объект</dt>
                      <dd className="min-w-0 break-all">{event.entity_id}</dd>
                      <dt className="text-text-muted">Тип</dt>
                      <dd className="min-w-0 break-all">{event.entity_type}</dd>
                      <dt className="text-text-muted">Запрос</dt>
                      <dd className="min-w-0 break-all font-mono">{event.request_id}</dd>
                    </dl>
                  </li>
                ))}
              </ul>
              <div className="hidden xl:block">
                <Table className="table-fixed">
                  <TableHeader>
                    <TableRow>
                      <TableHead>Событие</TableHead>
                      <TableHead>Участник</TableHead>
                      <TableHead>Объект</TableHead>
                      <TableHead>Тип</TableHead>
                      <TableHead>Запрос</TableHead>
                      <TableHead>Время</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {entries.map((event) => (
                      <TableRow key={event.id}>
                        <TableCell className="break-all font-mono text-xs">
                          {event.action}
                        </TableCell>
                        <TableCell className="break-all">{event.actor_id}</TableCell>
                        <TableCell className="break-all">{event.entity_id}</TableCell>
                        <TableCell className="break-all">{event.entity_type}</TableCell>
                        <TableCell className="break-all font-mono text-xs">
                          {event.request_id}
                        </TableCell>
                        <TableCell className="text-xs text-text-muted">
                          {formatDateTime(event.created_at)}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>
            </>
          )}
          {(cursors.length > 1 || nextCursor) && (
            <nav
              aria-label="Страницы аудита"
              className="mt-4 flex flex-wrap items-center gap-3 border-t border-border pt-3"
            >
              <span className="mr-auto text-sm text-text-secondary">Страница {cursors.length}</span>
              <Button
                type="button"
                variant="outline"
                className="h-10"
                disabled={cursors.length === 1 || auditQuery.isLoading || auditQuery.isFetching}
                onClick={() => changePage(cursors.slice(0, -1))}
              >
                <ChevronLeft className="h-4 w-4" />
                Назад
              </Button>
              <Button
                type="button"
                variant="outline"
                className="h-10"
                disabled={
                  !nextCursor || auditQuery.isLoading || auditQuery.isFetching || auditQuery.isError
                }
                onClick={() => nextCursor && changePage([...cursors, nextCursor])}
              >
                Далее
                <ChevronRight className="h-4 w-4" />
              </Button>
            </nav>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
