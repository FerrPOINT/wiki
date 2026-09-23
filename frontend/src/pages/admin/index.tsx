import { Link } from 'react-router'
import {
  ArrowRight,
  History,
  Library,
  Loader2,
  RotateCcw,
  Settings,
  ShieldCheck,
  Users,
  type LucideIcon,
} from 'lucide-react'
import { useAuditLog, useSpaces, useUsers, useWikiSettings } from '@/shared/api/hooks'
import { Button } from '@sdlc/ui/ui'
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

interface AdminMetricProps {
  error?: string
  icon: LucideIcon
  isLoading: boolean
  label: string
  onRetry: () => void
  status: string
  value: string
}

function AdminMetric({
  error,
  icon: Icon,
  isLoading,
  label,
  onRetry,
  status,
  value,
}: AdminMetricProps) {
  return (
    <div className="min-h-28 min-w-0 rounded-md border border-border p-3" aria-busy={isLoading}>
      <div className="flex min-h-10 items-center gap-2">
        <Icon className="h-4 w-4 shrink-0 text-accent" aria-hidden />
        <h3 className="min-w-0 flex-1 text-sm font-medium">{label}</h3>
        {isLoading && (
          <Loader2
            className="h-4 w-4 shrink-0 animate-spin text-text-muted"
            aria-label={`Загружаем: ${label}`}
          />
        )}
        {error && (
          <Button
            type="button"
            size="icon"
            variant="ghost"
            className="h-10 w-10 min-h-10 min-w-10 shrink-0 sm:min-h-10 sm:min-w-10"
            aria-label={`Повторить загрузку: ${label}`}
            title="Повторить"
            onClick={onRetry}
          >
            <RotateCcw className="h-4 w-4" aria-hidden />
          </Button>
        )}
      </div>

      {isLoading ? (
        <p className="mt-2 text-sm text-text-muted" role="status">
          Загрузка...
        </p>
      ) : error ? (
        <p className="mt-2 text-sm text-danger" role="alert">
          {error}
        </p>
      ) : (
        <>
          <div className="mt-2 text-2xl font-semibold">{value}</div>
          <p className="mt-1 text-xs text-text-muted">{status}</p>
        </>
      )}
    </div>
  )
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

  const overviewItems: AdminMetricProps[] = [
    {
      label: 'Профили Wiki',
      value: users.length.toString(),
      status: `Активных профилей: ${activeUsers}`,
      icon: Users,
      isLoading: usersQuery.isLoading,
      error: usersQuery.isError
        ? formatApiErrorForUser(usersQuery.error, 'Не удалось загрузить профили')
        : undefined,
      onRetry: () => void usersQuery.refetch(),
    },
    {
      label: 'Пространства',
      value: spaces.length.toString(),
      status: `Документов: ${documentCount}`,
      icon: Library,
      isLoading: spacesQuery.isLoading,
      error: spacesQuery.isError
        ? formatApiErrorForUser(spacesQuery.error, 'Не удалось загрузить пространства')
        : undefined,
      onRetry: () => void spacesQuery.refetch(),
    },
    {
      label: 'События аудита',
      value: auditEntries.length.toString(),
      status: auditEntries[0]
        ? `Первая страница, лимит ${ADMIN_AUDIT_PAGE_LIMIT}; последнее: ${formatDateTime(auditEntries[0].created_at)}`
        : `Первая страница, лимит ${ADMIN_AUDIT_PAGE_LIMIT}; событий пока нет`,
      icon: History,
      isLoading: auditQuery.isLoading,
      error: auditQuery.isError
        ? formatApiErrorForUser(auditQuery.error, 'Не удалось загрузить события аудита')
        : undefined,
      onRetry: () => void auditQuery.refetch(),
    },
    {
      label: 'Локальная регистрация',
      value: enabledLabel(settings?.registration_enabled),
      status: `Лимит файла: ${formatBytes(settings?.max_upload_bytes)}`,
      icon: ShieldCheck,
      isLoading: settingsQuery.isLoading,
      error: settingsQuery.isError
        ? formatApiErrorForUser(settingsQuery.error, 'Не удалось загрузить настройки')
        : undefined,
      onRetry: () => void settingsQuery.refetch(),
    },
  ]

  return (
    <div className="min-w-0 max-w-screen-2xl space-y-6">
      <header>
        <h1 className="text-2xl font-bold">Администрирование</h1>
        <p className="mt-1 max-w-3xl text-sm text-text-muted">
          Быстрый доступ к профилям Wiki, настройкам среды выполнения и журналу аудита.
        </p>
      </header>

      <section aria-labelledby="admin-sections-title">
        <h2 id="admin-sections-title" className="mb-3 text-base font-semibold">
          Административные разделы
        </h2>
        <div className="grid gap-3 lg:grid-cols-3">
          {adminSections.map((section) => {
            const Icon = section.icon
            return (
              <Link
                key={section.href}
                to={section.href}
                className="group flex min-h-20 min-w-0 items-center gap-3 rounded-md border border-border bg-surface p-3 transition-colors hover:border-accent hover:bg-surface-raised focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
              >
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
              </Link>
            )
          })}
        </div>
      </section>

      <section aria-labelledby="instance-state-title">
        <h2 id="instance-state-title" className="mb-3 text-base font-semibold">
          Состояние инстанса
        </h2>
        <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
          {overviewItems.map((item) => (
            <AdminMetric key={item.label} {...item} />
          ))}
        </div>
      </section>
    </div>
  )
}
