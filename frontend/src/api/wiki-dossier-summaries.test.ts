import { beforeEach, describe, expect, it, vi } from 'vitest'
import { listPhaseSummaries, listTaskSummaries } from './wiki'

const apiRequest = vi.hoisted(() => vi.fn())

vi.mock('./client', () => ({
  apiRequest,
  apiBlobRequest: vi.fn(),
}))

describe('dossier summary API paths', () => {
  beforeEach(() => vi.clearAllMocks())

  it('passes bounded task search and cursor as query parameters', () => {
    listTaskSummaries('TEAM SPACE', { limit: 12, cursor: 'TEAM-10', q: 'План' })

    const url = new URL(apiRequest.mock.calls[0]![0], 'http://wiki.test')
    expect(url.pathname).toBe('/api/v1/spaces/TEAM%20SPACE/task-summaries')
    expect(Object.fromEntries(url.searchParams)).toEqual({
      limit: '12',
      cursor: 'TEAM-10',
      q: 'План',
    })
  })

  it('uses the phase summary endpoint without empty optional filters', () => {
    listPhaseSummaries('BASE', { limit: 4, q: '' })

    const url = new URL(apiRequest.mock.calls[0]![0], 'http://wiki.test')
    expect(url.pathname).toBe('/api/v1/spaces/BASE/phase-summaries')
    expect(Object.fromEntries(url.searchParams)).toEqual({ limit: '4' })
  })
})
