import { type FormEvent, useEffect, useMemo, useRef, useState } from 'react'
import { useSearchParams } from 'react-router'
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
const previousCursorParam = 'previous_cursor'
type AuditFilters = typeof emptyFilters
type AuditFilterName = keyof AuditFilters

function normalized(value: string | null): string {
  return value?.trim() ?? ''
}

function canonicalDate(value: string | null): string {
  const normalizedValue = normalized(value)
  if (!normalizedValue) return ''
  const date = new Date(normalizedValue)
  return Number.isNaN(date.getTime()) ? '' : date.toISOString()
}

function toLocalDateTimeInput(value: string): string {
  if (!value) return ''
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return ''
  const localDate = new Date(date.getTime() - date.getTimezoneOffset() * 60_000)
  return localDate.toISOString().slice(0, 16)
}

function setOptionalParam(params: URLSearchParams, name: string, value: string): void {
  if (value) params.set(name, value)
  else params.delete(name)
}

function useAuditLogUrl() {
  const [searchParams, setSearchParams] = useSearchParams()
  const action = normalized(searchParams.get('action'))
  const entityType = normalized(searchParams.get('entity_type'))
  const requestedActorId = normalized(searchParams.get('actor_id'))
  const actorId = uuidPattern.test(requestedActorId) ? requestedActorId.toLowerCase() : ''
  const requestedFrom = canonicalDate(searchParams.get('from'))
  const requestedTo = canonicalDate(searchParams.get('to'))
  const hasInvalidRange =
    Boolean(requestedFrom && requestedTo) &&
    new Date(requestedFrom).getTime() >= new Date(requestedTo).getTime()
  const from = hasInvalidRange ? '' : requestedFrom
  const to = hasInvalidRange ? '' : requestedTo
  const cursor = normalized(searchParams.get('cursor')) || undefined
  const previousCursors = useMemo(
    () =>
      searchParams
        .getAll(previousCursorParam)
        .map((value) => value.trim())
        .filter(Boolean),
    [searchParams],
  )
  const appliedFilters = useMemo(
    () => ({ action, entity_type: entityType, actor_id: actorId, from, to }),
    [action, actorId, entityType, from, to],
  )
  const filtersForForm = useMemo(
    () => ({ ...appliedFilters, from: toLocalDateTimeInput(from), to: toLocalDateTimeInput(to) }),
    [appliedFilters, from, to],
  )

  useEffect(() => {
    const canonicalValues: Record<AuditFilterName | 'cursor', string> = {
      ...appliedFilters,
      cursor: cursor ?? '',
    }
    const canonicalPrevious = cursor ? previousCursors : []
    const shouldCanonicalize =
      Object.entries(canonicalValues).some(
        ([name, value]) =>
          searchParams.getAll(name).length !== (value ? 1 : 0) ||
          searchParams.get(name) !== (value || null),
      ) ||
      searchParams.getAll(previousCursorParam).length !== canonicalPrevious.length ||
      searchParams
        .getAll(previousCursorParam)
        .some((value, index) => value !== canonicalPrevious[index])
    if (!shouldCanonicalize) return

    setSearchParams(
      (current) => {
        const next = new URLSearchParams(current)
        for (const [name, value] of Object.entries(canonicalValues)) {
          setOptionalParam(next, name, value)
        }
        next.delete(previousCursorParam)
        for (const previousCursor of canonicalPrevious) {
          next.append(previousCursorParam, previousCursor)
        }
        return next
      },
      { replace: true },
    )
  }, [appliedFilters, cursor, previousCursors, searchParams, setSearchParams])

  function setFilters(filters: AuditFilters) {
    setSearchParams((current) => {
      const next = new URLSearchParams(current)
      for (const [name, value] of Object.entries(filters)) {
        setOptionalParam(next, name, value)
      }
      next.delete('cursor')
      next.delete(previousCursorParam)
      return next
    })
  }

  function changePage(direction: 'previous' | 'next', nextCursor?: string) {
    setSearchParams((current) => {
      const next = new URLSearchParams(current)
      const currentCursor = normalized(next.get('cursor'))
      const history = next
        .getAll(previousCursorParam)
        .map((value) => value.trim())
        .filter(Boolean)

      next.delete(previousCursorParam)
      if (direction === 'next') {
        const normalizedNextCursor = nextCursor?.trim() ?? ''
        if (!normalizedNextCursor) return current
        if (currentCursor) history.push(currentCursor)
        next.set('cursor', normalizedNextCursor)
      } else {
        setOptionalParam(next, 'cursor', history.pop() ?? '')
      }
      for (const previousCursor of history) next.append(previousCursorParam, previousCursor)
      return next
    })
  }

  return {
    appliedFilters,
    changePage,
    currentPage: cursor ? previousCursors.length + 2 : 1,
    cursor,
    filtersForForm,
    hasPreviousPage: Boolean(cursor),
    setFilters,
  }
}

export function AuditLogPage() {
  const [showFilters, setShowFilters] = useState(false)
  const [manualActorId, setManualActorId] = useState(false)
  const {
    appliedFilters,
    changePage: changeUrlPage,
    currentPage,
    cursor,
    filtersForForm,
    hasPreviousPage,
    setFilters,
  } = useAuditLogUrl()
  const [draftFilters, setDraftFilters] = useState(filtersForForm)
  const [filterError, setFilterError] = useState('')
  const eventsRef = useRef<HTMLElement>(null)
  const auditQuery = useAuditLog({ limit: PAGE_SIZE, cursor, ...appliedFilters })
  const usersQuery = useUsers(showFilters)
  const entries = auditQuery.data?.entries ?? []
  const nextCursor = auditQuery.data?.next_cursor
  const users = usersQuery.data?.users ?? []
  const activeFilterCount = Object.values(appliedFilters).filter(Boolean).length

  useEffect(() => {
    setDraftFilters(filtersForForm)
    setFilterError('')
  }, [filtersForForm])

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
    setFilters({
      action: draftFilters.action,
      entity_type: draftFilters.entity_type,
      actor_id: actorId,
      from: fromDate?.toISOString() ?? '',
      to: toDate?.toISOString() ?? '',
    })
    setFilterError('')
    setShowFilters(false)
  }

  function clearFilters() {
    setDraftFilters(emptyFilters)
    setFilters(emptyFilters)
    setFilterError('')
  }

  function changePage(direction: 'previous' | 'next', nextCursor?: string) {
    changeUrlPage(direction, nextCursor)
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
          className="h-10"
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
                  {draftFilters.action && !actionLabels[draftFilters.action] && (
                    <option value={draftFilters.action}>{draftFilters.action}</option>
                  )}
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
                  {draftFilters.entity_type &&
                    !entityTypes.some(([value]) => value === draftFilters.entity_type) && (
                      <option value={draftFilters.entity_type}>{draftFilters.entity_type}</option>
                    )}
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
                          className="h-10 w-10"
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
                        className="h-10 px-0"
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
                      className="h-10 px-0"
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
              <Button type="button" variant="outline" className="h-10" onClick={clearFilters}>
                Сбросить
              </Button>
              <Button type="submit" className="h-10">
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
              hasPreviousPage
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
                      <TableCell className="break-words">
                        <span className="block text-sm font-medium">
                          {actionLabels[event.action] ?? event.action}
                        </span>
                        {actionLabels[event.action] && (
                          <code className="break-all text-xs text-text-muted">{event.action}</code>
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
        {(hasPreviousPage || nextCursor) && (
          <nav
            aria-label="Страницы аудита"
            className="mt-4 flex flex-wrap items-center gap-3 border-t border-border pt-3"
          >
            <span className="mr-auto text-sm text-text-secondary">Страница {currentPage}</span>
            <Button
              type="button"
              variant="outline"
              className="h-10"
              disabled={!hasPreviousPage || auditQuery.isLoading || auditQuery.isFetching}
              onClick={() => changePage('previous')}
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
              onClick={() => nextCursor && changePage('next', nextCursor)}
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
