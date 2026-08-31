import { Link } from 'react-router'
import { CheckCircle2, ExternalLink, FileText, Filter, Link2, Upload } from 'lucide-react'
import { Button } from '@/shared/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'
import { Input } from '@/shared/ui/input'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/shared/ui/table'

const evidenceItems = [
  {
    title: 'Отчёт smoke-проверки',
    task: 'SDLC-42',
    phaseId: 'testing',
    phase: 'Проверка',
    source: 'CI-CD',
    status: 'проверено',
    owner: 'QA',
  },
  {
    title: 'Заметки архитектурного ревью',
    task: 'SDLC-42',
    phaseId: 'review',
    phase: 'Ревью',
    source: 'Wiki',
    status: 'связано',
    owner: 'Техлид',
  },
  {
    title: 'Материал релиза',
    task: 'SDLC-37',
    phaseId: 'release',
    phase: 'Релиз',
    source: 'CI-CD',
    status: 'нет метаданных',
    owner: 'Release',
  },
]

export function EvidencePage() {
  return (
    <div className="space-y-5">
      <section className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold">Материалы</h1>
          <p className="mt-1 max-w-3xl text-sm text-text-muted">
            Артефакты, ссылки и файлы, подтверждающие выполнение задач и фаз workflow.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button size="sm">
            <Upload className="h-4 w-4" />
            Загрузить файл
          </Button>
          <Button size="sm" variant="secondary">
            <Link2 className="h-4 w-4" />
            Добавить ссылку
          </Button>
        </div>
      </section>

      <section className="grid gap-3 rounded-md border border-border bg-surface p-3 md:grid-cols-[1fr_auto_auto_auto]">
        <Input
          placeholder="Поиск по материалам, задаче или источнику"
          aria-label="Поиск материалов"
        />
        <Button size="sm" variant="secondary">
          <Filter className="h-4 w-4" />
          Фаза
        </Button>
        <Button size="sm" variant="secondary">
          <Filter className="h-4 w-4" />
          Источник
        </Button>
        <Button size="sm" variant="outline">
          Сбросить
        </Button>
      </section>

      <section className="grid gap-4 sm:grid-cols-3">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Проверено</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-3">
              <CheckCircle2 className="h-5 w-5 text-success" />
              <span className="text-2xl font-semibold">19</span>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Связано</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-3">
              <ExternalLink className="h-5 w-5 text-accent" />
              <span className="text-2xl font-semibold">8</span>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm">Файлы</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-3">
              <FileText className="h-5 w-5 text-warning" />
              <span className="text-2xl font-semibold">12</span>
            </div>
          </CardContent>
        </Card>
      </section>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Реестр материалов</CardTitle>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Материал</TableHead>
                <TableHead>Задача</TableHead>
                <TableHead>Фаза</TableHead>
                <TableHead>Источник</TableHead>
                <TableHead>Статус</TableHead>
                <TableHead>Ответственный</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {evidenceItems.map((item) => (
                <TableRow key={`${item.task}-${item.title}`}>
                  <TableCell className="font-medium">{item.title}</TableCell>
                  <TableCell>
                    <Link
                      to={`/tasks/${item.task}`}
                      className="text-accent hover:text-accent-hover"
                    >
                      {item.task}
                    </Link>
                  </TableCell>
                  <TableCell>
                    <Link
                      to={`/phases/${item.phaseId}`}
                      className="text-accent hover:text-accent-hover"
                    >
                      {item.phase}
                    </Link>
                  </TableCell>
                  <TableCell>{item.source}</TableCell>
                  <TableCell>
                    <span className="rounded bg-surface-raised px-2 py-1 text-xs text-text-secondary">
                      {item.status}
                    </span>
                  </TableCell>
                  <TableCell>{item.owner}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  )
}
