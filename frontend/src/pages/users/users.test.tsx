import { fireEvent, render, screen } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { UsersPage } from './'

const useUsers = vi.hoisted(() => vi.fn())
const usersRefetch = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({ useUsers }))

function setupUsers(overrides: Record<string, unknown> = {}) {
  useUsers.mockReturnValue({
    data: {
      users: [
        {
          id: 'u1',
          email: 'admin@example.test',
          display_name: 'Анна',
          username: 'anna',
          active: true,
        },
        {
          id: 'u2',
          email: 'editor@example.test',
          display_name: 'Редактор',
          username: 'editor',
          active: false,
        },
      ],
    },
    isLoading: false,
    isError: false,
    refetch: usersRefetch,
    ...overrides,
  })
  render(<UsersPage />)
}

describe('UsersPage', () => {
  afterEach(() => vi.clearAllMocks())

  it('shows Wiki profiles without local password or role controls', () => {
    setupUsers()
    expect(screen.getByRole('heading', { name: 'Профили Wiki' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'Учётные записи' })).toHaveAttribute(
      'href',
      'http://localhost:7772/users',
    )
    expect(screen.getAllByText('Редактор')).toHaveLength(2)
    expect(screen.getAllByText('Профиль отключён')).toHaveLength(2)
    expect(screen.getAllByText('Профиль доступен')).toHaveLength(2)
    expect(screen.queryByLabelText(/пароль|роль/i)).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /создать|сохранить/i })).not.toBeInTheDocument()
  })

  it('filters users by name and email without losing the directory', () => {
    setupUsers()
    fireEvent.change(screen.getByRole('textbox', { name: 'Поиск пользователей' }), {
      target: { value: 'editor@' },
    })
    expect(screen.getAllByText('Редактор')).toHaveLength(2)
    expect(screen.queryByText('Анна')).not.toBeInTheDocument()
    fireEvent.change(screen.getByRole('textbox', { name: 'Поиск пользователей' }), {
      target: { value: 'missing' },
    })
    expect(screen.getByText('Ничего не найдено')).toBeInTheDocument()
  })

  it('renders an error with retry and keeps search available', () => {
    setupUsers({
      data: undefined,
      isError: true,
      error: { code: 'FORBIDDEN', message: 'Forbidden' },
    })
    expect(screen.getByRole('alert')).toHaveTextContent('Недостаточно прав для действия')
    fireEvent.click(screen.getByRole('button', { name: 'Повторить' }))
    expect(usersRefetch).toHaveBeenCalled()
    expect(screen.getByRole('textbox', { name: 'Поиск пользователей' })).toBeInTheDocument()
  })
})
