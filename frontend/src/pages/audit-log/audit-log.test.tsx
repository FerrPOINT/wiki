import { fireEvent, render, screen, within } from '@testing-library/react'
import { MemoryRouter, useLocation, useNavigate } from 'react-router'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import type { AuditEntry, AuditLogParams } from '@/api/wiki'

import { AuditLogPage } from './'

const useAuditLog = vi.hoisted(() => vi.fn())
const useUsers = vi.hoisted(() => vi.fn())
const usersRefetch = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({ useAuditLog, useUsers }))

function RouterState() {
  const location = useLocation()
  const navigate = useNavigate()
  return (
    <>
      <div data-testid="router-location">{`${location.pathname}${location.search}`}</div>
      <button type="button" onClick={() => navigate(-1)}>
        История назад
      </button>
      <button type="button" onClick={() => navigate(1)}>
        История вперёд
      </button>
    </>
  )
}

function renderAuditLog(initialEntry = '/audit-log') {
  return render(
    <MemoryRouter initialEntries={[initialEntry]}>
      <AuditLogPage />
      <RouterState />
    </MemoryRouter>,
  )
}

const actorId = '00000000-0000-4000-8000-000000000001'

const entries: AuditEntry[] = [
  {
    id: 'event-3',
    actor_id: actorId,
    action: 'document.publish',
    entity_type: 'document',
    entity_id: 'document-1',
    request_id: 'request-3',
    created_at: '2026-09-20T10:03:00Z',
  },
  {
    id: 'event-2',
    actor_id: 'user-2',
    action: 'space.member.update',
    entity_type: 'space',
    entity_id: 'space-1',
    request_id: 'request-2',
    created_at: '2026-09-20T10:02:00Z',
  },
  {
    id: 'event-1',
    actor_id: 'user-3',
    action: 'auth.login',
    entity_type: 'user',
    entity_id: 'user-3',
    request_id: 'request-1',
    created_at: '2026-09-20T10:01:00Z',
  },
]

function queryResult(data: { entries: AuditEntry[]; next_cursor: string | null }) {
  return {
    data,
    isLoading: false,
    isFetching: false,
    isError: false,
    error: null,
    refetch: vi.fn(),
  }
}

describe('AuditLogPage', () => {
  beforeEach(() => {
    useUsers.mockReturnValue({
      data: { users: [{ id: actorId, display_name: 'Анна', email: 'anna@example.test' }] },
      isLoading: false,
      isError: false,
      refetch: usersRefetch,
    })
  })
  afterEach(() => vi.clearAllMocks())

  it('shows readable events, preserves technical IDs and navigates cursor pages', () => {
    useAuditLog.mockImplementation(({ cursor }: { cursor?: string }) =>
      queryResult(
        cursor
          ? { entries: [entries[2]!], next_cursor: null }
          : { entries, next_cursor: 'cursor-1' },
      ),
    )
    renderAuditLog()

    expect(useAuditLog).toHaveBeenCalledWith(
      expect.objectContaining({ limit: 20, cursor: undefined }),
    )
    const events = screen.getByRole('region', { name: 'События аудита' })
    expect(within(events).getByRole('status')).toHaveTextContent('Событий на странице: 3')
    expect(screen.getAllByText('Опубликован документ')).toHaveLength(2)
    expect(screen.getAllByText('document.publish')).toHaveLength(2)
    expect(screen.getAllByText('request-3')).toHaveLength(2)
    const details = screen.getAllByText('Технические данные')[0]!.closest('details')
    expect(details).not.toHaveAttribute('open')
    fireEvent.click(screen.getAllByText('Технические данные')[0]!)
    expect(details).toHaveAttribute('open')
    expect(screen.getByRole('navigation', { name: 'Страницы аудита' })).toHaveTextContent(
      'Страница 1',
    )

    fireEvent.click(screen.getByRole('button', { name: 'Далее' }))
    expect(useAuditLog).toHaveBeenLastCalledWith(
      expect.objectContaining({ limit: 20, cursor: 'cursor-1' }),
    )
    expect(screen.getByTestId('router-location')).toHaveTextContent('/audit-log?cursor=cursor-1')
    expect(screen.getByRole('navigation', { name: 'Страницы аудита' })).toHaveTextContent(
      'Страница 2',
    )
    expect(screen.getByRole('button', { name: 'Далее' })).toBeDisabled()
    expect(within(events).getByRole('status')).toHaveTextContent('Событий на странице: 1')

    fireEvent.click(screen.getByRole('button', { name: 'Назад' }))
    expect(useAuditLog).toHaveBeenLastCalledWith(
      expect.objectContaining({ limit: 20, cursor: undefined }),
    )
    expect(screen.getByTestId('router-location')).toHaveTextContent('/audit-log')
    expect(screen.getByRole('navigation', { name: 'Страницы аудита' })).toHaveTextContent(
      'Страница 1',
    )
  })

  it('keeps back navigation available when the next page fails', () => {
    useAuditLog.mockImplementation(({ cursor }: { cursor?: string }) =>
      cursor
        ? {
            data: undefined,
            isLoading: false,
            isFetching: false,
            isError: true,
            error: { code: 'UNAVAILABLE' },
            refetch: vi.fn(),
          }
        : queryResult({ entries, next_cursor: 'cursor-1' }),
    )
    renderAuditLog()

    fireEvent.click(screen.getByRole('button', { name: 'Далее' }))
    expect(screen.getByRole('alert')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Назад' })).toBeEnabled()
    expect(screen.getByRole('button', { name: 'Далее' })).toBeDisabled()

    fireEvent.click(screen.getByRole('button', { name: 'Назад' }))
    expect(screen.queryByRole('alert')).not.toBeInTheDocument()
    expect(screen.getByRole('navigation', { name: 'Страницы аудита' })).toHaveTextContent(
      'Страница 1',
    )
  })

  it('does not show zero totals or pagination for an empty history', () => {
    useAuditLog.mockReturnValue(queryResult({ entries: [], next_cursor: null }))
    renderAuditLog()

    expect(screen.getByText('Событий аудита пока нет')).toBeInTheDocument()
    expect(screen.queryByRole('status')).not.toBeInTheDocument()
    expect(screen.queryByRole('navigation', { name: 'Страницы аудита' })).not.toBeInTheDocument()
  })

  it('keeps back navigation when a later cursor page becomes empty', () => {
    useAuditLog.mockImplementation(({ cursor }: AuditLogParams) =>
      queryResult(
        cursor ? { entries: [], next_cursor: null } : { entries, next_cursor: 'cursor-1' },
      ),
    )
    renderAuditLog()

    fireEvent.click(screen.getByRole('button', { name: 'Далее' }))
    expect(screen.getByText('На этой странице событий больше нет')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Назад' })).toBeEnabled()
  })

  it('serializes multi-page cursor history without dropping neighboring parameters', () => {
    useAuditLog.mockImplementation(({ cursor }: AuditLogParams) =>
      queryResult({
        entries,
        next_cursor: cursor === 'cursor-2' ? null : cursor === 'cursor-1' ? 'cursor-2' : 'cursor-1',
      }),
    )
    renderAuditLog('/audit-log?keep=1')

    fireEvent.click(screen.getByRole('button', { name: 'Далее' }))
    fireEvent.click(screen.getByRole('button', { name: 'Далее' }))
    expect(screen.getByTestId('router-location')).toHaveTextContent(
      '/audit-log?keep=1&cursor=cursor-2&previous_cursor=cursor-1',
    )
    expect(screen.getByRole('navigation', { name: 'Страницы аудита' })).toHaveTextContent(
      'Страница 3',
    )

    fireEvent.click(screen.getByRole('button', { name: 'Назад' }))
    expect(screen.getByTestId('router-location')).toHaveTextContent(
      '/audit-log?keep=1&cursor=cursor-1',
    )
    expect(screen.getByRole('navigation', { name: 'Страницы аудита' })).toHaveTextContent(
      'Страница 2',
    )
  })

  it('falls back to the original code for an unknown action', () => {
    useAuditLog.mockReturnValue(
      queryResult({ entries: [{ ...entries[0]!, action: 'integration.sync' }], next_cursor: null }),
    )

    renderAuditLog()

    expect(screen.getAllByText('integration.sync')).toHaveLength(3)
  })

  it('applies filters before cursor paging and resets to the first page', () => {
    useAuditLog.mockImplementation(({ cursor }: AuditLogParams) =>
      queryResult(
        cursor
          ? { entries: [entries[2]!], next_cursor: null }
          : { entries, next_cursor: 'cursor-1' },
      ),
    )
    renderAuditLog()

    fireEvent.click(screen.getByRole('button', { name: 'Далее' }))
    expect(screen.getByRole('navigation', { name: 'Страницы аудита' })).toHaveTextContent(
      'Страница 2',
    )
    fireEvent.click(screen.getByRole('button', { name: 'Фильтры' }))
    expect(useUsers).toHaveBeenLastCalledWith(true)
    fireEvent.change(screen.getByLabelText('Действие'), { target: { value: 'document.publish' } })
    fireEvent.change(screen.getByLabelText('Тип объекта'), { target: { value: 'document' } })
    fireEvent.change(screen.getByLabelText('Участник'), { target: { value: actorId } })
    fireEvent.change(screen.getByLabelText('С даты (местное время)'), {
      target: { value: '2026-09-20T10:00' },
    })
    fireEvent.change(screen.getByLabelText('До даты (не включая)'), {
      target: { value: '2026-09-21T10:00' },
    })
    fireEvent.click(screen.getByRole('button', { name: 'Применить' }))

    expect(useAuditLog).toHaveBeenLastCalledWith({
      limit: 20,
      cursor: undefined,
      action: 'document.publish',
      entity_type: 'document',
      actor_id: actorId,
      from: new Date('2026-09-20T10:00').toISOString(),
      to: new Date('2026-09-21T10:00').toISOString(),
    })
    expect(screen.getByRole('button', { name: 'Фильтры (5)' })).toHaveAttribute(
      'aria-expanded',
      'false',
    )
    expect(screen.getByRole('navigation', { name: 'Страницы аудита' })).toHaveTextContent(
      'Страница 1',
    )

    fireEvent.click(screen.getByRole('button', { name: 'Фильтры (5)' }))
    fireEvent.click(screen.getByRole('button', { name: 'Сбросить' }))
    expect(useAuditLog).toHaveBeenLastCalledWith({
      limit: 20,
      cursor: undefined,
      action: '',
      entity_type: '',
      actor_id: '',
      from: '',
      to: '',
    })
    expect(screen.getByRole('button', { name: 'Фильтры' })).toBeInTheDocument()
  })

  it('restores filters and cursor history from a direct URL', () => {
    useAuditLog.mockReturnValue(queryResult({ entries, next_cursor: null }))
    const from = new Date('2026-09-20T10:00').toISOString()
    const to = new Date('2026-09-21T10:00').toISOString()
    const params = new URLSearchParams({
      keep: '1',
      action: 'integration.sync',
      entity_type: 'integration',
      actor_id: actorId.toUpperCase(),
      from,
      to,
      cursor: 'cursor-2',
    })
    params.append('previous_cursor', 'cursor-1')

    renderAuditLog(`/audit-log?${params}`)

    expect(useAuditLog).toHaveBeenLastCalledWith({
      limit: 20,
      cursor: 'cursor-2',
      action: 'integration.sync',
      entity_type: 'integration',
      actor_id: actorId,
      from,
      to,
    })
    expect(screen.getByRole('button', { name: 'Фильтры (5)' })).toBeInTheDocument()
    expect(screen.getByRole('navigation', { name: 'Страницы аудита' })).toHaveTextContent(
      'Страница 3',
    )

    fireEvent.click(screen.getByRole('button', { name: 'Фильтры (5)' }))
    expect(screen.getByLabelText('Действие')).toHaveValue('integration.sync')
    expect(screen.getByLabelText('Тип объекта')).toHaveValue('integration')
    expect(screen.getByLabelText('Участник')).toHaveValue(actorId)
    expect(screen.getByLabelText('С даты (местное время)')).toHaveValue('2026-09-20T10:00')
    expect(screen.getByLabelText('До даты (не включая)')).toHaveValue('2026-09-21T10:00')
    expect(screen.getByTestId('router-location')).toHaveTextContent('keep=1')
  })

  it('restores applied filters through browser Back and Forward', () => {
    useAuditLog.mockReturnValue(queryResult({ entries, next_cursor: null }))
    renderAuditLog('/audit-log?keep=1')

    fireEvent.click(screen.getByRole('button', { name: 'Фильтры' }))
    fireEvent.change(screen.getByLabelText('Действие'), { target: { value: 'document.publish' } })
    fireEvent.click(screen.getByRole('button', { name: 'Применить' }))
    expect(screen.getByTestId('router-location')).toHaveTextContent(
      '/audit-log?keep=1&action=document.publish',
    )

    fireEvent.click(screen.getByRole('button', { name: 'История назад' }))
    expect(screen.getByRole('button', { name: 'Фильтры' })).toBeInTheDocument()
    expect(useAuditLog).toHaveBeenLastCalledWith(expect.objectContaining({ action: '' }))

    fireEvent.click(screen.getByRole('button', { name: 'История вперёд' }))
    expect(screen.getByRole('button', { name: 'Фильтры (1)' })).toBeInTheDocument()
    expect(useAuditLog).toHaveBeenLastCalledWith(
      expect.objectContaining({ action: 'document.publish' }),
    )
  })

  it('canonicalizes invalid URL state without dropping neighboring parameters', () => {
    useAuditLog.mockReturnValue(queryResult({ entries, next_cursor: null }))
    renderAuditLog(
      '/audit-log?keep=1&action=%20document.publish%20&actor_id=bad&actor_id=duplicate&from=bad&to=bad&cursor=%20&previous_cursor=cursor-1',
    )

    expect(screen.getByTestId('router-location')).toHaveTextContent(
      '/audit-log?keep=1&action=document.publish',
    )
    expect(useAuditLog).toHaveBeenLastCalledWith({
      limit: 20,
      cursor: undefined,
      action: 'document.publish',
      entity_type: '',
      actor_id: '',
      from: '',
      to: '',
    })
  })

  it('keeps audit controls at the platform target size', () => {
    useAuditLog.mockReturnValue(queryResult({ entries, next_cursor: null }))
    renderAuditLog()

    const filterButton = screen.getByRole('button', { name: 'Фильтры' })
    expect(filterButton).toHaveClass('h-10')
    fireEvent.click(filterButton)
    expect(screen.getByRole('button', { name: 'Ввести UUID' })).toHaveClass('h-10')
    expect(screen.getByRole('button', { name: 'Сбросить' })).toHaveClass('h-10')
    expect(screen.getByRole('button', { name: 'Применить' })).toHaveClass('h-10')
  })

  it('blocks an inverted date range without changing the audit request', () => {
    useAuditLog.mockReturnValue(queryResult({ entries, next_cursor: null }))
    renderAuditLog()

    fireEvent.click(screen.getByRole('button', { name: 'Фильтры' }))
    fireEvent.change(screen.getByLabelText('С даты (местное время)'), {
      target: { value: '2026-09-21T10:00' },
    })
    fireEvent.change(screen.getByLabelText('До даты (не включая)'), {
      target: { value: '2026-09-20T10:00' },
    })
    fireEvent.click(screen.getByRole('button', { name: 'Применить' }))

    expect(screen.getByRole('alert')).toHaveTextContent('Дата начала должна быть раньше')
    expect(useAuditLog.mock.calls.every((call) => !call[0]?.from && !call[0]?.to)).toBe(true)
  })

  it('offers manual actor ID and retry when the user directory fails', () => {
    useUsers.mockReturnValue({
      data: undefined,
      isLoading: false,
      isError: true,
      refetch: usersRefetch,
    })
    useAuditLog.mockReturnValue(queryResult({ entries, next_cursor: null }))
    renderAuditLog()

    fireEvent.click(screen.getByRole('button', { name: 'Фильтры' }))
    expect(screen.getByText('Каталог пользователей недоступен; укажите UUID')).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Повторить загрузку пользователей' }))
    expect(usersRefetch).toHaveBeenCalledOnce()
    fireEvent.change(screen.getByLabelText('Участник'), { target: { value: 'bad-id' } })
    fireEvent.click(screen.getByRole('button', { name: 'Применить' }))
    expect(screen.getByRole('alert')).toHaveTextContent('Укажите UUID участника')

    fireEvent.change(screen.getByLabelText('Участник'), { target: { value: actorId } })
    fireEvent.click(screen.getByRole('button', { name: 'Применить' }))
    expect(useAuditLog).toHaveBeenLastCalledWith(
      expect.objectContaining({ actor_id: actorId, cursor: undefined }),
    )
  })

  it('accepts a historical actor UUID even when the user directory loads', () => {
    useAuditLog.mockReturnValue(queryResult({ entries, next_cursor: null }))
    renderAuditLog()

    fireEvent.click(screen.getByRole('button', { name: 'Фильтры' }))
    fireEvent.click(screen.getByRole('button', { name: 'Ввести UUID' }))
    const historicalActorId = '00000000-0000-4000-8000-000000000099'
    fireEvent.change(screen.getByLabelText('Участник'), { target: { value: historicalActorId } })
    fireEvent.click(screen.getByRole('button', { name: 'Применить' }))

    expect(useAuditLog).toHaveBeenLastCalledWith(
      expect.objectContaining({ actor_id: historicalActorId }),
    )
    fireEvent.click(screen.getByRole('button', { name: 'Фильтры (1)' }))
    fireEvent.click(screen.getByRole('button', { name: 'Выбрать из списка' }))
    expect(screen.getByLabelText('Участник')).toHaveValue(historicalActorId)
  })

  it('keeps filters available after an empty filtered result', () => {
    useAuditLog.mockImplementation(({ action }: AuditLogParams) =>
      queryResult({ entries: action ? [] : entries, next_cursor: null }),
    )
    renderAuditLog()

    fireEvent.click(screen.getByRole('button', { name: 'Фильтры' }))
    fireEvent.change(screen.getByLabelText('Действие'), { target: { value: 'document.publish' } })
    fireEvent.click(screen.getByRole('button', { name: 'Применить' }))

    expect(screen.getByText('Событий по фильтрам не найдено')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Фильтры (1)' })).toBeInTheDocument()
  })
})
