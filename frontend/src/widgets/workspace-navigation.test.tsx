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
  spaces: [
    {
      id: id(n + 10),
      key: `WIKI${n}`,
      name: `Пространство ${n}`,
      status: 'active',
      resource_status: 'connected',
    },
  ],
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
    const project = projects.find(
      (p) =>
        path.endsWith(p.namespace.namespace_id) || path.includes(`/spaces/${p.space_key}/projects`),
    )
    if (project)
      return path.includes('/namespace-contexts/') || path.includes('/spaces/')
        ? {
            space: project.spaces[0],
            projects: projects
              .filter((p) => p.spaces.some((space) => space.key === project.space_key))
              .map((p) => ({
                namespace: p.namespace,
                name: p.name,
                project_key: p.project_key,
                state: p.state,
                available: true,
              })),
          }
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
  it('resolves an explicit legacy space link without inheriting the previously open project', async () => {
    mount('/tasks?space=WIKI2')
    await screen.findByLabelText('Content URL')
    expect(screen.getByLabelText('Content URL')).not.toHaveTextContent('namespace_id')
    expect(screen.getByLabelText('Content URL')).toHaveTextContent('space=WIKI2')
  })
  it('keeps all groups and uses canonical project identity after section navigation', async () => {
    mount()
    const first = await screen.findByRole('region', { name: 'Одинаковый проект · A' })
    await userEvent.setup().click(within(first).getByRole('link', { name: 'Материалы' }))
    expect(await screen.findByLabelText('Content URL')).toHaveTextContent('space=WIKI1')
    expect(screen.getByLabelText('Content URL')).toHaveTextContent('project_scope=all')
    expect(screen.getByRole('region', { name: 'Одинаковый проект · B' })).toBeInTheDocument()
  })
  it.each(['documents', 'tasks', 'phases', 'evidence', 'search'])(
    'rejects a conflicting space in %s before mounting content',
    async (section) => {
      mount(`/workspaces/${registry}/${id(2)}/${section}?space=WIKI1`)
      expect(await screen.findByRole('alert')).toHaveTextContent('противоречивый')
      expect(screen.queryByLabelText('Content URL')).toBeNull()
      expect(screen.queryByRole('region', { name: 'Одинаковый проект · A' })).toBeNull()
    },
  )
  it('loads only the explicitly expanded document tree and remembers project folds', async () => {
    const view = mount()
    const user = userEvent.setup()
    await screen.findByRole('region', { name: 'Одинаковый проект · A' })
    expect(requests.filter((path) => path.includes('/tree'))).toHaveLength(0)
    await user.click(
      screen.getByRole('button', { name: 'Дерево документов: Пространство 1 · WIKI1' }),
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
      items: [
        {
          ...projects[1],
          spaces: [],
          binding: null,
          space_key: null,
          resource_status: 'not_connected',
        },
      ],
      total: 1,
    })
    mount()
    await screen.findByRole('region', { name: 'Одинаковый проект · B' })
    expect(screen.getByText('Wiki не подключена')).toBeVisible()
    expect(screen.queryByRole('link', { name: 'Задачи' })).toBeNull()
  })
  it('requires an explicit space when the project has several and preserves that choice', async () => {
    const base = vi.mocked(apiRequest).getMockImplementation()!
    vi.mocked(apiRequest).mockImplementation(async (path, options) => {
      if (path === `/api/v1/workspace-projects/${registry}/${id(1)}`)
        return { ...projects[0], spaces: [...projects[0]!.spaces, ...projects[1]!.spaces] }
      if (path === '/api/v1/spaces/WIKI2/projects')
        return { space: projects[1]!.spaces[0], projects: [{ namespace: projects[0]!.namespace }] }
      return base(path, options)
    })
    mount(`/workspaces/${registry}/${id(1)}/documents`)
    expect(
      await screen.findByRole('heading', { name: 'Выберите пространство проекта' }),
    ).toBeVisible()
    expect(screen.queryByLabelText('Content URL')).toBeNull()
    expect(requests.filter((path) => path.includes('/tree'))).toHaveLength(0)
    await userEvent.setup().selectOptions(screen.getByLabelText('Пространство'), 'WIKI2')
    await waitFor(() =>
      expect(screen.getByLabelText('Content URL')).toHaveTextContent('space=WIKI2'),
    )
    expect(screen.getByLabelText('Content URL')).toHaveTextContent(id(1))
  })
  it('does not select a neighboring healthy space when another association is unavailable', async () => {
    const base = vi.mocked(apiRequest).getMockImplementation()!
    vi.mocked(apiRequest).mockImplementation(async (path, options) =>
      path === `/api/v1/workspace-projects/${registry}/${id(1)}`
        ? {
            ...projects[0],
            spaces: [
              ...projects[0]!.spaces,
              { ...projects[1]!.spaces[0], resource_status: 'unavailable' },
            ],
          }
        : base(path, options),
    )
    mount(`/workspaces/${registry}/${id(1)}/documents`)
    await screen.findByRole('heading', { name: 'Выберите пространство проекта' })
    expect(screen.queryByLabelText('Content URL')).toBeNull()
    expect(
      screen.getByRole('option', { name: 'Пространство 2 · WIKI2 · Недоступно' }),
    ).toBeDisabled()
    expect(requests.filter((path) => path.includes('/tree'))).toHaveLength(0)
  })
  it('opens the exact historical revision after project detach and clears its obsolete project context', async () => {
    const base = vi.mocked(apiRequest).getMockImplementation()!
    vi.mocked(apiRequest).mockImplementation(async (path, options) =>
      path === `/api/v1/documents/${id(50)}/space-context`
        ? { space: projects[0]!.spaces[0], projects: [] }
        : base(path, options),
    )
    mount(
      `/documents/${id(50)}/revisions/${id(60)}?registry_instance_id=${registry}&namespace_id=${id(1)}&task_id=${id(70)}&tracker_instance_id=${instance}`,
    )
    await waitFor(() =>
      expect(screen.getByLabelText('Content URL')).toHaveTextContent('project_scope=all'),
    )
    const url = screen.getByLabelText('Content URL').textContent!
    expect(url).toContain(`/documents/${id(50)}/revisions/${id(60)}`)
    expect(url).not.toContain('namespace_id')
    expect(url).not.toContain('task_id')
    expect(requests.filter((path) => path.includes('/tree'))).toHaveLength(0)
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
