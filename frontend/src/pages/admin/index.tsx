import {
  Card,
  CardContent,
  PageHeader,
  SectionHeader,
  Metric,
  type MetricProps,
  NamespaceLink as Link,
} from '@pdlc/ui/ui'

import { ArrowRight, History, Library, Settings, ShieldCheck, Users } from 'lucide-react'
import { useAuditLog, useSpaces, useUsers, useWikiSettings } from '@/shared/api/hooks'

import { formatApiErrorForUser } from '@/shared/lib/api-error'
import { formatBytes, formatDateTime } from '@/shared/lib/wiki-format'

const ADMIN_AUDIT_PAGE_LIMIT = 50

const adminSections = [
  {
    title: 'Пользователи',
    description: 'Профили Wiki только для чтения; учётные записи управляются в Admin Panel.',
    href: '/users',
    icon: Users,
  },
  {
    title: 'Настройки',
    description: 'Безопасный снимок настроек работающего инстанса только для чтения.',
    href: '/settings',
    icon: Settings,
  },
  {
    title: 'Аудит',
    description: 'Неизменяемые события документов, материалов и прав доступа.',
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
  const auditQuery = useAuditLog({ limit: ADMIN_AUDIT_PAGE_LIMIT })
  const settingsQuery = useWikiSettings()

  const users = usersQuery.data?.users ?? []
  const spaces = spacesQuery.data?.spaces ?? []
  const auditEntries = auditQuery.data?.entries ?? []
  const settings = settingsQuery.data
  const activeUsers = users.filter((user) => user.active).length
  const documentCount = spaces.reduce((sum, space) => sum + space.document_count, 0)

  const overviewItems = [
    {
      label: 'Профили Wiki',
      value: users.length.toString(),
      icon: <Users className="h-5 w-5" />,
      description: `Активных профилей: ${activeUsers}`,
      state: usersQuery.isLoading
        ? { kind: 'loading' }
        : (
              usersQuery.isError
                ? formatApiErrorForUser(usersQuery.error, 'Не удалось загрузить профили')
                : undefined
            )
          ? {
              kind: 'error',
              message: usersQuery.isError
                ? formatApiErrorForUser(usersQuery.error, 'Не удалось загрузить профили')
                : undefined,
              onRetry: () => void usersQuery.refetch(),
            }
          : { kind: 'ready' },
    },
    {
      label: 'Пространства',
      value: spaces.length.toString(),
      icon: <Library className="h-5 w-5" />,
      description: `Документов: ${documentCount}`,
      state: spacesQuery.isLoading
        ? { kind: 'loading' }
        : (
              spacesQuery.isError
                ? formatApiErrorForUser(spacesQuery.error, 'Не удалось загрузить пространства')
                : undefined
            )
          ? {
              kind: 'error',
              message: spacesQuery.isError
                ? formatApiErrorForUser(spacesQuery.error, 'Не удалось загрузить пространства')
                : undefined,
              onRetry: () => void spacesQuery.refetch(),
            }
          : { kind: 'ready' },
    },
    {
      label: 'События аудита',
      value: auditEntries.length.toString(),
      icon: <History className="h-5 w-5" />,
      description: auditEntries[0]
        ? `Первая страница, лимит ${ADMIN_AUDIT_PAGE_LIMIT}; последнее: ${formatDateTime(auditEntries[0].created_at)}`
        : `Первая страница, лимит ${ADMIN_AUDIT_PAGE_LIMIT}; событий пока нет`,
      state: auditQuery.isLoading
        ? { kind: 'loading' }
        : (
              auditQuery.isError
                ? formatApiErrorForUser(auditQuery.error, 'Не удалось загрузить события аудита')
                : undefined
            )
          ? {
              kind: 'error',
              message: auditQuery.isError
                ? formatApiErrorForUser(auditQuery.error, 'Не удалось загрузить события аудита')
                : undefined,
              onRetry: () => void auditQuery.refetch(),
            }
          : { kind: 'ready' },
    },
    {
      label: 'Локальная регистрация',
      value: enabledLabel(settings?.registration_enabled),
      icon: <ShieldCheck className="h-5 w-5" />,
      description: `Лимит файла: ${formatBytes(settings?.max_upload_bytes)}`,
      state: settingsQuery.isLoading
        ? { kind: 'loading' }
        : (
              settingsQuery.isError
                ? formatApiErrorForUser(settingsQuery.error, 'Не удалось загрузить настройки')
                : undefined
            )
          ? {
              kind: 'error',
              message: settingsQuery.isError
                ? formatApiErrorForUser(settingsQuery.error, 'Не удалось загрузить настройки')
                : undefined,
              onRetry: () => void settingsQuery.refetch(),
            }
          : { kind: 'ready' },
    },
  ] satisfies MetricProps[]

  return (
    <div className="min-w-0 space-y-6">
      <PageHeader
        title={'Администрирование'}
        context={
          <>
            <p className="mt-1 max-w-3xl text-sm text-text-muted">
              Быстрый доступ к профилям Wiki, настройкам среды выполнения и журналу аудита.
            </p>
          </>
        }
      />

      <section aria-labelledby="admin-sections-title">
        <SectionHeader
          headingId="admin-sections-title"
          className="mb-3"
          title={<>Административные разделы</>}
        />
        <div className="grid gap-3 lg:grid-cols-3">
          {adminSections.map((section) => {
            const Icon = section.icon
            return (
              <Card asChild interactive key={section.href}>
                <Link to={section.href}>
                  <CardContent className="group flex min-w-0 items-center gap-3">
                    <span className="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-surface-raised text-accent">
                      <Icon className="h-5 w-5" aria-hidden />
                    </span>
                    <span className="min-w-0 flex-1">
                      <span className="block text-sm font-semibold">{section.title}</span>
                      <span className="mt-1 block text-xs leading-5 text-text-muted">
                        {section.description}
                      </span>
                    </span>
                    <ArrowRight
                      className="h-4 w-4 shrink-0 text-text-muted transition-transform group-hover:translate-x-0.5 group-hover:text-accent"
                      aria-hidden
                    />
                  </CardContent>
                </Link>
              </Card>
            )
          })}
        </div>
      </section>

      <section aria-labelledby="instance-state-title">
        <SectionHeader
          headingId="instance-state-title"
          className="mb-3"
          title={<>Состояние инстанса</>}
        />
        <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
          {overviewItems.map((item) => (
            <Metric key={item.label} {...item} />
          ))}
        </div>
      </section>
    </div>
  )
}
