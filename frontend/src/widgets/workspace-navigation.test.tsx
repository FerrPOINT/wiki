import { cleanup, render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { MemoryRouter, Outlet, Route, Routes, useLocation } from 'react-router'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { apiRequest } from '@/api/client'
import { useProjectDisclosures } from '@sdlc/ui/lib'
import { NamespaceShellContext } from './namespace-context'
import { ProjectBoundary } from './project-boundary'
import { useDocumentDisclosures, WorkspaceNavigation } from './workspace-navigation'
vi.mock('@/api/client', () => ({ apiRequest: vi.fn() }))
const registry = '11111111-1111-4111-8111-111111111111'
const instance = '22222222-2222-4222-8222-222222222222'
const id = (n: number) => `33333333-3333-4333-8333-${String(n).padStart(12, '0')}`
const projects = [1, 2].map((n) => ({
  namespace: { registry_instance_id: registry, namespace_id: id(n) },
  tracker: { instance_id: instance, resource_id: id(n), kind: 'tracker_project' },
  name: 'Одинаковый проект',
  project_key: n === 1 ? 'A' : 'B',
  generation: 1,
  state: 'active',
  stale: false,
  observed_at: '2026-10-12T00:00:00Z',
  resource_status: 'connected',
  space_key: `WIKI${n}`,
  binding: {
    schema_version: 1,
    namespace: { registry_instance_id: registry, namespace_id: id(n) },
    resource: { kind: 'wiki_space', instance_id: instance, resource_id: id(n + 10) },
    generation: 1,
    state: 'active',
    operation_id: id(n + 20),
    drained: true,
  },
}))
const requests: string[] = []
function Content() {
  const location = useLocation()
  return <output aria-label="Content URL">{location.pathname + location.search}</output>
}
function Shell() {
  const disclosures = useProjectDisclosures('wiki:collapsed-projects:v1'),
    documents = useDocumentDisclosures()
  return (
    <>
      <NamespaceShellContext />
      <WorkspaceNavigation {...{ disclosures, documents }} />
      <ProjectBoundary>
        <Outlet />
      </ProjectBoundary>
    </>
  )
}
function mount(path = '/projects?project_scope=all') {
  return render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <MemoryRouter initialEntries={[path]}>
        <Routes>
          <Route element={<Shell />}>
            <Route path="*" element={<Content />} />
          </Route>
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>,
  )
}
beforeEach(() => {
  localStorage.clear()
  requests.length = 0
  vi.stubEnv('VITE_NAMESPACE_ENABLED', 'true')
  vi.mocked(apiRequest).mockImplementation(async (path) => {
    requests.push(path)
    if (path.includes('/tree'))
      return {
        documents: [
          {
            id: id(50),
            slug: 'document',
            title: 'Документ',
            document_type: 'page',
            status: 'published',
            children: [],
          },
        ],
      }
    if (path.startsWith('/api/v1/workspace-projects?'))
      return { items: projects, total: projects.length }
    const project = projects.find((p) => path.endsWith(p.namespace.namespace_id)||path.includes(`/spaces/${p.space_key}/namespace-context`))
    if (project)
      return path.includes('/namespace-contexts/')||path.includes('/spaces/')
        ? { binding: project.binding, label: project.name, resource_key: project.space_key }
        : project
    throw new Error('Unexpected request ' + path)
  })
})
afterEach(() => {
  cleanup()
  vi.unstubAllEnvs()
  vi.resetAllMocks()
})
describe('Wiki project navigation', () => {
  it('resolves an explicit legacy space link without inheriting the previously open project',async()=>{
    mount('/tasks?space=WIKI2')
    await waitFor(()=>expect(screen.getByLabelText('Content URL')).toHaveTextContent(id(2)))
    expect(screen.getByLabelText('Content URL')).toHaveTextContent('project_scope=all')
    expect(screen.getByLabelText('Content URL')).toHaveTextContent('space=WIKI2')
  })
  it('keeps all groups and uses canonical project identity after section navigation', async () => {
    mount()
    const first = await screen.findByRole('region', { name: 'Одинаковый проект · A' })
    await userEvent.setup().click(within(first).getByRole('link', { name: 'Задачи' }))
    expect(await screen.findByLabelText('Content URL')).toHaveTextContent('space=WIKI1')
    expect(screen.getByLabelText('Content URL')).toHaveTextContent('project_scope=all')
    expect(screen.getByRole('region', { name: 'Одинаковый проект · B' })).toBeInTheDocument()
  })
  it('resolves a selected project before mounting content and rejects a conflicting space', async () => {
    mount(`/workspaces/${registry}/${id(2)}/tasks?space=WIKI1`)
    expect(await screen.findByRole('alert')).toHaveTextContent('противоречивый')
    expect(screen.queryByLabelText('Content URL')).toBeNull()
    expect(screen.queryByRole('region', { name: 'Одинаковый проект · A' })).toBeNull()
  })
  it('loads only the explicitly expanded document tree and remembers project folds', async () => {
    const view = mount()
    const user = userEvent.setup()
    await screen.findByRole('region', { name: 'Одинаковый проект · A' })
    expect(requests.filter((path) => path.includes('/tree'))).toHaveLength(0)
    await user.click(
      screen.getByRole('button', { name: 'Дерево документов: Одинаковый проект · A' }),
    )
    await screen.findByRole('link', { name: 'Документ' })
    expect(requests.filter((path) => path.includes('/tree'))).toEqual(['/api/v1/spaces/WIKI1/tree'])
    await user.click(screen.getByRole('button', { name: 'Одинаковый проект · B' }))
    await waitFor(() => expect(localStorage.getItem('wiki:collapsed-projects:v1')).toContain(id(2)))
    view.unmount()
    mount()
    expect(await screen.findByRole('button', { name: 'Одинаковый проект · B' })).toHaveAttribute(
      'aria-expanded',
      'false',
    )
  })
  it('keeps a project without Wiki visible without offering foreign content', async () => {
    vi.mocked(apiRequest).mockResolvedValue({
      items: [{ ...projects[1], binding: null, space_key: null, resource_status: 'not_connected' }],
      total: 1,
    })
    mount()
    await screen.findByRole('region', { name: 'Одинаковый проект · B' })
    expect(screen.getByText('Wiki не подключена')).toBeVisible()
    expect(screen.queryByRole('link', { name: 'Задачи' })).toBeNull()
  })
  it('reads beyond the first 100 projects for picker and navigation', async () => {
    const many = Array.from({ length: 101 }, (_, n) => ({
      ...projects[0],
      namespace: { registry_instance_id: registry, namespace_id: id(n + 100) },
      tracker: { ...projects[0]!.tracker, resource_id: id(n + 200) },
      project_key: `P${n}`,
    }))
    vi.mocked(apiRequest).mockImplementation(async (path) =>
      path.includes('offset=100')
        ? { items: [many[100]], total: 101 }
        : { items: many.slice(0, 100), total: 101 },
    )
    mount()
    await screen.findByRole('button', { name: 'Одинаковый проект · P100' })
    expect(screen.getAllByRole('region')).toHaveLength(101)
  })
})
