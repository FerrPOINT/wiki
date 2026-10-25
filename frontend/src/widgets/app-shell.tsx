import { useApplicationAccess } from '@pdlc/ui/auth'
import {
  AccountMenu,
  AppShell as BaseAppShell,
  SidebarNavigationGroup,
  useSidebarDisclosure,
  SidebarItem,
  NamespaceLink as Link,
  NamespaceNavLink as NavLink,
  Button,
  PlatformHeader,
  PlatformMark,
} from '@pdlc/ui/ui'

import { NamespaceShellContext } from './namespace-context'
import { type ElementType } from 'react'

import { Outlet, useLocation } from 'react-router'

import { useProjectDisclosures } from '@pdlc/ui/lib'
import { WorkspaceNavigation, useDocumentDisclosures } from './workspace-navigation'
import { ProjectBoundary } from './project-boundary'
import { FileCheck2, FilePlus2, FileText, Home, Library, Search, ShieldCheck } from 'lucide-react'

import { useCurrentUser, useLogout } from '@/shared/api/hooks'

type NavItem = {
  to: string
  icon: ElementType
  label: string
}

const baseNavItems: NavItem[] = [
  { to: '/', icon: Home, label: 'Обзор' },
  { to: '/spaces', icon: Library, label: 'Пространства' },
  { to: '/evidence', icon: FileCheck2, label: 'Материалы' },
  { to: '/search', icon: Search, label: 'Поиск' },
]

const adminNavItems: NavItem[] = [{ to: '/admin', icon: ShieldCheck, label: 'Администрирование' }]

function SidebarLink({
  to,
  icon: Icon,
  label,
  onClick,
  compact = false,
}: {
  to: string
  icon: ElementType
  label: string
  onClick?: () => void
  compact?: boolean
}) {
  return (
    <SidebarItem asChild compact={compact}>
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
  compact = false,
}: {
  items: NavItem[]
  onNavigate?: () => void
  compact?: boolean
}) {
  return (
    <nav className="flex flex-col gap-1" aria-label="Основная навигация">
      {items.map((item) => (
        <SidebarLink key={item.to} {...item} onClick={onNavigate} compact={compact} />
      ))}
    </nav>
  )
}

export function AppShell() {
  const location = useLocation()
  const { data: user } = useCurrentUser()
  const logout = useLogout()
  const identity = user?.display_name?.trim() || user?.email || 'Пользователь'

  const namespaceEnabled = import.meta.env.VITE_NAMESPACE_ENABLED === 'true'
  const disclosures = useProjectDisclosures('wiki:collapsed-projects:v1')
  const documents = useDocumentDisclosures()
  const baseItems = namespaceEnabled
    ? baseNavItems.flatMap((item) =>
        item.to === '/'
          ? [item, { to: '/projects', icon: Library, label: 'Проекты' }]
          : ['/tasks', '/phases', '/evidence'].includes(item.to)
            ? []
            : [item],
      )
    : baseNavItems
  const navItems = baseItems
  const utilityItems = [
    { to: '/templates', icon: FileText, label: 'Шаблоны' },
    ...(useApplicationAccess()?.permits([], true) ? adminNavItems : []),
  ]
  const utilityActive = ['/templates', '/admin', '/users', '/settings', '/audit-log'].includes(
    location.pathname,
  )
  const utilities = useSidebarDisclosure(utilityActive ? location.pathname : null)
  const utilityNavigation = (responsive = false, onNavigate?: () => void) => (
    <SidebarNavigationGroup
      label="Служебное"
      active={utilityActive}
      compact={responsive}
      {...utilities}
    >
      <NavigationList items={utilityItems} compact={responsive} onNavigate={onNavigate} />
    </SidebarNavigationGroup>
  )
  const pageLayout =
    location.pathname === '/settings' || location.pathname === '/documents/new'
      ? 'reading'
      : /^\/(documents|tasks)\/[^/]+$/.test(location.pathname)
        ? 'detail-with-aside'
        : 'wide'

  return (
    <BaseAppShell
      currentServiceKey="wiki"
      title="Wiki"
      layout={pageLayout}
      collapsibleSidebar
      sidebarStorageKey="wiki:sidebar-collapsed"
      renderNavigation={({ compact, mobile, onNavigate }) => (
        <>
          {mobile && (
            <Button asChild className="mb-3 w-full gap-2">
              <Link to="/documents/new" onClick={onNavigate}>
                <FilePlus2 className="h-4 w-4" aria-hidden />
                Новый документ
              </Link>
            </Button>
          )}
          <NavigationList items={navItems} compact={compact} onNavigate={onNavigate} />
          {namespaceEnabled && (
            <WorkspaceNavigation
              disclosures={disclosures}
              documents={documents}
              compact={compact}
              onNavigate={onNavigate}
            />
          )}
          {utilityNavigation(compact, onNavigate)}
        </>
      )}
      renderHeader={(navigationTrigger) => (
        <PlatformHeader
          currentServiceKey="wiki"
          context={
            import.meta.env.VITE_NAMESPACE_ENABLED === 'true' ? (
              <NamespaceShellContext />
            ) : undefined
          }
          leading={
            <>
              {navigationTrigger}

              <Button asChild variant="ghost" size="icon">
                <Link to="/" className="hidden min-[360px]:flex" aria-label="Wiki">
                  <PlatformMark size="shell" withName={false} />
                </Link>
              </Button>
            </>
          }
          actions={
            <>
              <Button asChild size="sm" className="hidden min-w-10 gap-2 md:inline-flex">
                <Link to="/documents/new" aria-label="Новый документ">
                  <FilePlus2 className="h-4 w-4" aria-hidden />
                  <span>Новый документ</span>
                </Link>
              </Button>
              <AccountMenu
                account={{
                  label: identity,
                  secondary: user?.email && user.email !== identity ? user.email : undefined,
                  onLogout: () => logout.mutate(),
                }}
              />
            </>
          }
        />
      )}
    >
      {namespaceEnabled ? (
        <ProjectBoundary>
          <Outlet />
        </ProjectBoundary>
      ) : (
        <Outlet />
      )}
    </BaseAppShell>
  )
}
