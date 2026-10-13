import { describe, expect, it } from 'vitest'
import { wikiSearchScope } from './search-scope'

describe('Wiki search scope', () => {
  it('keeps the legacy default and lets global search retain a selected project', () => {
    expect(wikiSearchScope('', true)).toEqual({ scope: 'project', malformed: false })
    expect(wikiSearchScope('', false)).toEqual({ scope: 'all', malformed: false })
    expect(wikiSearchScope('?search_scope=all&project_scope=all', true)).toEqual({
      scope: 'all',
      malformed: false,
    })
  })
  it.each([
    '?search_scope=project',
    '?search_scope=other',
    '?search_scope=all&search_scope=project',
  ])('rejects unavailable or ambiguous scope %s', (search) => {
    expect(wikiSearchScope(search, false).malformed).toBe(true)
  })
})
