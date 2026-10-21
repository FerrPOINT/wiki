import { findChoiceOption } from '../../test/choice'
import { chooseValue } from '../../test/choice'
import { cleanup, render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { MemoryRouter } from 'react-router'
import { afterEach, expect, it, vi } from 'vitest'
import { apiRequest } from '@/api/client'
import { TaskRevisionLink } from './TaskRevisionLink'

vi.mock('@/api/client', () => ({ apiRequest: vi.fn() }))
afterEach(() => {
  cleanup()
  vi.resetAllMocks()
  vi.unstubAllEnvs()
})
it('requires a project choice and records the task project when a shared space has two projects', async () => {
  vi.stubEnv('VITE_NAMESPACE_ENABLED', 'false')
  const registry = '11111111-1111-4111-8111-111111111111'
  const projects = ['first', 'second'].map((name, index) => ({
    name,
    project_key: name,
    namespace: {
      registry_instance_id: registry,
      namespace_id: `22222222-2222-4222-8222-${String(index + 1).padStart(12, '0')}`,
    },
    available: true,
    state: 'active',
  }))
  const task = {
    tracker_instance_id: '33333333-3333-4333-8333-333333333333',
    task_id: '44444444-4444-4444-8444-444444444444',
  }
  vi.mocked(apiRequest).mockImplementation(async (path) =>
    path.includes('/available-tasks?')
      ? [{ task, namespace: projects[1]!.namespace, task_key: 'SECOND-1', label: 'Task' }]
      : { space: { key: 'SHARED', status: 'active' }, projects },
  )
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <MemoryRouter>
        <TaskRevisionLink documentId="doc-id" revisionId="revision-id" spaceKey="SHARED" />
      </MemoryRouter>
    </QueryClientProvider>,
  )
  await screen.findByLabelText('Проект задачи')
  expect(vi.mocked(apiRequest).mock.calls.some(([path]) => path.includes('/available-tasks'))).toBe(
    false,
  )
  const user = userEvent.setup()
  await chooseValue(
    screen.getByLabelText('Проект задачи'),
    `${registry}/${projects[1]!.namespace.namespace_id}`,
  )
  await findChoiceOption(await screen.findByLabelText('Задача проекта'), 'SECOND-1 · Task')
  await chooseValue(
    screen.getByLabelText('Задача проекта'),
    `${task.tracker_instance_id}/${task.task_id}`,
  )
  await user.click(screen.getByRole('button', { name: 'Связать ревизию' }))
  await waitFor(() =>
    expect(apiRequest).toHaveBeenCalledWith('/api/v1/spaces/SHARED/managed-task-links', {
      method: 'POST',
      body: {
        namespace: projects[1]!.namespace,
        task,
        document_id: 'doc-id',
        revision_id: 'revision-id',
      },
    }),
  )
})
