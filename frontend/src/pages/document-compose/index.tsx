import { useState } from 'react'
import { Link } from 'react-router'
import { FileText, GitBranch, Save, Tag } from 'lucide-react'
import { Button } from '@/shared/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'
import { Input } from '@/shared/ui/input'
import { Label } from '@/shared/ui/label'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/shared/ui/tabs'
import { Textarea } from '@/shared/ui/textarea'

const starterMarkdown = `# Краткое описание

## Контекст
Почему документ нужен и к какой задаче относится.

## Решение
Что принято или что требуется сделать.

## Проверка
- Сценарий проверки
- Ссылка на материал
`

const templateChips = ['Требования', 'Исследование', 'Заметки реализации', 'План проверки']

export function DocumentComposePage() {
  const [title, setTitle] = useState('Требования к Wiki')
  const [body, setBody] = useState(starterMarkdown)

  return (
    <div className="space-y-5">
      <section className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold">Новый документ</h1>
          <p className="mt-1 max-w-3xl text-sm text-text-muted">
            Черновик можно связать с задачей, фазой workflow и материалами проверки.
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button size="sm" variant="secondary">
            <FileText className="h-4 w-4" />
            Предпросмотр
          </Button>
          <Button size="sm">
            <Save className="h-4 w-4" />
            Сохранить черновик
          </Button>
        </div>
      </section>

      <section className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_20rem]">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Редактор</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid gap-4 md:grid-cols-[1fr_12rem]">
              <div className="space-y-1.5">
                <Label htmlFor="document-title">Название</Label>
                <Input
                  id="document-title"
                  value={title}
                  onChange={(event) => setTitle(event.target.value)}
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="document-space">Пространство</Label>
                <Input id="document-space" value="ENG" readOnly />
              </div>
            </div>

            <div className="flex flex-wrap gap-2">
              {templateChips.map((chip) => (
                <button
                  key={chip}
                  type="button"
                  className="rounded-md border border-border px-2.5 py-1.5 text-xs text-text-secondary hover:bg-surface-raised hover:text-text-primary"
                >
                  {chip}
                </button>
              ))}
            </div>

            <Tabs defaultValue="write">
              <TabsList>
                <TabsTrigger value="write">Markdown</TabsTrigger>
                <TabsTrigger value="preview">Просмотр</TabsTrigger>
              </TabsList>
              <TabsContent value="write">
                <Textarea
                  className="min-h-96 font-mono text-sm"
                  aria-label="Markdown документа"
                  value={body}
                  onChange={(event) => setBody(event.target.value)}
                />
              </TabsContent>
              <TabsContent value="preview">
                <div className="min-h-96 rounded-md border border-border bg-background p-4">
                  <h2 className="text-xl font-semibold">{title}</h2>
                  <pre className="mt-4 whitespace-pre-wrap text-sm leading-6 text-text-secondary">
                    {body}
                  </pre>
                </div>
              </TabsContent>
            </Tabs>
          </CardContent>
        </Card>

        <div className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Связи</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3 text-sm">
              <Link
                to="/tasks/SDLC-42"
                className="flex items-center gap-2 rounded-md border border-border p-3 hover:bg-surface-raised"
              >
                <FileText className="h-4 w-4 text-accent" />
                Задача SDLC-42
              </Link>
              <Link
                to="/phases/implementation"
                className="flex items-center gap-2 rounded-md border border-border p-3 hover:bg-surface-raised"
              >
                <GitBranch className="h-4 w-4 text-accent" />
                Фаза реализации
              </Link>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-base">Метаданные</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <div className="space-y-1.5">
                <Label htmlFor="document-type">Тип</Label>
                <Input id="document-type" value="requirements" readOnly />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="document-tags">Теги</Label>
                <div className="flex items-center gap-2 rounded-md border border-border px-3 py-2 text-sm text-text-secondary">
                  <Tag className="h-4 w-4 text-accent" />
                  mvp, api, ui
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </section>
    </div>
  )
}
