import { fireEvent, render, screen } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'

import { UsersPage } from './'

const useCreateUser = vi.hoisted(() => vi.fn())
const useUpdateUser = vi.hoisted(() => vi.fn())
const useUsers = vi.hoisted(() => vi.fn())

const createUserMutate = vi.hoisted(() => vi.fn())
const updateUserMutate = vi.hoisted(() => vi.fn())
const usersRefetch = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({
  useCreateUser,
  useUpdateUser,
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
        {
          active: true,
          display_name: 'Редактор',
          email: 'editor@example.test',
          id: 'user-editor',
          is_system_admin: false,
          role: 'user',
          username: 'editor',
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
  useUpdateUser.mockReturnValue({
    mutate: updateUserMutate,
    isPending: false,
    isError: false,
    error: null,
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

  it('submits new users only from explicit form input', () => {
    setupUsers()

    expect(screen.getByLabelText('Email')).toHaveValue('')
    expect(screen.getByLabelText('Логин')).toHaveValue('')
    expect(screen.getByLabelText('Имя')).toHaveValue('')

    fireEvent.change(screen.getByLabelText('Email'), {
      target: { value: 'editor@example.test' },
    })
    fireEvent.change(screen.getByLabelText('Логин'), {
      target: { value: 'editor' },
    })
    fireEvent.change(screen.getByLabelText('Имя'), {
      target: { value: 'Редактор' },
    })
    fireEvent.change(screen.getByLabelText('Пароль нового пользователя'), {
      target: { value: 'correct-horse-battery-staple' },
    })
    fireEvent.submit(screen.getByLabelText('Email').closest('form')!)

    expect(createUserMutate).toHaveBeenCalledWith(
      {
        email: 'editor@example.test',
        username: 'editor',
        display_name: 'Редактор',
        password: 'correct-horse-battery-staple',
        role: 'user',
      },
      { onSuccess: expect.any(Function) },
    )
  })

  it('submits global role and status updates through the shared API hook', () => {
    setupUsers()

    fireEvent.change(screen.getByLabelText('Роль пользователя editor@example.test'), {
      target: { value: 'admin' },
    })
    fireEvent.change(screen.getByLabelText('Статус пользователя editor@example.test'), {
      target: { value: 'disabled' },
    })
    fireEvent.click(screen.getAllByRole('button', { name: 'Сохранить' })[1]!)

    expect(updateUserMutate).toHaveBeenCalledWith({
      userId: 'user-editor',
      body: {
        role: 'admin',
        is_system_admin: true,
        active: false,
      },
    })
  })
})
