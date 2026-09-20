import { type FormEvent, useRef, useState } from 'react'
import { ChevronLeft, ChevronRight, ListFilter, RefreshCw } from 'lucide-react'
import { useAuditLog, useUsers } from '@/shared/api/hooks'
import { EmptyState, ErrorState, LoadingState } from '@sdlc/ui/ui'
import { Button } from '@sdlc/ui/ui'
import { Input } from '@sdlc/ui/ui'
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
const selectClassName =
  'min-h-10 w-full rounded-md border border-border-strong bg-surface px-3 text-sm text-text-primary focus-visible:outline-2 focus-visible:outline-accent disabled:opacity-50'
const entityTypes = [
  ['document', 'Документ'],
  ['space', 'Пространство'],
  ['user', 'Пользователь'],
  ['task', 'Задача'],
  ['phase', 'Фаза'],
  ['evidence', 'Материал'],
  ['attachment', 'Файл'],
  ['template', 'Шаблон'],
] as const
const emptyFilters = { action: '', entity_type: '', actor_id: '', from: '', to: '' }
const uuidPattern = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i

export function AuditLogPage() {
  const [showFilters, setShowFilters] = useState(false)
  const [manualActorId, setManualActorId] = useState(false)
  const [draftFilters, setDraftFilters] = useState(emptyFilters)
  const [appliedFilters, setAppliedFilters] = useState(emptyFilters)
  const [filterError, setFilterError] = useState('')
  const [cursors, setCursors] = useState<(string | null)[]>([null])
  const eventsRef = useRef<HTMLElement>(null)
  const cursor = cursors[cursors.length - 1] ?? undefined
  const auditQuery = useAuditLog({ limit: PAGE_SIZE, cursor, ...appliedFilters })
  const usersQuery = useUsers(showFilters)
  const entries = auditQuery.data?.entries ?? []
  const nextCursor = auditQuery.data?.next_cursor
  const users = usersQuery.data?.users ?? []
  const activeFilterCount = Object.values(appliedFilters).filter(Boolean).length

  function updateDraft(name: keyof typeof emptyFilters, value: string) {
    setDraftFilters((current) => ({ ...current, [name]: value }))
    setFilterError('')
  }

  function applyFilters(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const actorId = draftFilters.actor_id.trim()
    if (actorId && !uuidPattern.test(actorId)) {
      setFilterError('Укажите UUID участника')
      return
    }
    const fromDate = draftFilters.from ? new Date(draftFilters.from) : null
    const toDate = draftFilters.to ? new Date(draftFilters.to) : null
    if (
      (fromDate && Number.isNaN(fromDate.getTime())) ||
      (toDate && Number.isNaN(toDate.getTime()))
    ) {
      setFilterError('Проверьте даты периода')
      return
    }
    if (fromDate && toDate && fromDate.getTime() >= toDate.getTime()) {
      setFilterError('Дата начала должна быть раньше даты окончания')
      return
    }
    setAppliedFilters({
      action: draftFilters.action,
      entity_type: draftFilters.entity_type,
      actor_id: actorId,
      from: fromDate?.toISOString() ?? '',
      to: toDate?.toISOString() ?? '',
    })
    setCursors([null])
    setFilterError('')
    setShowFilters(false)
  }

  function clearFilters() {
    setDraftFilters(emptyFilters)
    setAppliedFilters(emptyFilters)
    setCursors([null])
    setFilterError('')
  }

  function changePage(nextCursors: (string | null)[]) {
    setCursors(nextCursors)
    eventsRef.current?.scrollIntoView?.({ block: 'start' })
  }

  return (
    <div className="min-w-0 space-y-5">
      <h1 className="text-2xl font-bold">Аудит</h1>

      <div className="space-y-3">
        <Button
          type="button"
          size="sm"
          variant="outline"
          className="min-h-10"
          aria-expanded={showFilters}
          onClick={() => setShowFilters((value) => !value)}
        >
          <ListFilter className="h-4 w-4" />
          Фильтры{activeFilterCount > 0 ? ` (${activeFilterCount})` : ''}
        </Button>
        {showFilters && (
          <form
            aria-label="Фильтры аудита"
            onSubmit={applyFilters}
            className="space-y-3 border-y border-border py-3"
          >
            <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
              <div className="space-y-1">
                <label htmlFor="audit-action" className="text-sm font-medium">
                  Действие
                </label>
                <select
                  id="audit-action"
                  className={selectClassName}
                  value={draftFilters.action}
                  onChange={(event) => updateDraft('action', event.target.value)}
                >
                  <option value="">Все действия</option>
                  {Object.entries(actionLabels).map(([value, label]) => (
                    <option key={value} value={value}>
                      {label}
                    </option>
                  ))}
                </select>
              </div>
              <div className="space-y-1">
                <label htmlFor="audit-entity-type" className="text-sm font-medium">
                  Тип объекта
                </label>
                <select
                  id="audit-entity-type"
                  className={selectClassName}
                  value={draftFilters.entity_type}
                  onChange={(event) => updateDraft('entity_type', event.target.value)}
                >
                  <option value="">Все типы</option>
                  {entityTypes.map(([value, label]) => (
                    <option key={value} value={value}>
                      {label}
                    </option>
                  ))}
                </select>
              </div>
              <div className="space-y-1">
                <label htmlFor="audit-actor" className="text-sm font-medium">
                  Участник
                </label>
                {manualActorId ||
                usersQuery.isError ||
                (!usersQuery.isLoading && users.length === 0) ? (
                  <div className="space-y-1">
                    <div className="flex gap-2">
                      <Input
                        id="audit-actor"
                        className="min-h-10 min-w-0"
                        placeholder="UUID участника"
                        value={draftFilters.actor_id}
                        onChange={(event) => updateDraft('actor_id', event.target.value)}
                      />
                      {usersQuery.isError && (
                        <Button
                          type="button"
                          size="icon"
                          variant="outline"
                          className="min-h-10 min-w-10"
                          title="Повторить загрузку пользователей"
                          aria-label="Повторить загрузку пользователей"
                          onClick={() => usersQuery.refetch()}
                        >
                          <RefreshCw className="h-4 w-4" />
                        </Button>
                      )}
                    </div>
                    {usersQuery.isError && (
                      <p className="text-xs text-text-muted">
                        Каталог пользователей недоступен; укажите UUID
                      </p>
                    )}
                    {!usersQuery.isError && users.length > 0 && (
                      <Button
                        type="button"
                        size="sm"
                        variant="ghost"
                        className="h-8 px-0"
                        onClick={() => setManualActorId(false)}
                      >
                        Выбрать из списка
                      </Button>
                    )}
                  </div>
                ) : (
                  <div className="space-y-1">
                    <select
                      id="audit-actor"
                      className={selectClassName}
                      value={draftFilters.actor_id}
                      disabled={usersQuery.isLoading}
                      onChange={(event) => updateDraft('actor_id', event.target.value)}
                    >
                      <option value="">
                        {usersQuery.isLoading ? 'Загружаем пользователей' : 'Все участники'}
                      </option>
                      {draftFilters.actor_id &&
                        !users.some((user) => user.id === draftFilters.actor_id) && (
                          <option value={draftFilters.actor_id}>{draftFilters.actor_id}</option>
                        )}
                      {users.map((user) => (
                        <option key={user.id} value={user.id}>
                          {user.display_name ?? user.username ?? user.email} · {user.email}
                        </option>
                      ))}
                    </select>
                    <Button
                      type="button"
                      size="sm"
                      variant="ghost"
                      className="h-8 px-0"
                      onClick={() => setManualActorId(true)}
                    >
                      Ввести UUID
                    </Button>
                  </div>
                )}
              </div>
              <div className="space-y-1">
                <label htmlFor="audit-from" className="text-sm font-medium">
                  С даты (местное время)
                </label>
                <Input
                  id="audit-from"
                  type="datetime-local"
                  className="min-h-10"
                  value={draftFilters.from}
                  onChange={(event) => updateDraft('from', event.target.value)}
                />
              </div>
              <div className="space-y-1">
                <label htmlFor="audit-to" className="text-sm font-medium">
                  До даты (не включая)
                </label>
                <Input
                  id="audit-to"
                  type="datetime-local"
                  className="min-h-10"
                  value={draftFilters.to}
                  onChange={(event) => updateDraft('to', event.target.value)}
                />
              </div>
            </div>
            {filterError && (
              <p role="alert" className="text-sm text-danger">
                {filterError}
              </p>
            )}
            <div className="flex flex-wrap justify-end gap-2">
              <Button type="button" variant="outline" className="min-h-10" onClick={clearFilters}>
                Сбросить
              </Button>
              <Button type="submit" className="min-h-10">
                Применить
              </Button>
            </div>
          </form>
        )}
      </div>

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
          <EmptyState
            message={
              cursors.length > 1
                ? 'На этой странице событий больше нет'
                : activeFilterCount > 0
                  ? 'Событий по фильтрам не найдено'
                  : 'Событий аудита пока нет'
            }
          />
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
