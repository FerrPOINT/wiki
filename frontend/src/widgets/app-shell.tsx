import { SidebarItem } from '@sdlc/ui/ui'
import { NamespaceShellContext } from './namespace-context'
import { useEffect, useState, type ElementType } from 'react'
import { NamespaceLink as Link } from '@sdlc/ui/ui'
import { NavLink, Outlet, useLocation } from 'react-router'
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
  PageFrame,
  PlatformHeader,
  PlatformMark,
  ThemeMenuItems,
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
    <SidebarItem asChild compact={responsiveLabel ? 'responsive' : false}>
      <NavLink to={to} end={to === '/'} onClick={onClick} aria-label={label} title={label}>
        <Icon aria-hidden />
        <span className="base-sidebar-item-label">{label}</span>
      </NavLink>
    </SidebarItem>
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
  const location = useLocation()
  const { data: user } = useCurrentUser()
  const logout = useLogout()
  const identity = user?.display_name?.trim() || user?.email || 'Пользователь'

  useEffect(() => {
    const desktop = window.matchMedia('(min-width: 768px)')
    const closeOnDesktop = () => {
      if (desktop.matches) setMobileMenuOpen(false)
    }
    desktop.addEventListener('change', closeOnDesktop)
    return () => desktop.removeEventListener('change', closeOnDesktop)
  }, [])

  const navItems = [...baseNavItems, ...(user?.is_system_admin ? adminNavItems : [])]
  const pageLayout =
    location.pathname === '/settings' || location.pathname === '/documents/new'
      ? 'reading'
      : /^\/(documents|tasks)\/[^/]+$/.test(location.pathname)
        ? 'detail-with-aside'
        : 'wide'

  return (
    <div className="min-h-screen bg-background text-text-primary">
      <PlatformHeader
        currentServiceKey="wiki"
        context={
          import.meta.env.VITE_NAMESPACE_ENABLED === 'true' ? <NamespaceShellContext /> : undefined
        }
        leading={
          <>
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
                className="!left-0 !top-0 !flex !h-dvh !max-h-dvh !w-[min(320px,calc(100%-2rem))] !max-w-none !translate-x-0 !translate-y-0 !flex-col !gap-0 !rounded-none !border-y-0 !border-l-0 !p-0 [&>button]:h-11 [&>button]:w-11"
              >
                <DialogHeader className="flex h-[var(--shell-header-height)] shrink-0 justify-center border-b border-border px-4 pr-14 text-left">
                  <DialogTitle className="text-base">
                    <span className="sr-only">Навигация Wiki</span>
                    <span className="flex items-center gap-3" aria-hidden>
                      <PlatformMark size="sm" withName={false} />
                      <span>Wiki</span>
                    </span>
                  </DialogTitle>
                </DialogHeader>
                <div className="min-h-0 flex-1 overflow-y-auto p-3">
                  <Button asChild className="mb-3 min-h-11 w-full gap-2">
                    <Link to="/documents/new" onClick={() => setMobileMenuOpen(false)}>
                      <FilePlus2 className="h-4 w-4" aria-hidden />
                      Новый документ
                    </Link>
                  </Button>
                  <NavigationList items={navItems} onNavigate={() => setMobileMenuOpen(false)} />
                </div>
              </DialogContent>
            </Dialog>

            <Link
              to="/"
              className="hidden h-11 min-w-11 items-center justify-center rounded-md focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus min-[360px]:flex md:h-10 md:min-w-10"
              aria-label="Wiki"
            >
              <PlatformMark size="sm" withName={false} />
            </Link>
          </>
        }
        actions={
          <>
            <Button asChild size="sm" className="hidden h-10 min-w-10 gap-2 px-3 md:inline-flex">
              <Link to="/documents/new" aria-label="Новый документ">
                <FilePlus2 className="h-4 w-4" aria-hidden />
                <span>Новый документ</span>
              </Link>
            </Button>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-11 w-11 md:h-10 md:w-10"
                  aria-label="Аккаунт"
                >
                  <User className="h-5 w-5" aria-hidden />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="w-64 max-w-[calc(100vw-2rem)]">
                <div className="break-words px-2 py-1.5 text-sm font-medium text-text-primary">
                  {identity}
                </div>
                {user?.email && user.email !== identity && (
                  <div className="break-words px-2 pb-2 text-xs text-text-muted">{user.email}</div>
                )}
                <ThemeMenuItems />
                <DropdownMenuItem
                  onSelect={() => logout.mutate()}
                  className="min-h-11 gap-2 text-text-secondary md:min-h-10"
                >
                  <LogOut className="h-4 w-4" aria-hidden />
                  <span>Выйти</span>
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </>
        }
      />
      <aside className="fixed bottom-0 left-0 top-[var(--shell-header-height)] z-20 hidden w-[var(--shell-sidebar-compact)] flex-col border-r border-border bg-surface md:flex xl:w-[var(--shell-sidebar-expanded)]">
        <div className="min-h-0 flex-1 overflow-y-auto p-2 xl:p-3">
          <NavigationList items={navItems} responsiveLabels />
        </div>
      </aside>
      <div className="md:pl-[var(--shell-sidebar-compact)] xl:pl-[var(--shell-sidebar-expanded)]">
        <main className="shell-main min-h-[calc(100dvh-var(--shell-header-height))]">
          <PageFrame mode={pageLayout}>
            <Outlet />
          </PageFrame>
        </main>
      </div>
    </div>
  )
}
