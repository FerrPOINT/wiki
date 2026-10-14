import { cleanup, render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { MemoryRouter } from 'react-router'
import { afterEach, expect, it, vi } from 'vitest'
import { apiRequest } from '@/api/client'
import { SpaceProjectsEditor } from './SpaceProjectsEditor'

vi.mock('@/api/client', () => ({ apiRequest: vi.fn() }))
afterEach(() => {
  cleanup()
  vi.resetAllMocks()
})
const registry = '11111111-1111-4111-8111-111111111111'
const projects = [1, 2].map((n) => ({
  namespace: {
    registry_instance_id: registry,
    namespace_id: `22222222-2222-4222-8222-${String(n).padStart(12, '0')}`,
  },
  name: 'Общий проект',
  project_key: `P${n}`,
  state: 'active',
  available: true,
  stale: false,
}))
function mount(canManage = true) {
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <MemoryRouter>
        <SpaceProjectsEditor spaceKey="SHARED" canManage={canManage} />
        <a href="/documents/kept-id">Существующий документ</a>
      </MemoryRouter>
    </QueryClientProvider>,
  )
}
it('adds two distinct projects, removes the last association and keeps the document destination', async () => {
  let linked: typeof projects = []
  const writes: string[] = []
  vi.mocked(apiRequest).mockImplementation(async (path, options) => {
    if (path.startsWith('/api/v1/workspace-projects?')) return { items: projects, total: 2 }
    if (options?.method === 'PUT' || options?.method === 'DELETE') {
      writes.push(options.method + ' ' + path)
      const project = projects.find((item) => path.endsWith(item.namespace.namespace_id))!
      linked =
        options.method === 'PUT' ? [...linked, project] : linked.filter((item) => item !== project)
      return undefined
    }
    return {
      space: { id: 'space-id', key: 'SHARED', name: 'Shared', status: 'active' },
      projects: linked,
    }
  })
  mount()
  const user = userEvent.setup()
  expect(await screen.findByText('Самостоятельное пространство · проектов нет')).toBeVisible()
  for (const project of projects) {
    await user.selectOptions(
      screen.getByLabelText('Добавить проект'),
      `${registry}/${project.namespace.namespace_id}`,
    )
    await user.click(screen.getByRole('button', { name: 'Добавить' }))
    await screen.findByRole('link', { name: `${project.name} · ${project.project_key}` })
  }
  expect(screen.getAllByRole('button', { name: 'Отвязать проект Общий проект' })).toHaveLength(2)
  await user.click(screen.getAllByRole('button', { name: 'Отвязать проект Общий проект' })[0]!)
  await waitFor(() =>
    expect(screen.getAllByRole('button', { name: 'Отвязать проект Общий проект' })).toHaveLength(1),
  )
  await user.click(screen.getByRole('button', { name: 'Отвязать проект Общий проект' }))
  expect(await screen.findByText('Самостоятельное пространство · проектов нет')).toBeVisible()
  expect(screen.getByRole('link', { name: 'Существующий документ' })).toHaveAttribute(
    'href',
    '/documents/kept-id',
  )
  expect(writes).toEqual(
    projects
      .map(
        (project) =>
          `PUT /api/v1/spaces/SHARED/projects/${registry}/${project.namespace.namespace_id}`,
      )
      .concat(
        projects.map(
          (project) =>
            `DELETE /api/v1/spaces/SHARED/projects/${registry}/${project.namespace.namespace_id}`,
        ),
      ),
  )
})
it('shows links to a viewer without offering link mutations or requesting the external catalog', async () => {
  vi.mocked(apiRequest).mockResolvedValue({ space: { key: 'SHARED', status: 'active' }, projects })
  mount(false)
  await screen.findByRole('link', { name: 'Общий проект · P1' })
  expect(screen.queryByLabelText('Добавить проект')).toBeNull()
  expect(screen.queryByRole('button', { name: /Отвязать/ })).toBeNull()
  expect(apiRequest).toHaveBeenCalledTimes(1)
})
