import { Check, Database, Globe2, KeyRound, Minus, Settings2 } from 'lucide-react'
import { useWikiSettings } from '@/shared/api/hooks'
import { ErrorState, LoadingState } from '@sdlc/ui/ui'
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
          id: 'instance',
          title: 'Инстанс',
          icon: Settings2,
          fields: [
            { label: 'Название инстанса', value: settings.instance_name },
            { label: 'Путь API', value: settings.api_base_path },
            {
              label: 'Пространство по умолчанию',
              value: settings.default_space_key,
            },
          ],
        },
        {
          id: 'search-markdown',
          title: 'Поиск и Markdown',
          icon: Database,
          fields: [
            { label: 'Поиск', value: settings.search_backend },
            {
              label: 'Рендер Markdown',
              value: settings.markdown_renderer,
            },
            { label: 'Очистка HTML', value: settings.html_sanitizer },
          ],
        },
        {
          id: 'access',
          title: 'Доступ',
          icon: KeyRound,
          fields: [
            {
              label: 'Публичная регистрация',
              value: enabledLabel(settings.registration_enabled),
              enabled: settings.registration_enabled,
            },
            {
              label: 'Публичные ссылки',
              value: enabledLabel(settings.public_links_enabled),
              enabled: settings.public_links_enabled,
            },
          ],
        },
        {
          id: 'files-locale',
          title: 'Файлы и локализация',
          icon: Globe2,
          fields: [
            { label: 'Хранилище файлов', value: settings.storage_backend },
            {
              label: 'Максимальный размер файла',
              value: formatBytes(settings.max_upload_bytes),
            },
            {
              label: 'Язык по умолчанию',
              value: settings.default_language,
            },
            { label: 'Часовой пояс', value: settings.timezone },
          ],
        },
      ]
    : []

  return (
    <div className="min-w-0 space-y-5">
      <h1 className="text-2xl font-bold">Настройки</h1>

      {settingsQuery.isLoading && <LoadingState message="Загружаем настройки" />}
      {settingsQuery.isError && (
        <ErrorState
          message={formatApiErrorForUser(settingsQuery.error, 'Не удалось загрузить настройки')}
          onRetry={() => settingsQuery.refetch()}
        />
      )}

      {!settingsQuery.isLoading && !settingsQuery.isError && settings && (
        <div className="grid gap-x-10 gap-y-7 lg:grid-cols-2">
          {settingsGroups.map((group) => {
            const Icon = group.icon
            return (
              <section
                key={group.id}
                aria-labelledby={group.id}
                className="min-w-0 border-t border-border pt-3"
              >
                <h2 id={group.id} className="flex items-center gap-2 text-base font-semibold">
                  <Icon aria-hidden className="h-4 w-4 shrink-0 text-accent" />
                  {group.title}
                </h2>
                <dl className="mt-2 divide-y divide-border">
                  {group.fields.map((field) => (
                    <div
                      key={field.label}
                      className="grid grid-cols-1 gap-0.5 py-2 sm:grid-cols-[minmax(0,1fr)_minmax(0,1.2fr)] sm:gap-4"
                    >
                      <dt className="text-sm text-text-muted">{field.label}</dt>
                      <dd className="flex min-w-0 items-start gap-1.5 text-sm font-medium">
                        {'enabled' in field &&
                          (field.enabled ? (
                            <Check aria-hidden className="mt-0.5 h-4 w-4 shrink-0 text-success" />
                          ) : (
                            <Minus
                              aria-hidden
                              className="mt-0.5 h-4 w-4 shrink-0 text-text-muted"
                            />
                          ))}
                        <span className="min-w-0 break-all">{field.value}</span>
                      </dd>
                    </div>
                  ))}
                </dl>
              </section>
            )
          })}
        </div>
      )}
    </div>
  )
}
