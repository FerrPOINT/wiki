import { Link, useParams } from 'react-router'
import { CheckCircle2, CircleDashed, FileCheck2, FileText, GitBranch, Link2 } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'
import { Progress } from '@/shared/ui/progress'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/shared/ui/table'

const tasks = [
  {
    key: 'SDLC-42',
    title: 'Сформировать Wiki как базу знаний SDLC',
    documents: 4,
    materials: 6,
    readiness: 86,
  },
  {
    key: 'SDLC-39',
    title: 'Собрать материалы проверки',
    documents: 2,
    materials: 3,
    readiness: 64,
  },
  {
    key: 'SDLC-37',
    title: 'Описать чеклист релиза',
    documents: 3,
    materials: 2,
    readiness: 72,
  },
]

const taskDocuments = [
  { title: 'Требования', type: 'requirements', status: 'опубликован' },
  { title: 'Архитектура', type: 'architecture', status: 'опубликован' },
  { title: 'План проверки', type: 'test_plan', status: 'черновик' },
]

const taskPhases = [
  { key: 'analysis', label: 'Анализ', done: true, materials: 2 },
  { key: 'implementation', label: 'Реализация', done: true, materials: 3 },
  { key: 'testing', label: 'Проверка', done: false, materials: 1 },
]

const taskMaterials = [
  { title: 'PR: wiki-mvp-docs', source: 'Git', status: 'связано' },
  { title: 'Smoke-проверка frontend', source: 'CI-CD', status: 'проверено' },
  { title: 'Скриншоты страниц', source: 'Wiki', status: 'проверено' },
]

export function TaskDossiersPage() {
  return (
    <div className="space-y-5">
      <section>
        <h1 className="text-2xl font-bold">Задачи</h1>
        <p className="mt-1 max-w-3xl text-sm text-text-muted">
          Wiki собирает документы и материалы вокруг внешнего ключа задачи, но не владеет её
          статусом в трекере.
        </p>
      </section>

      <section className="grid gap-4 lg:grid-cols-3">
        {tasks.map((task) => (
          <Card key={task.key}>
            <CardHeader>
              <CardTitle className="text-base">{task.key}</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <p className="min-h-10 text-sm text-text-secondary">{task.title}</p>
              <div className="grid grid-cols-2 gap-2 text-xs text-text-muted">
                <span className="rounded bg-surface-raised px-2 py-1">
                  Документы: {task.documents}
                </span>
                <span className="rounded bg-surface-raised px-2 py-1">
                  Материалы: {task.materials}
                </span>
              </div>
              <div className="space-y-1.5">
                <div className="flex justify-between text-xs text-text-muted">
                  <span>Заполненность</span>
                  <span>{task.readiness}%</span>
                </div>
                <Progress value={task.readiness} />
              </div>
              <Link to={`/tasks/${task.key}`} className="inline-flex text-sm text-accent">
                Открыть задачу
              </Link>
            </CardContent>
          </Card>
        ))}
      </section>
    </div>
  )
}

export function TaskDossierPage() {
  const { taskKey = 'SDLC-42' } = useParams()

  return (
    <div className="space-y-5">
      <section className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
        <div>
          <div className="text-sm text-text-muted">карточка задачи</div>
          <h1 className="mt-2 text-2xl font-bold">{taskKey}</h1>
          <p className="mt-1 max-w-3xl text-sm text-text-muted">
            Документы, фазы и материалы, связанные с задачей.
          </p>
        </div>
        <div className="grid grid-cols-3 gap-2 text-center text-xs text-text-muted sm:min-w-80">
          <div className="rounded-md border border-border p-2">
            <div className="text-lg font-semibold text-text-primary">3</div>
            документы
          </div>
          <div className="rounded-md border border-border p-2">
            <div className="text-lg font-semibold text-text-primary">3</div>
            фазы
          </div>
          <div className="rounded-md border border-border p-2">
            <div className="text-lg font-semibold text-text-primary">6</div>
            материалы
          </div>
        </div>
      </section>

      <section className="grid gap-4 xl:grid-cols-[1.2fr_0.8fr]">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Документы задачи</CardTitle>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Документ</TableHead>
                  <TableHead>Тип</TableHead>
                  <TableHead>Статус</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {taskDocuments.map((document) => (
                  <TableRow key={document.title}>
                    <TableCell>
                      <Link
                        to="/documents/product-requirements"
                        className="inline-flex items-center gap-2 text-accent hover:text-accent-hover"
                      >
                        <FileText className="h-4 w-4" />
                        {document.title}
                      </Link>
                    </TableCell>
                    <TableCell>{document.type}</TableCell>
                    <TableCell>
                      <span className="rounded bg-surface-raised px-2 py-1 text-xs text-text-secondary">
                        {document.status}
                      </span>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">Фазы</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            {taskPhases.map((phase) => (
              <Link
                key={phase.key}
                to={`/phases/${phase.key}`}
                className="flex items-center justify-between rounded-md border border-border p-3 text-sm hover:bg-surface-raised"
              >
                <span className="inline-flex items-center gap-2">
                  <GitBranch className="h-4 w-4 text-accent" />
                  {phase.label}
                </span>
                <span className="inline-flex items-center gap-2 text-xs text-text-muted">
                  {phase.materials} материала
                  {phase.done ? (
                    <CheckCircle2 className="h-4 w-4 text-success" />
                  ) : (
                    <CircleDashed className="h-4 w-4 text-warning" />
                  )}
                </span>
              </Link>
            ))}
          </CardContent>
        </Card>
      </section>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-base">
            <FileCheck2 className="h-4 w-4 text-accent" />
            Материалы задачи
          </CardTitle>
        </CardHeader>
        <CardContent className="grid gap-3 lg:grid-cols-3">
          {taskMaterials.map((material) => (
            <Link
              key={material.title}
              to="/evidence"
              className="rounded-md border border-border p-3 hover:bg-surface-raised"
            >
              <div className="flex items-center gap-2 text-sm font-medium">
                <Link2 className="h-4 w-4 text-accent" />
                {material.title}
              </div>
              <div className="mt-2 flex gap-2 text-xs text-text-muted">
                <span>{material.source}</span>
                <span>·</span>
                <span>{material.status}</span>
              </div>
            </Link>
          ))}
        </CardContent>
      </Card>
    </div>
  )
}
