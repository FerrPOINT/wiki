import { fireEvent, render, screen } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'

import { UsersPage } from './'

const useCreateUser = vi.hoisted(() => vi.fn())
const useUsers = vi.hoisted(() => vi.fn())

const createUserMutate = vi.hoisted(() => vi.fn())
const usersRefetch = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({
  useCreateUser,
  useUsers,
}))

function setupUsers({
  createOverrides = {},
  usersOverrides = {},
}: {
  createOverrides?: Record<string, unknown>
  usersOverrides?: Record<string, unknown>
} = {}) {
  useUsers.mockReturnValue({
    data: {
      users: [
        {
          active: true,
          display_name: 'Администратор',
          email: 'admin@example.test',
          id: 'user-admin',
          is_system_admin: true,
          role: 'admin',
          username: 'admin',
        },
      ],
    },
    isLoading: false,
    isError: false,
    refetch: usersRefetch,
    ...usersOverrides,
  })
  useCreateUser.mockReturnValue({
    mutate: createUserMutate,
    isPending: false,
    isError: false,
    error: null,
    ...createOverrides,
  })

  render(<UsersPage />)
}

describe('UsersPage', () => {
  afterEach(() => {
    vi.clearAllMocks()
  })

  it('renders permission denied query errors with a retry action', () => {
    setupUsers({
      usersOverrides: {
        data: undefined,
        isError: true,
        error: { code: 'FORBIDDEN', message: 'Forbidden' },
      },
    })

    expect(screen.getByRole('alert')).toHaveTextContent('Недостаточно прав для действия')
    fireEvent.click(screen.getByRole('button', { name: 'Повторить' }))
    expect(usersRefetch).toHaveBeenCalled()
  })

  it('renders create-user validation details without request identifiers', () => {
    setupUsers({
      createOverrides: {
        isError: true,
        error: {
          code: 'VALIDATION_ERROR',
          message: 'Validation failed; details=password: required; requestId=req-99',
          details: [{ field: 'password', message: 'required' }],
        },
      },
    })

    expect(screen.getByText('Проверьте заполнение полей: Пароль: required')).toBeInTheDocument()
    expect(screen.queryByText(/requestId/)).not.toBeInTheDocument()
  })
})
