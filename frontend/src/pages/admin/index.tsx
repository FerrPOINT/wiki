import { Link } from 'react-router'
import { FileCheck2, History, Library, Settings, ShieldCheck, Users } from 'lucide-react'
import { useAuditLog, useSpaces, useUsers, useWikiSettings } from '@/shared/api/hooks'
import { ErrorState, LoadingState } from '@sdlc/ui/ui'
import { Card, CardContent, CardHeader, CardTitle } from '@sdlc/ui/ui'
import { formatFirstApiErrorForUser } from '@/shared/lib/api-error'
import { formatBytes, formatDateTime } from '@/shared/lib/wiki-format'

const adminSections = [
  {
    title: 'Пользователи',
    description: 'Учётные записи, роли и доступы к пространствам.',
    href: '/users',
    icon: Users,
  },
  {
    title: 'Настройки',
    description: 'Настройки инстанса, поиска, локали и политик доступа.',
    href: '/settings',
    icon: Settings,
  },
  {
    title: 'Аудит',
    description: 'События документов, материалов, пользователей и прав доступа.',
    href: '/audit-log',
    icon: History,
  },
]

function enabledLabel(value: boolean | undefined): string {
  if (value === undefined) return 'не задано'
  return value ? 'включена' : 'выключена'
}

export function AdminPage() {
  const usersQuery = useUsers()
  const spacesQuery = useSpaces()
  const auditQuery = useAuditLog()
  const settingsQuery = useWikiSettings()

  const users = usersQuery.data?.users ?? []
  const spaces = spacesQuery.data?.spaces ?? []
  const auditEntries = auditQuery.data?.entries ?? []
  const settings = settingsQuery.data
  const activeUsers = users.filter((user) => user.active).length
  const documentCount = spaces.reduce((sum, space) => sum + space.document_count, 0)
  const isLoading =
    usersQuery.isLoading || spacesQuery.isLoading || auditQuery.isLoading || settingsQuery.isLoading
  const isError =
    usersQuery.isError || spacesQuery.isError || auditQuery.isError || settingsQuery.isError
  const overviewError = formatFirstApiErrorForUser(
    [usersQuery.error, spacesQuery.error, auditQuery.error, settingsQuery.error],
    'Не удалось загрузить состояние инстанса',
  )

  function retryOverview() {
    void usersQuery.refetch()
    void spacesQuery.refetch()
    void auditQuery.refetch()
    void settingsQuery.refetch()
  }

  const overviewItems = [
    {
      label: 'Пользователи',
      value: users.length.toString(),
      status: `Активных: ${activeUsers}`,
      icon: Users,
    },
    {
      label: 'Пространства',
      value: spaces.length.toString(),
      status: `Документов: ${documentCount}`,
      icon: Library,
    },
    {
      label: 'Аудит',
      value: auditEntries.length.toString(),
      status: auditEntries[0] ? formatDateTime(auditEntries[0].created_at) : 'Событий пока нет',
      icon: History,
    },
    {
      label: 'Регистрация',
      value: enabledLabel(settings?.registration_enabled),
      status: `Файлы до ${formatBytes(settings?.max_upload_bytes)}`,
      icon: ShieldCheck,
    },
  ]

  return (
    <div className="space-y-5">
      <section>
        <h1 className="text-2xl font-bold">Администрирование</h1>
        <p className="mt-1 max-w-3xl text-sm text-text-muted">
          Центр управления Wiki: доступы, настройки и аудит для базы знаний SDLC.
        </p>
      </section>

      <section className="grid gap-4 lg:grid-cols-3">
        {adminSections.map((section) => {
          const Icon = section.icon
          return (
            <Card key={section.href}>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base">
                  <Icon className="h-4 w-4 text-accent" />
                  {section.title}
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                <p className="text-sm text-text-secondary">{section.description}</p>
                <Link to={section.href} className="text-sm text-accent hover:text-accent-hover">
                  Открыть
                </Link>
              </CardContent>
            </Card>
          )
        })}
      </section>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-base">
            <FileCheck2 className="h-4 w-4 text-accent" />
            Состояние инстанса
          </CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading && <LoadingState message="Загружаем состояние инстанса" />}
          {isError && <ErrorState message={overviewError} onRetry={retryOverview} />}
          {!isLoading && !isError && (
            <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
              {overviewItems.map((item) => {
                const Icon = item.icon
                return (
                  <div key={item.label} className="rounded-md border border-border p-3">
                    <div className="flex items-center gap-2 text-sm font-medium">
                      <Icon className="h-4 w-4 text-accent" />
                      {item.label}
                    </div>
                    <div className="mt-3 text-2xl font-semibold">{item.value}</div>
                    <div className="mt-2 text-xs text-text-muted">{item.status}</div>
                  </div>
                )
              })}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
