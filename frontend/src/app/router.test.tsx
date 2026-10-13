import { createMemoryRouter } from 'react-router'
import { describe, expect, it } from 'vitest'
import { createWikiRoutes } from './router'

const namespacePaths = [
  '/namespace',
  '/documents/:documentId/revisions/:revisionId',
  '/managed-tasks/:trackerInstance/:taskId',
]

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
