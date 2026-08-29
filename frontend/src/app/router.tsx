import { lazy, Suspense, type ReactElement } from 'react'
import { createBrowserRouter, Navigate } from 'react-router'
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
const RegisterPage = lazy(() =>
  import('@/pages/register').then((m) => ({ default: m.RegisterPage })),
)
const AdminPage = lazy(() => import('@/pages/admin').then((m) => ({ default: m.AdminPage })))

function PageLoader() {
  return (
    <div className="flex items-center justify-center py-16 text-sm text-text-muted">Загрузка...</div>
  )
}

const withSuspense = (element: ReactElement) => (
  <Suspense fallback={<PageLoader />}>{element}</Suspense>
)

export const router = createBrowserRouter([
  {
    element: <RequireAuth />,
    children: [
      {
        element: <AppShell />,
        children: [
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
  { path: '/register', element: withSuspense(<RegisterPage />) },
  { path: '*', element: <Navigate to="/" replace /> },
])
