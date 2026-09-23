import { act, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter, useLocation, useNavigate } from 'react-router'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { UsersPage } from './'

const useUsers = vi.hoisted(() => vi.fn())
const usersRefetch = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({ useUsers }))

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

function setupUsers(overrides: Record<string, unknown> = {}, initialEntry = '/users') {
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
  render(
    <MemoryRouter initialEntries={[initialEntry]}>
      <UsersPage />
      <RouterState />
    </MemoryRouter>,
  )
}

describe('UsersPage', () => {
  afterEach(() => {
    vi.useRealTimers()
    vi.clearAllMocks()
  })

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
    expect(screen.getByRole('link', { name: 'Учётные записи' })).toHaveClass('h-10')
    expect(screen.getByRole('textbox', { name: 'Поиск пользователей' })).toHaveClass('h-10')
  })

  it('filters users by name and email without losing the directory', () => {
    setupUsers()
    fireEvent.change(screen.getByRole('textbox', { name: 'Поиск пользователей' }), {
      target: { value: 'editor@' },
    })
    expect(screen.getAllByText('Редактор')).toHaveLength(2)
    expect(screen.queryByText('Анна')).not.toBeInTheDocument()
    expect(screen.getByText('Найдено: 1 из 2')).toBeInTheDocument()
    fireEvent.change(screen.getByRole('textbox', { name: 'Поиск пользователей' }), {
      target: { value: 'missing' },
    })
    expect(screen.getByText('По вашему запросу ничего не найдено')).toBeInTheDocument()
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

  it('restores search from a direct URL and preserves neighboring parameters', () => {
    setupUsers({}, '/users?keep=1&q=%20editor%40%20')

    expect(screen.getByRole('textbox', { name: 'Поиск пользователей' })).toHaveValue('editor@')
    expect(screen.getAllByText('Редактор')).toHaveLength(2)
    expect(screen.queryByText('Анна')).not.toBeInTheDocument()
    expect(screen.getByTestId('router-location')).toHaveTextContent('/users?keep=1&q=editor%40')
  })

  it('paginates a dense directory and restores pages through browser history', async () => {
    const users = Array.from({ length: 60 }, (_, index) => ({
      id: `u${index}`,
      email: `user-${String(index).padStart(3, '0')}@example.test`,
      display_name: `Пользователь ${String(index).padStart(3, '0')}`,
      username: `user-${index}`,
      active: true,
    }))
    setupUsers({ data: { users } }, '/users?keep=1&page=2')

    expect(screen.getAllByText('Пользователь 025')).toHaveLength(2)
    expect(screen.getAllByText('Пользователь 049')).toHaveLength(2)
    expect(screen.queryByText('Пользователь 024')).not.toBeInTheDocument()
    expect(screen.queryByText('Пользователь 050')).not.toBeInTheDocument()
    expect(screen.getByText('Показаны 26–50 из 60')).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: 'Далее' }))
    expect(screen.getByTestId('router-location')).toHaveTextContent('/users?keep=1&page=3')
    expect(screen.getAllByText('Пользователь 050')).toHaveLength(2)

    fireEvent.click(screen.getByRole('button', { name: 'История назад' }))
    await waitFor(() => expect(screen.getByText('2 / 3')).toBeInTheDocument())
    fireEvent.click(screen.getByRole('button', { name: 'История вперёд' }))
    await waitFor(() => expect(screen.getByText('3 / 3')).toBeInTheDocument())
  })

  it('resets pagination when search changes and canonicalizes invalid state', () => {
    vi.useFakeTimers()
    const users = Array.from({ length: 30 }, (_, index) => ({
      id: `u${index}`,
      email: `user-${index}@example.test`,
      display_name: `Пользователь ${index}`,
      username: `user-${index}`,
      active: true,
    }))
    setupUsers({ data: { users } }, '/users?keep=1&q=&q=duplicate&page=0')

    expect(screen.getByTestId('router-location')).toHaveTextContent('/users?keep=1')
    fireEvent.click(screen.getByRole('button', { name: 'Далее' }))
    expect(screen.getByTestId('router-location')).toHaveTextContent('/users?keep=1&page=2')
    fireEvent.change(screen.getByRole('textbox', { name: 'Поиск пользователей' }), {
      target: { value: 'Пользователь 29' },
    })
    act(() => vi.advanceTimersByTime(300))
    expect(screen.getByTestId('router-location')).toHaveTextContent(
      '/users?keep=1&q=%D0%9F%D0%BE%D0%BB%D1%8C%D0%B7%D0%BE%D0%B2%D0%B0%D1%82%D0%B5%D0%BB%D1%8C+29',
    )
    expect(screen.queryByRole('navigation', { name: 'Страницы профилей' })).not.toBeInTheDocument()
  })

  it('keeps pagination controls at the platform target size', () => {
    const users = Array.from({ length: 26 }, (_, index) => ({
      id: `u${index}`,
      email: `user-${index}@example.test`,
      display_name: `Пользователь ${index}`,
      username: `user-${index}`,
      active: true,
    }))
    setupUsers({ data: { users } })

    expect(screen.getByRole('button', { name: 'Назад' })).toHaveClass('min-h-10', 'sm:min-h-10')
    expect(screen.getByRole('button', { name: 'Далее' })).toHaveClass('min-h-10', 'sm:min-h-10')
  })
})
