import { useRef, useState } from 'react'
import { ChevronLeft, ChevronRight } from 'lucide-react'
import { useAuditLog } from '@/shared/api/hooks'
import { EmptyState, ErrorState, LoadingState } from '@sdlc/ui/ui'
import { Button } from '@sdlc/ui/ui'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@sdlc/ui/ui'
import { formatApiErrorForUser } from '@/shared/lib/api-error'
import { formatDateTime } from '@/shared/lib/wiki-format'

const actionLabels: Record<string, string> = {
  'wiki.seeded': 'Создана начальная структура',
  'wiki.bootstrap': 'Инициализирована Wiki',
  'auth.register': 'Зарегистрирован пользователь',
  'auth.login': 'Вход пользователя',
  'auth.logout': 'Выход пользователя',
  'user.create': 'Создан пользователь',
  'user.update': 'Обновлён пользователь',
  'space.create': 'Создано пространство',
  'space.update': 'Обновлено пространство',
  'space.archive': 'Архивировано пространство',
  'space.member_upsert': 'Изменён участник пространства',
  'space.member_delete': 'Удалён участник пространства',
  'document.create': 'Создан документ',
  'document.draft_update': 'Обновлён черновик',
  'document.publish': 'Опубликован документ',
  'document.archive': 'Архивирован документ',
  'document.move': 'Перемещён документ',
  'task.link_document': 'Документ связан с задачей',
  'phase.link_document': 'Документ связан с фазой',
  'evidence.create': 'Добавлен материал',
  'attachment.upload': 'Загружен файл',
  'template.create': 'Создан шаблон',
}

const PAGE_SIZE = 20

export function AuditLogPage() {
  const [cursors, setCursors] = useState<(string | null)[]>([null])
  const eventsRef = useRef<HTMLElement>(null)
  const cursor = cursors[cursors.length - 1] ?? undefined
  const auditQuery = useAuditLog({ limit: PAGE_SIZE, cursor })
  const entries = auditQuery.data?.entries ?? []
  const nextCursor = auditQuery.data?.next_cursor

  function changePage(nextCursors: (string | null)[]) {
    setCursors(nextCursors)
    eventsRef.current?.scrollIntoView?.({ block: 'start' })
  }

  return (
    <div className="min-w-0 space-y-5">
      <h1 className="text-2xl font-bold">Аудит</h1>

      <section ref={eventsRef} aria-label="События аудита" className="min-w-0 scroll-mt-16">
        {!auditQuery.isLoading && !auditQuery.isError && entries.length > 0 && (
          <p role="status" className="mb-3 text-sm text-text-muted">
            Событий на странице: {entries.length}
          </p>
        )}
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
            <ul className="divide-y divide-border border-y border-border xl:hidden">
              {entries.map((event) => (
                <li key={event.id} className="py-3">
                  <div className="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1">
                    <span className="min-w-0 break-words text-sm font-medium">
                      {actionLabels[event.action] ?? event.action}
                    </span>
                    <time dateTime={event.created_at} className="text-xs text-text-muted">
                      {formatDateTime(event.created_at)}
                    </time>
                  </div>
                  <dl className="mt-1 grid grid-cols-[auto_minmax(0,1fr)] gap-x-3 gap-y-1 text-xs">
                    <dt className="text-text-muted">Участник</dt>
                    <dd className="min-w-0 break-all">{event.actor_id}</dd>
                    <dt className="text-text-muted">Объект</dt>
                    <dd className="min-w-0 break-all">
                      {event.entity_type}: {event.entity_id}
                    </dd>
                  </dl>
                  <details className="mt-1 text-xs">
                    <summary className="min-h-10 cursor-pointer py-2 text-text-secondary hover:text-text-primary">
                      Технические данные
                    </summary>
                    <dl className="grid grid-cols-[auto_minmax(0,1fr)] gap-x-3 gap-y-1 pb-2">
                      <dt className="text-text-muted">Действие</dt>
                      <dd className="min-w-0 break-all font-mono">{event.action}</dd>
                      <dt className="text-text-muted">Запрос</dt>
                      <dd className="min-w-0 break-all font-mono">{event.request_id}</dd>
                    </dl>
                  </details>
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
                    <TableHead>Запрос</TableHead>
                    <TableHead>Время</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {entries.map((event) => (
                    <TableRow key={event.id}>
                      <TableCell className="break-all">
                        <span className="block text-sm font-medium">
                          {actionLabels[event.action] ?? event.action}
                        </span>
                        {actionLabels[event.action] && (
                          <code className="text-xs text-text-muted">{event.action}</code>
                        )}
                      </TableCell>
                      <TableCell className="break-all">{event.actor_id}</TableCell>
                      <TableCell className="break-all">
                        <span className="block text-xs text-text-muted">{event.entity_type}</span>
                        {event.entity_id}
                      </TableCell>
                      <TableCell className="break-all font-mono text-xs">
                        {event.request_id}
                      </TableCell>
                      <TableCell className="whitespace-nowrap text-xs text-text-muted">
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
      </section>
    </div>
  )
}
