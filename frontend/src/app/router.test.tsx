import { createMemoryRouter, RouterProvider } from 'react-router'
import { act, cleanup, render, screen, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it } from 'vitest'
import { createWikiRoutes } from './router'

const namespacePaths = [
  '/namespace',
  '/documents/:documentId/revisions/:revisionId',
  '/managed-tasks/:trackerInstance/:taskId',
]

afterEach(cleanup)

it.each([false, true])(
  'replaces legacy registration with central login (namespace=%s)',
  async (enabled) => {
    // Keep the real redirect and avoid starting SSO at its destination.
    const router = createMemoryRouter(
      [
        { path: '/previous', element: <h1>Previous page</h1> },
        ...createWikiRoutes(enabled).map((route) =>
          route.path === '/login' ? { ...route, element: <h1>Central login</h1> } : route,
        ),
      ],
      { initialEntries: ['/previous', '/register'], initialIndex: 1 },
    )
    try {
      render(<RouterProvider router={router} />)
      expect(await screen.findByRole('heading', { name: 'Central login' })).toBeInTheDocument()
      expect(router.state.location.pathname).toBe('/login')
      expect(router.state.historyAction).toBe('REPLACE')
      expect(screen.queryByRole('textbox')).toBeNull()
      await act(async () => {
        await router.navigate(-1)
      })
      await waitFor(() => expect(router.state.location.pathname).toBe('/previous'))
      expect(screen.getByRole('heading', { name: 'Previous page' })).toBeInTheDocument()
    } finally {
      router.dispose()
    }
  },
)

function matchedPaths(pathname: string, namespaceEnabled: boolean) {
  const router = createMemoryRouter(createWikiRoutes(namespaceEnabled), {
    initialEntries: [pathname],
  })
  return router.state.matches.map((match) => match.route.path)
}

describe('Namespace UI routes', () => {
  it('does not register Namespace routes when the feature flag is disabled', () => {
    for (const pathname of [
      '/namespace',
      '/documents/document-1/revisions/revision-1',
      '/managed-tasks/tracker-1/task-1',
    ]) {
      const paths = matchedPaths(pathname, false)
      expect(namespacePaths.some((route) => paths.includes(route))).toBe(false)
    }
  })

  it('registers Namespace routes when enabled and keeps legacy document routes available', () => {
    expect(matchedPaths('/namespace', true)).toContain('/namespace')
    expect(matchedPaths('/documents/document-1/revisions/revision-1', true)).toContain(
      '/documents/:documentId/revisions/:revisionId',
    )
    expect(matchedPaths('/managed-tasks/tracker-1/task-1', true)).toContain(
      '/managed-tasks/:trackerInstance/:taskId',
    )
    expect(matchedPaths('/documents/document-1', false)).toContain('/documents/:documentId')
  })
})
