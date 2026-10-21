import { useState, type FormEvent } from 'react'
import { useSearchParams } from 'react-router'
import { Select, Button, Label } from '@pdlc/ui/ui'
import { useQuery } from '@tanstack/react-query'
import { apiRequest } from '@/api/client'
import { useNamespaceContext } from '@/widgets/namespace-context'
import { useSpaceProjects } from '@/api/workspaces'
import { projectIdentity } from '@pdlc/ui/lib'
import type { components } from '@/api/generated'
import { useAuthStore } from '@/shared/auth/store'

export function TaskRevisionLink({
  documentId,
  revisionId,
  spaceKey,
}: {
  documentId: string
  revisionId: string
  spaceKey: string
}) {
  const [params] = useSearchParams()
  const context = useNamespaceContext()
  const actor = useAuthStore((state) => state.userId)
  const projects = useSpaceProjects(spaceKey)
  const [projectChoice, setProjectChoice] = useState('')
  const available =
    projects.data?.projects.filter((project) => project.state === 'active' && project.available) ??
    []
  const preferred = context.ref
    ? available.find(
        (project) => projectIdentity(project.namespace) === projectIdentity(context.ref!),
      )
    : available.length === 1
      ? available[0]
      : undefined
  const ref =
    available.find((project) => projectIdentity(project.namespace) === projectChoice)?.namespace ??
    preferred?.namespace
  const [busy, setBusy] = useState(false)
  const [message, setMessage] = useState('')
  const [offset, setOffset] = useState(0)
  const [selected, setSelected] = useState(
    params.get('task_id') && params.get('tracker_instance_id')
      ? `${params.get('tracker_instance_id')}/${params.get('task_id')}`
      : '',
  )
  const tasks = useQuery({
    queryKey: [
      'wiki-available-tasks',
      actor,
      ref?.registry_instance_id,
      ref?.namespace_id,
      spaceKey,
      offset,
    ],
    enabled: Boolean(ref && projects.data?.space.status === 'active'),
    queryFn: ({ signal }) =>
      apiRequest<components['schemas']['TaskCatalogItem'][]>(
        `/api/v1/spaces/${encodeURIComponent(spaceKey)}/available-tasks?offset=${offset}&registry_instance_id=${ref!.registry_instance_id}&namespace_id=${ref!.namespace_id}`,
        { signal },
      ),
  })
  if (!projects.data || projects.data.space.status !== 'active' || !projects.data.projects.length)
    return null
  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const [tracker, task] = selected.split('/')
    if (!tracker || !task || !ref) return
    const body: components['schemas']['LinkTaskRevision'] = {
      namespace: ref,
      document_id: documentId,
      revision_id: revisionId,
      task: { tracker_instance_id: tracker, task_id: task },
    }
    setBusy(true)
    setMessage('')
    try {
      await apiRequest(`/api/v1/spaces/${encodeURIComponent(spaceKey)}/managed-task-links`, {
        method: 'POST',
        body,
      })
      setMessage('Ревизия связана с задачей. Ссылка сохраняет эту версию документа.')
    } catch (error) {
      setMessage(error instanceof Error ? error.message : 'Не удалось проверить задачу')
    } finally {
      setBusy(false)
    }
  }
  return (
    <form onSubmit={submit} className="max-w-xl space-y-3 rounded-lg border border-border p-4">
      <h2 className="font-semibold">Связать эту ревизию с задачей</h2>
      <div className="space-y-2">
        <Label htmlFor={`task-project-${documentId}`}>Проект задачи</Label>
        <Select
          id={`task-project-${documentId}`}
          value={ref ? projectIdentity(ref) : ''}
          className="w-full"
          onValueChange={(nextValue) => {
            setProjectChoice(nextValue)
            setSelected('')
            setOffset(0)
          }}
          options={[
            { value: '', label: 'Выберите проект' },
            ...(available.map((project) => ({
              value: String(projectIdentity(project.namespace)),
              label: [project.name, ' · ', project.project_key].join(''),
            })) ?? []),
          ]}
        />
      </div>
      {!ref ? (
        <p role="status">Выберите связанный проект, чтобы загрузить задачи.</p>
      ) : tasks.isError ? (
        <p role="alert" className="text-danger">
          Каталог задач недоступен. Повторите загрузку.
        </p>
      ) : (
        <div className="space-y-2">
          <Label htmlFor={`task-ref-${documentId}`}>Задача проекта</Label>
          <Select
            id={`task-ref-${documentId}`}
            required
            value={selected}
            onValueChange={(nextValue) => setSelected(nextValue)}
            className="w-full"
            options={[
              { value: '', label: 'Выберите задачу' },
              ...(selected &&
              !tasks.data?.some(
                (item) => `${item.task.tracker_instance_id}/${item.task.task_id}` === selected,
              )
                ? [{ value: String(selected), label: 'Задача из ссылки Tracker' }]
                : []),
              ...(tasks.data?.map((item) => ({
                value: String(`${item.task.tracker_instance_id}/${item.task.task_id}`),
                label: [item.task_key, ' · ', item.label].join(''),
              })) ?? []),
            ]}
          />
        </div>
      )}
      <div className="flex flex-wrap gap-2">
        <Button
          type="button"
          variant="outline"
          disabled={offset === 0}
          onClick={() => setOffset(Math.max(0, offset - 50))}
        >
          Назад
        </Button>
        <Button
          type="button"
          variant="outline"
          disabled={(tasks.data?.length ?? 0) < 50}
          onClick={() => setOffset(offset + 50)}
        >
          Далее
        </Button>
        <Button type="button" variant="outline" onClick={() => void tasks.refetch()}>
          Обновить
        </Button>
      </div>
      {message && (
        <p role="status" className="text-sm">
          {message}
        </p>
      )}
      <Button disabled={busy || !selected || !ref}>
        {busy ? 'Проверяем задачу…' : 'Связать ревизию'}
      </Button>
    </form>
  )
}
