import { useState, type ElementType } from 'react'
import { Link, NavLink, Outlet } from 'react-router'
import {
  ClipboardList,
  FileCheck2,
  FilePlus2,
  FileText,
  GitBranch,
  History,
  Home,
  Library,
  LogOut,
  Menu,
  Search,
  Settings,
  ShieldCheck,
  User,
  Users,
} from 'lucide-react'
import {
  Button,
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  PlatformMark,
  ServiceSwitcher,
  ThemeToggle,
} from '@sdlc/ui/ui'
import { useCurrentUser, useLogout } from '@/shared/api/hooks'

type NavItem = {
  to: string
  icon: ElementType
  label: string
}

const baseNavItems: NavItem[] = [
  { to: '/', icon: Home, label: 'Обзор' },
  { to: '/spaces', icon: Library, label: 'Пространства' },
  { to: '/tasks', icon: ClipboardList, label: 'Задачи' },
  { to: '/phases', icon: GitBranch, label: 'Фазы' },
  { to: '/evidence', icon: FileCheck2, label: 'Материалы' },
  { to: '/templates', icon: FileText, label: 'Шаблоны' },
  { to: '/search', icon: Search, label: 'Поиск' },
]

const adminNavItems: NavItem[] = [
  { to: '/audit-log', icon: History, label: 'Аудит' },
  { to: '/users', icon: Users, label: 'Пользователи' },
  { to: '/settings', icon: Settings, label: 'Настройки' },
  { to: '/admin', icon: ShieldCheck, label: 'Администрирование' },
]

function SidebarLink({
  to,
  icon: Icon,
  label,
  onClick,
  responsiveLabel = false,
}: {
  to: string
  icon: ElementType
  label: string
  onClick?: () => void
  responsiveLabel?: boolean
}) {
  return (
    <NavLink
      to={to}
      end={to === '/'}
      onClick={onClick}
      title={responsiveLabel ? label : undefined}
      className={({ isActive }) =>
        `flex min-h-11 items-center gap-3 rounded-md px-3 text-sm transition-colors ${
          responsiveLabel ? 'md:justify-center md:px-2 xl:justify-start xl:px-3' : ''
        } ${
          isActive
            ? 'bg-surface-raised text-text-primary'
            : 'text-text-secondary hover:bg-surface-raised hover:text-text-primary'
        }`
      }
    >
      <Icon className="h-5 w-5 shrink-0" aria-hidden />
      <span className={responsiveLabel ? 'md:sr-only xl:not-sr-only xl:truncate' : 'truncate'}>
        {label}
      </span>
    </NavLink>
  )
}

function NavigationList({
  items,
  onNavigate,
  responsiveLabels = false,
}: {
  items: NavItem[]
  onNavigate?: () => void
  responsiveLabels?: boolean
}) {
  return (
    <nav className="flex flex-col gap-1" aria-label="Основная навигация">
      {items.map((item) => (
        <SidebarLink
          key={item.to}
          {...item}
          onClick={onNavigate}
          responsiveLabel={responsiveLabels}
        />
      ))}
    </nav>
  )
}

export function AppShell() {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false)
  const { data: user } = useCurrentUser()
  const logout = useLogout()

  const navItems = [...baseNavItems, ...(user?.is_system_admin ? adminNavItems : [])]

  return (
    <div className="min-h-screen bg-background text-text-primary">
      <aside className="fixed inset-y-0 left-0 z-40 hidden w-[72px] flex-col border-r border-border bg-surface md:flex xl:w-[264px]">
        <div className="flex h-[60px] shrink-0 items-center justify-center border-b border-border px-3 xl:justify-start xl:px-5">
          <div className="flex min-w-0 items-center gap-3" role="img" aria-label="Wiki">
            <PlatformMark size="sm" withName={false} />
            <div className="hidden min-w-0 xl:block">
              <div className="truncate text-sm font-semibold">Wiki</div>
              <div className="truncate text-xs text-text-muted">База знаний</div>
            </div>
          </div>
        </div>
        <div className="min-h-0 flex-1 overflow-y-auto p-2 xl:p-3">
          <NavigationList items={navItems} responsiveLabels />
        </div>
      </aside>

      <div className="min-h-screen md:pl-[72px] xl:pl-[264px]">
        <header className="sticky top-0 z-30 flex h-[60px] items-center justify-between border-b border-border bg-surface px-3 md:px-4">
          <div className="flex min-w-0 items-center gap-2">
            <Dialog open={mobileMenuOpen} onOpenChange={setMobileMenuOpen}>
              <DialogTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-11 w-11 md:hidden"
                  aria-label="Открыть навигацию"
                >
                  <Menu className="h-5 w-5" aria-hidden />
                </Button>
              </DialogTrigger>
              <DialogContent
                aria-describedby={undefined}
                className="!left-0 !top-0 !flex !h-dvh !max-h-dvh !w-[min(320px,calc(100%-2rem))] !max-w-none !translate-x-0 !translate-y-0 !flex-col !gap-0 !rounded-none !border-y-0 !border-l-0 !p-0 [&>button]:h-10 [&>button]:w-10"
              >
                <DialogHeader className="flex h-[60px] shrink-0 justify-center border-b border-border px-4 pr-14 text-left">
                  <DialogTitle className="text-base">
                    <span className="sr-only">Навигация Wiki</span>
                    <span className="flex items-center gap-3" aria-hidden>
                      <PlatformMark size="sm" withName={false} />
                      <span>Wiki</span>
                    </span>
                  </DialogTitle>
                </DialogHeader>
                <div className="min-h-0 flex-1 overflow-y-auto p-3">
                  <NavigationList items={navItems} onNavigate={() => setMobileMenuOpen(false)} />
                </div>
              </DialogContent>
            </Dialog>

            <Link
              to="/"
              className="flex h-10 min-w-10 items-center justify-center gap-2 px-2 md:hidden"
              aria-label="Wiki"
            >
              <PlatformMark size="sm" withName={false} />
              <span className="hidden truncate text-sm font-semibold min-[420px]:inline">Wiki</span>
            </Link>
            <span className="hidden truncate text-sm font-medium text-text-secondary md:inline">
              База знаний
            </span>
          </div>

          <div className="flex shrink-0 items-center gap-1 sm:gap-2">
            <Button asChild size="sm" className="h-10 min-w-10 gap-1 px-2 sm:px-3">
              <Link to="/documents/new" aria-label="Новый документ">
                <FilePlus2 className="h-4 w-4" aria-hidden />
                <span className="hidden sm:inline">Новый документ</span>
              </Link>
            </Button>
            <ServiceSwitcher currentKey="wiki" />
            <div className="[&_button]:h-10 [&_button]:w-10">
              <ThemeToggle />
            </div>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="icon" className="h-10 w-10" aria-label="Аккаунт">
                  <User className="h-5 w-5" aria-hidden />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="w-56">
                <div className="px-2 py-1.5 text-sm font-medium text-text-primary">
                  {user?.display_name ?? user?.email ?? 'Пользователь'}
                </div>
                <div className="px-2 pb-2 text-xs text-text-muted">{user?.email}</div>
                <DropdownMenuItem
                  onClick={() => logout.mutate()}
                  className="gap-2 text-text-secondary"
                >
                  <LogOut className="h-4 w-4" aria-hidden />
                  <span>Выйти</span>
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </header>

        <main className="min-w-0 p-4 md:p-6">
          <Outlet />
        </main>
      </div>
    </div>
  )
}
