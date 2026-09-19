import { describe, expect, it } from 'vitest'
import { resolveSpaceKey } from './space-selection'

describe('resolveSpaceKey', () => {
  const spaces = [{ key: 'POLKA' }, { key: 'BASE' }]

  it('preserves an explicit space even when it is absent from the catalog', () => {
    expect(resolveSpaceKey(' qa ', spaces)).toBe('QA')
  })

  it('keeps legacy BASE links when that space exists', () => {
    expect(resolveSpaceKey(null, spaces)).toBe('BASE')
  })

  it('uses an available space when BASE is absent', () => {
    expect(resolveSpaceKey(null, [{ key: 'POLKA' }])).toBe('POLKA')
  })

  it('does not invent a space in an empty catalog', () => {
    expect(resolveSpaceKey(null, [])).toBe('')
  })
})
