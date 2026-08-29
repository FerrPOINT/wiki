import { Database, Globe2, KeyRound, Save, Settings2 } from 'lucide-react'
import { Button } from '@/shared/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'
import { Input } from '@/shared/ui/input'
import { Label } from '@/shared/ui/label'

const settingsGroups = [
  {
    title: 'Инстанс',
    icon: Settings2,
    fields: [
      { id: 'instance-name', label: 'Название инстанса', value: 'Wiki' },
      { id: 'base-url', label: 'Base URL', value: 'https://wiki.local' },
    ],
  },
  {
    title: 'Поиск',
    icon: Database,
    fields: [
      { id: 'index-backend', label: 'Индекс', value: 'PostgreSQL FTS' },
      { id: 'index-lag-slo', label: 'SLO обновления индекса', value: '30 секунд' },
    ],
  },
  {
    title: 'Доступ',
    icon: KeyRound,
    fields: [
      { id: 'default-role', label: 'Роль по умолчанию', value: 'читатель' },
      { id: 'public-links', label: 'Публичные ссылки', value: 'выключены' },
    ],
  },
  {
    title: 'Локализация',
    icon: Globe2,
    fields: [
      { id: 'default-language', label: 'Язык по умолчанию', value: 'ru' },
      { id: 'timezone', label: 'Часовой пояс', value: 'Europe/Moscow' },
    ],
  },
]

export function SettingsPage() {
  return (
    <div className="space-y-5">
      <section className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold">Настройки</h1>
          <p className="mt-1 max-w-3xl text-sm text-text-muted">
            Инстанс, поиск, доступы и локализация Wiki. Значения здесь отражают целевой MVP.
          </p>
        </div>
        <Button size="sm">
          <Save className="h-4 w-4" />
          Сохранить
        </Button>
      </section>

      <section className="grid gap-4 lg:grid-cols-2">
        {settingsGroups.map((group) => {
          const Icon = group.icon
          return (
            <Card key={group.title}>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base">
                  <Icon className="h-4 w-4 text-accent" />
                  {group.title}
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                {group.fields.map((field) => (
                  <div key={field.label} className="space-y-1.5">
                    <Label htmlFor={field.id}>{field.label}</Label>
                    <Input id={field.id} value={field.value} readOnly />
                  </div>
                ))}
              </CardContent>
            </Card>
          )
        })}
      </section>
    </div>
  )
}
