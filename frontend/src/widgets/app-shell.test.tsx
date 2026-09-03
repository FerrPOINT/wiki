import { describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router'
import { ThemeProvider } from '@sdlc/ui/lib'

import { AppShell } from './app-shell'

const useCurrentUser = vi.hoisted(() => vi.fn())
const useLogout = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({
  useCurrentUser,
  useLogout,
}))

function mockHooks({
  isSystemAdmin = true,
  logoutMutate = vi.fn(),
}: {
  isSystemAdmin?: boolean
  logoutMutate?: ReturnType<typeof vi.fn>
} = {}) {
  useCurrentUser.mockReturnValue({
    data: {
      email: 'user@example.test',
      display_name: isSystemAdmin ? 'Администратор' : 'Пользователь',
      is_system_admin: isSystemAdmin,
    },
  })
  useLogout.mockReturnValue({ mutate: logoutMutate })
}

function renderShell(initialPath = '/') {
  return render(
    <ThemeProvider>
      <MemoryRouter initialEntries={[initialPath]}>
        <AppShell />
      </MemoryRouter>
    </ThemeProvider>,
  )
}

describe('AppShell', () => {
  it('shows only MVP navigation routes for a system admin', () => {
    mockHooks({ isSystemAdmin: true })

    renderShell()

    expect(screen.getByRole('link', { name: /обзор/i })).toHaveAttribute('href', '/')
    expect(screen.getByRole('link', { name: /пространства/i })).toHaveAttribute('href', '/spaces')
    expect(screen.getByRole('link', { name: /задачи/i })).toHaveAttribute('href', '/tasks')
    expect(screen.getByRole('link', { name: /фазы/i })).toHaveAttribute('href', '/phases')
    expect(screen.getByRole('link', { name: /материалы/i })).toHaveAttribute('href', '/evidence')
    expect(screen.getByRole('link', { name: /шаблоны/i })).toHaveAttribute('href', '/templates')
    expect(screen.getByRole('link', { name: /аудит/i })).toHaveAttribute('href', '/audit-log')
    expect(screen.getByRole('link', { name: /пользователи/i })).toHaveAttribute('href', '/users')
    expect(screen.getByRole('link', { name: /настройки/i })).toHaveAttribute('href', '/settings')
    expect(screen.getByRole('link', { name: /администрирование/i })).toHaveAttribute(
      'href',
      '/admin',
    )
  })

  it('hides admin navigation for regular users', () => {
    mockHooks({ isSystemAdmin: false })

    renderShell()

    expect(screen.queryByRole('link', { name: /аудит/i })).not.toBeInTheDocument()
    expect(screen.queryByRole('link', { name: /пользователи/i })).not.toBeInTheDocument()
    expect(screen.queryByRole('link', { name: /настройки/i })).not.toBeInTheDocument()
    expect(screen.queryByRole('link', { name: /администрирование/i })).not.toBeInTheDocument()
  })

  it('opens account menu and invokes logout', async () => {
    const user = userEvent.setup()
    const logoutMutate = vi.fn()
    mockHooks({ logoutMutate })

    renderShell()

    await user.click(screen.getByRole('button', { name: /аккаунт/i }))
    expect(await screen.findByText('user@example.test')).toBeInTheDocument()

    await user.click(screen.getByRole('menuitem', { name: /выйти/i }))
    expect(logoutMutate).toHaveBeenCalledTimes(1)
  })
})
