import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { MemoryRouter } from 'react-router'

import { LoginPage } from './'
import { useAuthStore } from '@/shared/auth/store'
import { ThemeProvider } from '@/shared/lib/theme'

const login = vi.hoisted(() => vi.fn())
const register = vi.hoisted(() => vi.fn())
const getCurrentUser = vi.hoisted(() => vi.fn())
const logout = vi.hoisted(() => vi.fn())
vi.mock('@/api/auth', () => ({ login, register, getCurrentUser, logout }))

function wrapper(children: React.ReactNode) {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
  return (
    <ThemeProvider>
      <QueryClientProvider client={qc}>
        <MemoryRouter>{children}</MemoryRouter>
      </QueryClientProvider>
    </ThemeProvider>
  )
}

describe('LoginPage', () => {
  beforeEach(() => {
    useAuthStore.setState({
      token: null,
      userId: null,
      email: null,
      username: null,
      displayName: null,
    })
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('renders login form and submits', async () => {
    login.mockResolvedValueOnce({
      access_token: 'tok',
      user_id: 'u1',
      email: 'admin@example.com',
    })

    render(wrapper(<LoginPage />))
    expect(screen.getByText('Wiki')).toBeInTheDocument()

    const email = screen.getByLabelText(/email/i) as HTMLInputElement
    expect(email).toHaveValue('')
    await userEvent.type(email, 'admin@example.com')
    const password = screen.getByLabelText(/пароль|password/i) as HTMLInputElement
    expect(password).toHaveValue('')
    await userEvent.type(password, 'correct-horse-battery-staple')

    const submit = screen.getByRole('button', { name: /войти|Log in/i })
    await userEvent.click(submit)

    await waitFor(() =>
      expect(login).toHaveBeenCalledWith(
        {
          email: 'admin@example.com',
          password: 'correct-horse-battery-staple',
        },
        expect.anything(),
      ),
    )
    expect(login.mock.calls[0]?.[0]).toEqual({
      email: 'admin@example.com',
      password: 'correct-horse-battery-staple',
    })
  })
})
