import { fireEvent, render, screen } from '@testing-library/react'
import { createMemoryRouter, RouterProvider } from 'react-router'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { DocumentComposePage } from './'

const context = vi.hoisted(() => vi.fn())
const mutate = vi.hoisted(() => vi.fn())
vi.mock('@/widgets/namespace-context', () => ({ useNamespaceContext: context }))
vi.mock('@/shared/api/hooks', () => ({
  useSpaces: () => ({ data: { spaces: [] }, isLoading: false, isError: false }),
  useTemplates: () => ({ data: { templates: [] } }),
  useCreateDocument: () => ({ mutate, isPending: false, isError: false }),
}))

function mount(
  overrides: { conflict?: boolean; status?: string; pending?: boolean; error?: boolean } = {},
) {
  context.mockReturnValue({
    ref: { registry_instance_id: 'registry', namespace_id: 'project' },
    malformed: false,
    conflict: overrides.conflict ?? false,
    query: {
      data: { resource_key: 'PROJECT_DOCS', status: overrides.status ?? 'active' },
      isPending: overrides.pending ?? false,
      isError: overrides.error ?? false,
    },
  })
  render(
    <RouterProvider
      router={createMemoryRouter([{ path: '/documents/new', element: <DocumentComposePage /> }], {
        initialEntries: ['/documents/new?space=OTHER'],
      })}
    />,
  )
  fireEvent.change(screen.getByLabelText('Название'), { target: { value: 'Keep this draft' } })
  fireEvent.change(screen.getByLabelText('Markdown документа'), {
    target: { value: '# Keep this content' },
  })
}

describe('authoritative project space after project-association integration', () => {
  beforeEach(() => vi.clearAllMocks())

  it('uses the confirmed active context independently of the global catalogue', () => {
    mount()
    expect(screen.getByLabelText('Пространство')).toHaveValue('PROJECT_DOCS')
    fireEvent.click(screen.getByRole('button', { name: 'Сохранить черновик' }))
    expect(mutate).toHaveBeenCalledWith(
      expect.objectContaining({ spaceKey: 'PROJECT_DOCS' }),
      expect.any(Object),
    )
  })

  it.each([{ conflict: true }, { status: 'archived' }, { pending: true }, { error: true }])(
    'blocks unconfirmed or conflicting context without losing input: %j',
    (state) => {
      mount(state)
      expect(screen.getByRole('button', { name: 'Сохранить черновик' })).toBeDisabled()
      expect(screen.getByLabelText('Название')).toHaveValue('Keep this draft')
      expect(mutate).not.toHaveBeenCalled()
    },
  )
})
