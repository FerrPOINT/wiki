import { fireEvent, render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { AdminPage } from './'

const useAuditLog = vi.hoisted(() => vi.fn())
const useSpaces = vi.hoisted(() => vi.fn())
const useUsers = vi.hoisted(() => vi.fn())
const useWikiSettings = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({
  useAuditLog,
  useSpaces,
  useUsers,
  useWikiSettings,
}))

function resolvedQuery(data: unknown) {
  return {
    data,
    error: null,
    isLoading: false,
    isError: false,
    refetch: vi.fn(),
  }
}

describe('AdminPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders API-backed administration overview', () => {
    useUsers.mockReturnValue(
      resolvedQuery({
        users: [
          {
            id: 'user-1',
            email: 'admin@example.test',
            username: 'admin',
            display_name: 'Администратор',
            role: 'admin',
            is_system_admin: true,
            active: true,
          },
        ],
      }),
    )
    useSpaces.mockReturnValue(
      resolvedQuery({
        spaces: [
          {
            id: 'space-sdlc',
            key: 'BASE',
            name: 'База знаний Base',
            description: 'Документы платформы Base',
            owner_id: 'user-1',
            status: 'active',
            document_count: 2,
            member_count: 1,
            created_at: '2026-08-31T10:00:00Z',
            updated_at: '2026-08-31T10:00:00Z',
          },
        ],
      }),
    )
    useAuditLog.mockReturnValue(
      resolvedQuery({
        entries: [
          {
            id: 'audit-1',
            actor_id: 'user-1',
            action: 'wiki.seeded',
            entity_type: 'space',
            entity_id: 'BASE',
            created_at: '2026-08-31T10:00:00Z',
          },
        ],
      }),
    )
    useWikiSettings.mockReturnValue(
      resolvedQuery({
        instance_name: 'Wiki',
        api_base_path: '/api/v1',
        default_space_key: 'BASE',
        default_language: 'ru',
        timezone: 'Europe/Moscow',
        registration_enabled: true,
        public_links_enabled: false,
        search_backend: 'PostgreSQL FTS',
        storage_backend: 'local',
        max_upload_bytes: 26214400,
        markdown_renderer: 'comrak',
        html_sanitizer: 'ammonia',
      }),
    )

    render(
      <MemoryRouter>
        <AdminPage />
      </MemoryRouter>,
    )

    expect(screen.getByRole('heading', { name: 'Администрирование' })).toBeInTheDocument()
    expect(screen.getByText('Состояние инстанса')).toBeInTheDocument()
    expect(screen.getByText('Активных профилей: 1')).toBeInTheDocument()
    expect(screen.getByText('Документов: 2')).toBeInTheDocument()
    expect(screen.getByText('Лимит файла: 25 МБ')).toBeInTheDocument()
    expect(screen.getByText(/Первая страница, лимит 50;/)).toBeInTheDocument()
    expect(useAuditLog).toHaveBeenCalledWith({ limit: 50 })

    const usersLink = screen.getByRole('link', { name: /Пользователи.*Admin Panel/ })
    expect(usersLink).toHaveAttribute('href', '/users')
    expect(usersLink).toHaveClass('min-h-20')
    expect(screen.queryByRole('link', { name: 'Открыть' })).not.toBeInTheDocument()
  })

  it('keeps available metrics visible and retries only the failed source', () => {
    const usersRefetch = vi.fn()
    const spacesRefetch = vi.fn()
    const auditRefetch = vi.fn()
    const settingsRefetch = vi.fn()

    useUsers.mockReturnValue({
      ...resolvedQuery({
        users: [
          {
            id: 'user-1',
            email: 'admin@example.test',
            username: 'admin',
            display_name: 'Администратор',
            role: 'admin',
            is_system_admin: true,
            active: true,
          },
        ],
      }),
      refetch: usersRefetch,
    })
    useSpaces.mockReturnValue({
      ...resolvedQuery({
        spaces: [
          {
            id: 'space-sdlc',
            key: 'BASE',
            name: 'База знаний Base',
            description: 'Документы платформы Base',
            owner_id: 'user-1',
            status: 'active',
            document_count: 2,
            member_count: 1,
            created_at: '2026-08-31T10:00:00Z',
            updated_at: '2026-08-31T10:00:00Z',
          },
        ],
      }),
      refetch: spacesRefetch,
    })
    useAuditLog.mockReturnValue({
      ...resolvedQuery({ entries: [], next_cursor: null }),
      refetch: auditRefetch,
    })
    useWikiSettings.mockReturnValue({
      data: undefined,
      error: {},
      isLoading: false,
      isError: true,
      refetch: settingsRefetch,
    })

    render(
      <MemoryRouter>
        <AdminPage />
      </MemoryRouter>,
    )

    expect(screen.getByText('Активных профилей: 1')).toBeInTheDocument()
    expect(screen.getByText('Документов: 2')).toBeInTheDocument()
    expect(screen.getByText('Первая страница, лимит 50; событий пока нет')).toBeInTheDocument()
    expect(screen.getByRole('alert')).toHaveTextContent('Не удалось загрузить настройки')

    const retry = screen.getByRole('button', {
      name: 'Повторить загрузку: Локальная регистрация',
    })
    expect(retry).toHaveClass('h-10', 'w-10', 'sm:min-h-10', 'sm:min-w-10')
    fireEvent.click(retry)

    expect(settingsRefetch).toHaveBeenCalledOnce()
    expect(usersRefetch).not.toHaveBeenCalled()
    expect(spacesRefetch).not.toHaveBeenCalled()
    expect(auditRefetch).not.toHaveBeenCalled()
  })
})
