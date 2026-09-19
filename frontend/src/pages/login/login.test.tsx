import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router'
import { ThemeProvider } from '@sdlc/ui/lib'
import { LoginPage } from './'
import { useAuthStore } from '@/shared/auth/store'

const beginSso = vi.hoisted(() => vi.fn(async () => {}))
vi.mock('@sdlc/ui/sso', () => ({ beginSso }))

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
  beforeEach(() => {
    beginSso.mockClear()
    useAuthStore.getState().logout()
  })

  it('uses central SSO and has no password form', async () => {
    renderLogin()
    await waitFor(() =>
      expect(beginSso).toHaveBeenCalledWith(expect.objectContaining({ clientId: 'wiki' }), '/'),
    )
    expect(screen.getByRole('heading', { name: 'Вход в Wiki' })).toBeInTheDocument()
    expect(screen.queryByLabelText(/пароль/i)).not.toBeInTheDocument()
  })

  it('requires an explicit action after global logout', async () => {
    renderLogin('/login?logged_out=1')
    expect(beginSso).not.toHaveBeenCalled()
    await userEvent.click(screen.getByRole('button', { name: 'Войти через SDLC' }))
    expect(beginSso).toHaveBeenCalledTimes(1)
  })
})
