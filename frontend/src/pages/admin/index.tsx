import { Link } from 'react-router'
import { FileText, History, Settings, ShieldCheck, Users } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/shared/ui/card'

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

const readinessItems = [
  { label: 'База требований', status: 'актуально', icon: FileText },
  { label: 'Модель доступа', status: 'описана', icon: ShieldCheck },
  { label: 'Страницы MVP', status: 'синхронизированы', icon: History },
]

export function AdminPage() {
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
          <CardTitle className="text-base">Готовность MVP</CardTitle>
        </CardHeader>
        <CardContent className="grid gap-3 sm:grid-cols-3">
          {readinessItems.map((item) => {
            const Icon = item.icon
            return (
              <div key={item.label} className="rounded-md border border-border p-3">
                <div className="flex items-center gap-2 text-sm font-medium">
                  <Icon className="h-4 w-4 text-accent" />
                  {item.label}
                </div>
                <div className="mt-2 w-fit rounded bg-surface-raised px-2 py-1 text-xs text-text-muted">
                  {item.status}
                </div>
              </div>
            )
          })}
        </CardContent>
      </Card>
    </div>
  )
}
