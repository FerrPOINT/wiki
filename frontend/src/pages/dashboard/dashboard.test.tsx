import { describe, expect, it } from 'vitest'
import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router'

import { DashboardPage } from './'

function wrapper(children: React.ReactNode) {
  return <MemoryRouter>{children}</MemoryRouter>
}

describe('DashboardPage', () => {
  it('renders the wiki overview and document actions', () => {
    render(wrapper(<DashboardPage />))

    expect(screen.getByRole('heading', { name: 'Wiki' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: /новый документ/i })).toHaveAttribute(
      'href',
      '/documents/new',
    )
    expect(screen.getByText('Последние документы')).toBeInTheDocument()
    expect(screen.getByText('Нужно закрыть')).toBeInTheDocument()
  })
})
