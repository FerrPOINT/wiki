import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router'
import { describe, expect, it, vi } from 'vitest'

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
    isLoading: false,
    isError: false,
    refetch: vi.fn(),
  }
}

describe('AdminPage', () => {
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
    expect(screen.getByText('Активных: 1')).toBeInTheDocument()
    expect(screen.getByText('Документов: 2')).toBeInTheDocument()
    expect(screen.getByText('Файлы до 25 МБ')).toBeInTheDocument()
  })
})
