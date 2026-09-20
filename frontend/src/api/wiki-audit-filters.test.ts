import { describe, expect, it, vi } from 'vitest'

import { listAuditLog } from './wiki'

const apiRequest = vi.hoisted(() => vi.fn())

vi.mock('./client', () => ({ apiRequest, apiBlobRequest: vi.fn() }))

describe('listAuditLog', () => {
  it('serializes exact server filters before cursor pagination', async () => {
    await listAuditLog({
      limit: 20,
      cursor: 'next page',
      action: 'document.publish',
      entity_type: 'document',
      actor_id: '00000000-0000-4000-8000-000000000001',
      from: '2026-09-20T07:00:00.000Z',
      to: '2026-09-21T07:00:00.000Z',
    })

    const url = new URL(apiRequest.mock.calls[0]![0], 'http://wiki.local')
    expect(url.pathname).toBe('/api/v1/audit-log')
    expect(Object.fromEntries(url.searchParams)).toEqual({
      limit: '20',
      cursor: 'next page',
      action: 'document.publish',
      entity_type: 'document',
      actor_id: '00000000-0000-4000-8000-000000000001',
      from: '2026-09-20T07:00:00.000Z',
      to: '2026-09-21T07:00:00.000Z',
    })
  })

  it('omits cleared filters', async () => {
    await listAuditLog({ limit: 20, action: '', entity_type: '', actor_id: '', from: '', to: '' })

    expect(apiRequest).toHaveBeenLastCalledWith('/api/v1/audit-log?limit=20')
  })
})
