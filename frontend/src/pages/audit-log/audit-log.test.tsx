import { fireEvent, render, screen, within } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'

import type { AuditEntry } from '@/api/wiki'

import { AuditLogPage } from './'

const useAuditLog = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({ useAuditLog }))

const entries: AuditEntry[] = [
  {
    id: 'event-3',
    actor_id: 'user-1',
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
  afterEach(() => vi.clearAllMocks())

  it('labels counts as page-only and navigates through cursor pages', () => {
    useAuditLog.mockImplementation(({ cursor }: { cursor?: string }) =>
      queryResult(
        cursor
          ? { entries: [entries[2]!], next_cursor: null }
          : { entries, next_cursor: 'cursor-1' },
      ),
    )
    render(<AuditLogPage />)

    expect(useAuditLog).toHaveBeenCalledWith({ limit: 20, cursor: undefined })
    const summary = screen.getByRole('region', { name: 'Сводка текущей страницы' })
    expect(summary).toHaveTextContent('Показано 3 (лимит 20)')
    expect(summary).toHaveTextContent('Документы: 1')
    expect(summary).toHaveTextContent('Доступ: 1')
    expect(summary).toHaveTextContent('Пользователи: 1')
    expect(screen.getByRole('navigation', { name: 'Страницы аудита' })).toHaveTextContent(
      'Страница 1',
    )

    fireEvent.click(screen.getByRole('button', { name: 'Далее' }))
    expect(useAuditLog).toHaveBeenLastCalledWith({ limit: 20, cursor: 'cursor-1' })
    expect(screen.getByRole('navigation', { name: 'Страницы аудита' })).toHaveTextContent(
      'Страница 2',
    )
    expect(screen.getByRole('button', { name: 'Далее' })).toBeDisabled()
    expect(
      within(screen.getByRole('region', { name: 'Сводка текущей страницы' })).getByText(/Показано/),
    ).toHaveTextContent('Показано 1 (лимит 20)')

    fireEvent.click(screen.getByRole('button', { name: 'Назад' }))
    expect(useAuditLog).toHaveBeenLastCalledWith({ limit: 20, cursor: undefined })
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
    render(<AuditLogPage />)

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
    render(<AuditLogPage />)

    expect(screen.getByText('Событий аудита пока нет')).toBeInTheDocument()
    expect(
      screen.queryByRole('region', { name: 'Сводка текущей страницы' }),
    ).not.toBeInTheDocument()
    expect(screen.queryByRole('navigation', { name: 'Страницы аудита' })).not.toBeInTheDocument()
  })
})
