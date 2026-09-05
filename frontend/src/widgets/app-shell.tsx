import { useState, type ElementType } from 'react'
import { Link, Outlet, useLocation } from 'react-router'
import {
  ClipboardList,
  FileCheck2,
  FilePlus2,
  FileText,
  Files,
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
  X,
} from 'lucide-react'
import { Button, PlatformMark } from '@sdlc/ui/ui'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@sdlc/ui/ui'
import { ThemeToggle } from '@sdlc/ui/ui'
import { ServiceSwitcher } from '@sdlc/ui/ui'
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
  active,
  onClick,
}: {
  to: string
  icon: ElementType
  label: string
  active: boolean
  onClick?: () => void
}) {
  return (
    <Link
      to={to}
      onClick={onClick}
      className={`flex items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors ${
        active
          ? 'bg-surface-raised text-text-primary'
          : 'text-text-secondary hover:bg-surface-raised hover:text-text-primary'
      }`}
    >
      <Icon className="h-4 w-4 shrink-0" />
      <span className="truncate">{label}</span>
    </Link>
  )
}

export function AppShell() {
  const location = useLocation()
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false)
  const { data: user } = useCurrentUser()
  const logout = useLogout()

  const navItems = [...baseNavItems, ...(user?.is_system_admin ? adminNavItems : [])]

  function isActive(path: string) {
    if (path === '/') return location.pathname === '/'
    return location.pathname.startsWith(path)
  }

  function closeMobileMenu() {
    setMobileMenuOpen(false)
  }

  return (
    <div className="min-h-screen bg-background text-text-primary">
      <header className="sticky top-0 z-50 flex h-12 items-center justify-between border-b border-border bg-surface px-3 md:px-4">
        <div className="flex items-center gap-3 md:gap-4">
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8 md:hidden"
            onClick={() => setMobileMenuOpen((value) => !value)}
            aria-label="Открыть навигацию"
          >
            {mobileMenuOpen ? (
              <X className="h-[18px] w-[18px]" />
            ) : (
              <Menu className="h-[18px] w-[18px]" />
            )}
          </Button>
          <Link to="/" className="flex items-center gap-2 font-bold">
            <PlatformMark size="sm" withName={false} />
            <span className="hidden sm:inline">Wiki</span>
          </Link>
          <Link
            to="/spaces"
            className="hidden items-center gap-2 rounded-md px-2 py-1 text-sm text-text-secondary hover:bg-surface-raised hover:text-text-primary sm:flex"
          >
            <Files className="h-4 w-4" />
            <span>Документы</span>
          </Link>
          <Link
            to="/search"
            className="hidden items-center gap-2 rounded-md px-2 py-1 text-sm text-text-secondary hover:bg-surface-raised hover:text-text-primary sm:flex"
          >
            <Search className="h-4 w-4" />
            <span>Поиск</span>
          </Link>
        </div>

        <div className="flex items-center gap-2 md:gap-3">
          <Button asChild size="sm" className="h-7 gap-1 px-2.5 text-xs">
            <Link to="/documents/new">
              <FilePlus2 className="h-3.5 w-3.5" />
              <span className="hidden sm:inline">Новый документ</span>
            </Link>
          </Button>
          <ServiceSwitcher currentKey="wiki" />
          <ThemeToggle />
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="icon" className="h-8 w-8" aria-label="Аккаунт">
                <User className="h-[18px] w-[18px]" />
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
                <LogOut className="h-4 w-4" />
                <span>Выйти</span>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </header>

      <div className="flex min-h-[calc(100vh-3rem)]">
        <aside className="hidden w-60 shrink-0 flex-col gap-2 border-r border-border bg-surface p-3 md:flex">
          {navItems.map((item) => (
            <SidebarLink
              key={item.to}
              to={item.to}
              icon={item.icon}
              label={item.label}
              active={isActive(item.to)}
            />
          ))}
        </aside>

        {mobileMenuOpen && (
          <div className="fixed inset-0 z-40 md:hidden">
            <div
              className="absolute inset-0 bg-black/40"
              onClick={() => setMobileMenuOpen(false)}
            />
            <aside className="absolute left-0 top-0 h-full w-64 border-r border-border bg-surface p-3 pt-14 shadow-lg">
              {navItems.map((item) => (
                <SidebarLink
                  key={item.to}
                  to={item.to}
                  icon={item.icon}
                  label={item.label}
                  active={isActive(item.to)}
                  onClick={closeMobileMenu}
                />
              ))}
            </aside>
          </div>
        )}

        <main className="min-w-0 flex-1 p-4 md:p-6">
          <div className="mx-auto w-full max-w-7xl">
            <Outlet />
          </div>
        </main>
      </div>
    </div>
  )
}
