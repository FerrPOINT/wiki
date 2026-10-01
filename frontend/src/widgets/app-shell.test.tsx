import { afterEach, describe, expect, it, vi } from 'vitest'
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react'
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
  vi.stubGlobal(
    'matchMedia',
    vi.fn(() => ({
      matches: false,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    })),
  )
  useCurrentUser.mockReturnValue({
    data: {
      email: 'user@example.test',
      display_name: isSystemAdmin ? 'Администратор' : 'Пользователь',
      is_system_admin: isSystemAdmin,
    },
  })
  useLogout.mockReturnValue({ mutate: logoutMutate })
}

afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
})

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

    expect(screen.getByRole('link', { name: 'Wiki' })).toHaveAttribute('href', '/')
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

  it('opens account menu and invokes logout', () => {
    const logoutMutate = vi.fn()
    mockHooks({ logoutMutate })

    renderShell()

    expect(screen.queryByText('user@example.test')).not.toBeInTheDocument()
    fireEvent.keyDown(screen.getByRole('button', { name: /аккаунт/i }), { key: 'ArrowDown' })
    const menu = screen.getByRole('menu', { hidden: true })
    expect(within(menu).getAllByText('user@example.test')).toHaveLength(1)

    fireEvent.click(within(menu).getByRole('menuitem', { name: /выйти/i }))
    expect(logoutMutate).toHaveBeenCalledTimes(1)
  })

  it('owns one global header above the offset page and navigation-only sidebar', () => {
    mockHooks()
    const { container } = renderShell()
    const header = screen.getByRole('banner')
    expect(container.querySelectorAll('[data-platform-header]')).toHaveLength(1)
    expect(header.parentElement).toBe(container.firstElementChild)
    expect(
      [...header.querySelectorAll('[data-platform-header-slot]')].map((slot) =>
        slot.getAttribute('data-platform-header-slot'),
      ),
    ).toEqual(['leading', 'services', 'actions'])
    expect(
      within(header).getByRole('button', { name: 'Открыть список сервисов: Wiki' }),
    ).toBeInTheDocument()
    expect(within(header).getByRole('link', { name: 'Новый документ' })).toHaveAttribute(
      'href',
      '/documents/new',
    )
    expect(container.querySelector('aside')).not.toHaveTextContent('Wiki')
    expect(screen.queryByText('База знаний')).not.toBeInTheDocument()
  })

  it.each([undefined, '', '   ', 'user@example.test'])(
    'does not duplicate email for identity %s',
    (name) => {
      mockHooks()
      useCurrentUser.mockReturnValue({ data: { email: 'user@example.test', display_name: name } })
      renderShell()
      fireEvent.keyDown(screen.getByRole('button', { name: 'Аккаунт' }), { key: 'ArrowDown' })
      expect(
        within(screen.getByRole('menu', { hidden: true })).getAllByText('user@example.test'),
      ).toHaveLength(1)
    },
  )

  it('keeps long identity in a bounded account menu and supports missing user data', () => {
    mockHooks()
    const name = 'long-account-name-that-must-not-expand-the-platform-header@example.test'
    useCurrentUser.mockReturnValue({ data: { display_name: name } })
    const { rerender } = renderShell()
    expect(screen.getByRole('banner')).not.toHaveTextContent(name)
    fireEvent.keyDown(screen.getByRole('button', { name: 'Аккаунт' }), { key: 'ArrowDown' })
    const menu = screen.getByRole('menu', { hidden: true })
    expect(within(menu).getByText(name)).toHaveClass('break-words')
    expect(menu).toHaveClass('max-w-[calc(100vw-2rem)]')
    fireEvent.keyDown(document, { key: 'Escape' })
    useCurrentUser.mockReturnValue({ data: undefined })
    rerender(
      <ThemeProvider>
        <MemoryRouter>
          <AppShell />
        </MemoryRouter>
      </ThemeProvider>,
    )
    fireEvent.keyDown(screen.getByRole('button', { name: 'Аккаунт' }), { key: 'ArrowDown' })
    expect(
      within(screen.getByRole('menu', { hidden: true })).getByText('Пользователь'),
    ).toBeInTheDocument()
  })

  it('preserves mobile creation and closes the drawer after following its link', async () => {
    mockHooks()
    renderShell()
    const trigger = screen.getByRole('button', { name: 'Открыть навигацию' })
    fireEvent.click(trigger)
    const dialog = await screen.findByRole('dialog', { name: 'Навигация Wiki' })
    const create = within(dialog).getByRole('link', { name: 'Новый документ' })
    expect(create).toHaveAttribute('href', '/documents/new')
    fireEvent.click(create)
    await waitFor(() => expect(screen.queryByRole('dialog')).not.toBeInTheDocument())
    expect(document.querySelector('[data-page-layout]')).toHaveAttribute(
      'data-page-layout',
      'reading',
    )
    expect(trigger).toHaveFocus()
  })

  it('closes the drawer on desktop resize and removes its listener on unmount', async () => {
    mockHooks()
    const media = { matches: false, addEventListener: vi.fn(), removeEventListener: vi.fn() }
    vi.stubGlobal(
      'matchMedia',
      vi.fn(() => media),
    )
    const { unmount } = renderShell()
    fireEvent.click(screen.getByRole('button', { name: 'Открыть навигацию' }))
    await screen.findByRole('dialog')
    const listener = media.addEventListener.mock.calls[0]?.[1] as () => void
    expect(listener).toBeTypeOf('function')
    media.matches = true
    act(() => listener())
    await waitFor(() => expect(screen.queryByRole('dialog')).not.toBeInTheDocument())
    expect(document.body).not.toHaveStyle({ pointerEvents: 'none' })
    unmount()
    expect(media.removeEventListener).toHaveBeenCalledWith('change', listener)
  })

  it('marks a parent section active on a direct nested route', () => {
    mockHooks()

    renderShell('/tasks/BASE-42')

    expect(screen.getByRole('link', { name: 'Задачи' })).toHaveAttribute('aria-current', 'page')
    expect(screen.getByRole('link', { name: 'Обзор' })).not.toHaveAttribute('aria-current')
  })

  it.each([
    ['/documents/document-1', 'detail-with-aside'],
    ['/documents/new', 'reading'],
    ['/settings', 'reading'],
    ['/tasks/BASE-42', 'detail-with-aside'],
    ['/phases/implementation', 'wide'],
    ['/tasks', 'wide'],
    ['/phases', 'wide'],
    ['/spaces', 'wide'],
  ])('uses the semantic mode %s -> %s', (route, mode) => {
    mockHooks()
    const { container } = renderShell(route)
    expect(container.querySelector('[data-page-layout]')).toHaveAttribute('data-page-layout', mode)
  })

  it('keeps focus inside the mobile drawer and restores it after Escape', async () => {
    const user = userEvent.setup()
    mockHooks()
    renderShell('/tasks/BASE-42')
    const trigger = screen.getByRole('button', { name: 'Открыть навигацию' })

    await user.click(trigger)

    const dialog = await screen.findByRole('dialog', { name: 'Навигация Wiki' })
    expect(within(dialog).getByRole('navigation', { name: 'Основная навигация' })).toBeVisible()
    expect(dialog.contains(document.activeElement)).toBe(true)

    await user.keyboard('{Escape}')

    await waitFor(() =>
      expect(screen.queryByRole('dialog', { name: 'Навигация Wiki' })).not.toBeInTheDocument(),
    )
    expect(trigger).toHaveFocus()
  })
})
