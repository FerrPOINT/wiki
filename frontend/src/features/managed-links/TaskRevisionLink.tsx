import { useState, type FormEvent } from 'react'
import { useSearchParams } from 'react-router'
import { Button, Label } from '@sdlc/ui/ui'
import { useQuery } from '@tanstack/react-query'
import { apiRequest } from '@/api/client'
import { useNamespaceContext } from '@/widgets/namespace-context'
import type { components } from '@/api/generated'

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
  const { ref, query } = useNamespaceContext()
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
      ref?.registry_instance_id,
      ref?.namespace_id,
      spaceKey,
      offset,
    ],
    enabled: Boolean(
      ref && query.data?.resource_key === spaceKey && query.data.binding.state === 'active',
    ),
    queryFn: ({ signal }) =>
      apiRequest<components['schemas']['TaskCatalogItem'][]>(
        `/api/v1/spaces/${encodeURIComponent(spaceKey)}/available-tasks?offset=${offset}`,
        { signal },
      ),
  })
  if (!ref || query.data?.resource_key !== spaceKey || query.data.binding.state !== 'active')
    return null
  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const [tracker, task] = selected.split('/')
    if (!tracker || !task) return
    const body: components['schemas']['LinkTaskRevision'] = {
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
      {tasks.isError ? (
        <p role="alert" className="text-danger">
          Каталог задач недоступен. Повторите загрузку.
        </p>
      ) : (
        <div className="space-y-2">
          <Label htmlFor={`task-ref-${documentId}`}>Задача проекта</Label>
          <select
            id={`task-ref-${documentId}`}
            required
            value={selected}
            onChange={(event) => setSelected(event.target.value)}
            className="min-h-10 w-full rounded-md border border-border bg-surface px-3"
          >
            <option value="">Выберите задачу</option>
            {selected &&
              !tasks.data?.some(
                (item) => `${item.task.tracker_instance_id}/${item.task.task_id}` === selected,
              ) && <option value={selected}>Задача из ссылки Tracker</option>}
            {tasks.data?.map((item) => (
              <option
                key={item.task.task_id}
                value={`${item.task.tracker_instance_id}/${item.task.task_id}`}
              >
                {item.task_key} · {item.label}
              </option>
            ))}
          </select>
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
      <Button disabled={busy || !selected}>{busy ? 'Проверяем задачу…' : 'Связать ревизию'}</Button>
    </form>
  )
}
