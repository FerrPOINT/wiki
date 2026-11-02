import { cleanup, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router'
import { afterEach, beforeEach, expect, it, vi } from 'vitest'
import { DocumentRevisionPage } from './index'

const source = vi.hoisted(() => ({ denied: false }))

vi.mock('@/shared/api/hooks', () => ({
  useDocument: () => ({ isPending: false, isError: false, data: { space_key: 'DOCS' } }),
  useDocumentRevision: () => ({
    isPending: false,
    isError: false,
    data: {
      title: 'Published revision',
      version: 3,
      published_at: '2026-10-27T00:00:00Z',
      body_html:
        '<table><caption>Amounts</caption><tbody><tr><td>1</td></tr></tbody></table><pre><code>long-command</code></pre>',
    },
  }),
}))
vi.mock('@/widgets/namespace-context', () => ({
  useNamespaceContext: () => ({
    ref: source.denied
      ? {
          registry_instance_id: '11111111-1111-4111-8111-111111111111',
          namespace_id: '22222222-2222-4222-8222-222222222222',
        }
      : null,
    malformed: false,
    query: { isPending: false, isError: false, data: { resource_key: 'OTHER' } },
  }),
}))
vi.mock('@/features/managed-links/TaskRevisionLink', () => ({ TaskRevisionLink: () => null }))

beforeEach(() => {
  source.denied = false
})
afterEach(cleanup)

function RevisionRoute() {
  return (
    <MemoryRouter initialEntries={['/documents/doc/revisions/revision']}>
      <Routes>
        <Route
          path="/documents/:documentId/revisions/:revisionId"
          element={<DocumentRevisionPage />}
        />
      </Routes>
    </MemoryRouter>
  )
}

function mount() {
  return render(<RevisionRoute />)
}

it('makes published revision HTML keyboard scrollable through the real page', async () => {
  const page = mount()
  expect(screen.getByRole('heading', { name: 'Published revision' })).toBeInTheDocument()
  await waitFor(() => {
    expect(screen.getByRole('table', { name: 'Amounts' })).toHaveAttribute('tabindex', '0')
    expect(page.container.querySelector('pre')).toHaveAttribute('tabindex', '0')
  })
})

it('keeps content hidden when the requested namespace belongs to another space', () => {
  source.denied = true
  const page = mount()
  expect(screen.getByText('Ревизия недоступна в выбранном проекте.')).toBeInTheDocument()
  expect(page.container.querySelector('pre, table, .base-prose')).toBeNull()
})

it('preserves the code node, tab stop and focus across equal HTML re-renders', () => {
  const page = mount()
  const code = page.container.querySelector('pre')!
  code.focus()
  page.rerender(<RevisionRoute />)
  expect(page.container.querySelector('pre')).toBe(code)
  expect(code).toHaveAttribute('tabindex', '0')
  expect(code).toHaveFocus()
})
