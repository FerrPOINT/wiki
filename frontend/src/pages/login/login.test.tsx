import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router'
import { ThemeProvider } from '@sdlc/ui/lib'
import { LoginPage } from './'
import { useAuthStore } from '@/shared/auth/store'
import { SsoLogoutPendingError } from '@sdlc/ui/sso'

const beginSso = vi.hoisted(() => vi.fn(async () => {}))
vi.mock('@sdlc/ui/sso', async (importOriginal) => ({
  ...(await importOriginal<typeof import('@sdlc/ui/sso')>()),
  beginSso,
}))

function renderLogin(path = '/login') {
  return render(
    <ThemeProvider>
      <MemoryRouter initialEntries={[path]}>
        <LoginPage />
      </MemoryRouter>
    </ThemeProvider>,
  )
}

describe('Wiki login', () => {
  it('presents one platform login without product branding or legacy warnings', async () => {
    renderLogin('/login?logged_out=1')
    expect(await screen.findByRole('heading', { name: 'Вход в платформу', exact: true })).toBeInTheDocument()
    expect(screen.queryByText(/Base|SDLC|Task Tracker|Fleet Control|Wiki|CI[/]CD|Admin Panel|второй фактор|защита входа/i)).not.toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Войти через SSO', exact: true })).toBeInTheDocument()
  })

  beforeEach(() => {
    beginSso.mockReset().mockResolvedValue(undefined)
    useAuthStore.getState().logout()
  })

  it('uses central SSO and has no password form', async () => {
    renderLogin()
    await waitFor(() =>
      expect(beginSso).toHaveBeenCalledWith(expect.objectContaining({ clientId: 'wiki' }), '/'),
    )
    expect(screen.getByRole('heading', { name: 'Вход в платформу' })).toBeInTheDocument()
    expect(screen.queryByLabelText(/пароль/i)).not.toBeInTheDocument()
  })

  it('requires an explicit action after global logout', async () => {
    renderLogin('/login?logged_out=1')
    expect(beginSso).not.toHaveBeenCalled()
    await userEvent.click(screen.getByRole('button', { name: 'Войти через SSO' }))
    expect(beginSso).toHaveBeenCalledTimes(1)
    expect(beginSso).toHaveBeenCalledWith(expect.objectContaining({ clientId: 'wiki' }), '/', {
      interactive: true,
    })
  })

  it.each([new SsoLogoutPendingError(), new DOMException('Cancelled', 'AbortError')])(
    'does not report interrupted automatic navigation as an auth failure: %s',
    async (error) => {
      beginSso.mockRejectedValueOnce(error)
      renderLogin()
      await waitFor(() => expect(beginSso).toHaveBeenCalledOnce())
      await userEvent.click(screen.getByRole('button', { name: 'Войти через SSO' }))
      expect(screen.queryByRole('alert')).not.toBeInTheDocument()
      expect(beginSso).toHaveBeenLastCalledWith(
        expect.objectContaining({ clientId: 'wiki' }),
        '/',
        { interactive: true },
      )
    },
  )

  it('shows real auth failures and clears them on an explicit retry', async () => {
    beginSso.mockRejectedValueOnce(new Error('private auth details'))
    renderLogin('/login?logged_out=1')
    await userEvent.click(screen.getByRole('button', { name: 'Войти через SSO' }))
    expect(await screen.findByRole('alert')).toHaveTextContent('Central Auth временно недоступен.')
    expect(screen.queryByText(/private auth details/)).not.toBeInTheDocument()
    await userEvent.click(screen.getByRole('button', { name: 'Войти через SSO' }))
    await waitFor(() => expect(screen.queryByRole('alert')).not.toBeInTheDocument())
  })
})
