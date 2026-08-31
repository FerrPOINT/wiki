import { Link, useParams } from 'react-router'
import { CheckCircle2, CircleDashed, FileCheck2, FileText, GitBranch } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'
import { Progress } from '@/shared/ui/progress'

const phases = [
  { id: 'analysis', name: 'Анализ', state: 'done', readiness: 100, missing: 0 },
  { id: 'implementation', name: 'Реализация', state: 'done', readiness: 92, missing: 1 },
  { id: 'testing', name: 'Проверка', state: 'active', readiness: 68, missing: 2 },
  { id: 'release', name: 'Релиз', state: 'planned', readiness: 35, missing: 3 },
]

const phaseDocuments = [
  { title: 'Заметки реализации', status: 'опубликован' },
  { title: 'План проверки', status: 'черновик' },
  { title: 'Материалы проверки', status: 'опубликован' },
]

const phaseMaterials = [
  { title: 'Ссылка: smoke-проверка', status: 'проверено' },
  { title: 'Файл: qa-screen.png', status: 'связано' },
  { title: 'Ссылка: pull request', status: 'связано' },
]

const phaseNames = new Map(phases.map((phase) => [phase.id, phase.name]))

export function PhaseDossiersPage() {
  return (
    <div className="space-y-5">
      <section>
        <h1 className="text-2xl font-bold">Фазы workflow</h1>
        <p className="mt-1 max-w-3xl text-sm text-text-muted">
          Каждая завершённая фаза должна иметь документы и материалы, достаточные для аудита.
        </p>
      </section>

      <section className="grid gap-4 lg:grid-cols-4">
        {phases.map((phase) => (
          <Card key={phase.id}>
            <CardHeader>
              <CardTitle className="flex items-center justify-between gap-2 text-base">
                {phase.name}
                {phase.state === 'done' ? (
                  <CheckCircle2 className="h-4 w-4 text-success" />
                ) : (
                  <CircleDashed className="h-4 w-4 text-warning" />
                )}
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-1.5">
                <div className="flex justify-between text-xs text-text-muted">
                  <span>Заполненность</span>
                  <span>{phase.readiness}%</span>
                </div>
                <Progress
                  value={phase.readiness}
                  variant={phase.missing > 2 ? 'danger' : 'default'}
                />
              </div>
              <div className="text-xs text-text-muted">
                {phase.missing === 0 ? 'Материалы закрыты' : `Не хватает: ${phase.missing}`}
              </div>
              <Link to={`/phases/${phase.id}`} className="text-sm text-accent">
                Открыть фазу
              </Link>
            </CardContent>
          </Card>
        ))}
      </section>
    </div>
  )
}

export function PhaseDossierPage() {
  const { phaseId = 'implementation' } = useParams()
  const phaseName = phaseNames.get(phaseId) ?? phaseId

  return (
    <div className="space-y-5">
      <section className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
        <div>
          <div className="text-sm text-text-muted">карточка фазы</div>
          <h1 className="mt-2 text-2xl font-bold">{phaseName}</h1>
          <p className="mt-1 max-w-3xl text-sm text-text-muted">
            Документы и материалы, привязанные к фазе workflow.
          </p>
        </div>
        <div className="min-w-72 rounded-md border border-border p-3">
          <div className="flex items-center justify-between text-sm">
            <span className="font-medium">Заполненность</span>
            <span className="text-text-muted">92%</span>
          </div>
          <Progress value={92} className="mt-3" />
          <p className="mt-2 text-xs text-text-muted">Остался один материал проверки.</p>
        </div>
      </section>

      <section className="grid gap-4 xl:grid-cols-[1fr_1fr]">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base">
              <FileText className="h-4 w-4 text-accent" />
              Документы фазы
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            {phaseDocuments.map((document) => (
              <Link
                key={document.title}
                to="/documents/implementation-evidence"
                className="flex items-center justify-between rounded-md border border-border p-3 text-sm hover:bg-surface-raised"
              >
                <span>{document.title}</span>
                <span className="rounded bg-surface-raised px-2 py-1 text-xs text-text-secondary">
                  {document.status}
                </span>
              </Link>
            ))}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base">
              <FileCheck2 className="h-4 w-4 text-accent" />
              Материалы
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-3 text-sm text-text-secondary">
            {phaseMaterials.map((material) => (
              <Link
                key={material.title}
                to="/evidence"
                className="flex items-center justify-between rounded-md border border-border p-3 hover:bg-surface-raised"
              >
                <span className="inline-flex items-center gap-2">
                  <GitBranch className="h-4 w-4 text-accent" />
                  {material.title}
                </span>
                <span className="rounded bg-surface-raised px-2 py-1 text-xs text-text-muted">
                  {material.status}
                </span>
              </Link>
            ))}
          </CardContent>
        </Card>
      </section>
    </div>
  )
}
