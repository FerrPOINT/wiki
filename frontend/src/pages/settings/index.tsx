import { Database, Globe2, KeyRound, Settings2 } from 'lucide-react'
import { useWikiSettings } from '@/shared/api/hooks'
import { ErrorState, LoadingState } from '@/shared/ui/async-states'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'
import { Input } from '@/shared/ui/input'
import { Label } from '@/shared/ui/label'
import { formatApiErrorForUser } from '@/shared/lib/api-error'
import { formatBytes } from '@/shared/lib/wiki-format'

function enabledLabel(value: boolean): string {
  return value ? 'включено' : 'выключено'
}

export function SettingsPage() {
  const settingsQuery = useWikiSettings()
  const settings = settingsQuery.data
  const settingsGroups = settings
    ? [
        {
          title: 'Инстанс',
          icon: Settings2,
          fields: [
            { id: 'instance-name', label: 'Название инстанса', value: settings.instance_name },
            { id: 'api-base-path', label: 'Путь API', value: settings.api_base_path },
            {
              id: 'default-space-key',
              label: 'Пространство по умолчанию',
              value: settings.default_space_key,
            },
          ],
        },
        {
          title: 'Поиск и Markdown',
          icon: Database,
          fields: [
            { id: 'search-backend', label: 'Поиск', value: settings.search_backend },
            {
              id: 'markdown-renderer',
              label: 'Рендер Markdown',
              value: settings.markdown_renderer,
            },
            { id: 'html-sanitizer', label: 'Очистка HTML', value: settings.html_sanitizer },
          ],
        },
        {
          title: 'Доступ',
          icon: KeyRound,
          fields: [
            {
              id: 'registration-enabled',
              label: 'Публичная регистрация',
              value: enabledLabel(settings.registration_enabled),
            },
            {
              id: 'public-links',
              label: 'Публичные ссылки',
              value: enabledLabel(settings.public_links_enabled),
            },
          ],
        },
        {
          title: 'Файлы и локализация',
          icon: Globe2,
          fields: [
            { id: 'storage-backend', label: 'Хранилище файлов', value: settings.storage_backend },
            {
              id: 'max-upload-bytes',
              label: 'Максимальный размер файла',
              value: formatBytes(settings.max_upload_bytes),
            },
            {
              id: 'default-language',
              label: 'Язык по умолчанию',
              value: settings.default_language,
            },
            { id: 'timezone', label: 'Часовой пояс', value: settings.timezone },
          ],
        },
      ]
    : []

  return (
    <div className="space-y-5">
      <section className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold">Настройки</h1>
          <p className="mt-1 max-w-3xl text-sm text-text-muted">
            Текущие политики инстанса, поиска, доступа, файлов и локализации Wiki.
          </p>
        </div>
      </section>

      {settingsQuery.isLoading && <LoadingState message="Загружаем настройки" />}
      {settingsQuery.isError && (
        <ErrorState
          message={formatApiErrorForUser(settingsQuery.error, 'Не удалось загрузить настройки')}
          onRetry={() => settingsQuery.refetch()}
        />
      )}

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
