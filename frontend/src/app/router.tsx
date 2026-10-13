import { LoadingState } from '@sdlc/ui/ui'
import { createProductRouter, PlatformRoute } from '@sdlc/ui/platform'
import { lazy, Suspense, type ReactElement } from 'react'
import { Navigate, type RouteObject } from 'react-router'
import { RequireAuth } from '@/shared/auth/require-auth'
import { AppShell } from '@/widgets/app-shell'

const DashboardPage = lazy(() =>
  import('@/pages/dashboard').then((m) => ({ default: m.DashboardPage })),
)
const SpacesPage = lazy(() => import('@/pages/spaces').then((m) => ({ default: m.SpacesPage })))
const DocumentComposePage = lazy(() =>
  import('@/pages/document-compose').then((m) => ({ default: m.DocumentComposePage })),
)
const DocumentPage = lazy(() =>
  import('@/pages/document').then((m) => ({ default: m.DocumentPage })),
)
const TaskDossiersPage = lazy(() =>
  import('@/pages/task-dossier').then((m) => ({ default: m.TaskDossiersPage })),
)
const TaskDossierPage = lazy(() =>
  import('@/pages/task-dossier').then((m) => ({ default: m.TaskDossierPage })),
)
const PhaseDossiersPage = lazy(() =>
  import('@/pages/phase-dossier').then((m) => ({ default: m.PhaseDossiersPage })),
)
const PhaseDossierPage = lazy(() =>
  import('@/pages/phase-dossier').then((m) => ({ default: m.PhaseDossierPage })),
)
const WikiSearchPage = lazy(() =>
  import('@/pages/wiki-search').then((m) => ({ default: m.WikiSearchPage })),
)
const EvidencePage = lazy(() =>
  import('@/pages/evidence').then((m) => ({ default: m.EvidencePage })),
)
const TemplatesPage = lazy(() =>
  import('@/pages/templates').then((m) => ({ default: m.TemplatesPage })),
)
const AuditLogPage = lazy(() =>
  import('@/pages/audit-log').then((m) => ({ default: m.AuditLogPage })),
)
const UsersPage = lazy(() => import('@/pages/users').then((m) => ({ default: m.UsersPage })))
const SettingsPage = lazy(() =>
  import('@/pages/settings').then((m) => ({ default: m.SettingsPage })),
)
const LoginPage = lazy(() => import('@/pages/login').then((m) => ({ default: m.LoginPage })))
const SsoCallbackPage = lazy(() =>
  import('@/pages/sso-callback').then((m) => ({ default: m.SsoCallbackPage })),
)
const AdminPage = lazy(() => import('@/pages/admin').then((m) => ({ default: m.AdminPage })))
const NamespacePage = lazy(() =>
  import('@/pages/namespace').then((m) => ({ default: m.NamespacePage })),
)
const ProjectsPage = lazy(() =>
  import('@/pages/workspaces').then((m) => ({ default: m.ProjectsPage })),
)
const WorkspacePage = lazy(() =>
  import('@/pages/workspaces').then((m) => ({ default: m.WorkspacePage })),
)
const DocumentRevisionPage = lazy(() =>
  import('@/pages/document-revision').then((m) => ({ default: m.DocumentRevisionPage })),
)
const ManagedTaskPage = lazy(() =>
  import('@/pages/managed-task').then((m) => ({ default: m.ManagedTaskPage })),
)

function PageLoader() { return <LoadingState /> }

const withSuspense = (element: ReactElement) => (
  <Suspense fallback={<PageLoader />}><PlatformRoute>{element}</PlatformRoute></Suspense>
)

export function createWikiRoutes(namespaceEnabled: boolean): RouteObject[] {
  const namespaceRoutes: RouteObject[] = namespaceEnabled
    ? [
        { path: '/namespace', element: withSuspense(<NamespacePage />) },
        { path: '/projects', element: withSuspense(<ProjectsPage />) },
        { path: '/workspaces/:registry/:namespace', element: withSuspense(<WorkspacePage />) },
        {
          path: '/workspaces/:registry/:namespace/documents',
          element: withSuspense(<WorkspacePage documents />),
        },
        {
          path: '/workspaces/:registry/:namespace/tasks',
          element: withSuspense(<TaskDossiersPage />),
        },
        {
          path: '/workspaces/:registry/:namespace/phases',
          element: withSuspense(<PhaseDossiersPage />),
        },
        {
          path: '/workspaces/:registry/:namespace/evidence',
          element: withSuspense(<EvidencePage />),
        },
        {
          path: '/workspaces/:registry/:namespace/search',
          element: withSuspense(<WikiSearchPage />),
        },
        {
          path: '/documents/:documentId/revisions/:revisionId',
          element: withSuspense(<DocumentRevisionPage />),
        },
        {
          path: '/managed-tasks/:trackerInstance/:taskId',
          element: withSuspense(<ManagedTaskPage />),
        },
      ]
    : []

  return [
    {
      element: <RequireAuth />,
      children: [
        {
          element: <AppShell />,
          children: [
            ...namespaceRoutes,
            { path: '/', element: withSuspense(<DashboardPage />) },
            { path: '/spaces', element: withSuspense(<SpacesPage />) },
            { path: '/documents/new', element: withSuspense(<DocumentComposePage />) },
            { path: '/documents/:documentId', element: withSuspense(<DocumentPage />) },
            { path: '/tasks', element: withSuspense(<TaskDossiersPage />) },
            { path: '/tasks/:taskKey', element: withSuspense(<TaskDossierPage />) },
            { path: '/phases', element: withSuspense(<PhaseDossiersPage />) },
            { path: '/phases/:phaseId', element: withSuspense(<PhaseDossierPage />) },
            { path: '/evidence', element: withSuspense(<EvidencePage />) },
            { path: '/templates', element: withSuspense(<TemplatesPage />) },
            { path: '/audit-log', element: withSuspense(<AuditLogPage />) },
            { path: '/users', element: withSuspense(<UsersPage />) },
            { path: '/settings', element: withSuspense(<SettingsPage />) },
            { path: '/search', element: withSuspense(<WikiSearchPage />) },
            { path: '/admin', element: withSuspense(<AdminPage />) },
          ],
        },
      ],
    },
    { path: '/login', element: withSuspense(<LoginPage />) },
    { path: '/sso/callback', element: withSuspense(<SsoCallbackPage />) },
    { path: '/register', element: <Navigate to="/login" replace /> },
    { path: '*', element: <Navigate to="/" replace /> },
  ]
}

export const router = createProductRouter(
  createWikiRoutes(import.meta.env.VITE_NAMESPACE_ENABLED === 'true'),
  { serviceKey: 'wiki', portalOrigin: import.meta.env.VITE_PLATFORM_ORIGIN },
)
