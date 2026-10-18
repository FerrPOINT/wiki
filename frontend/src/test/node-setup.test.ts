// @vitest-environment node

import i18n from 'i18next'
import { expect, it } from 'vitest'

it('keeps Node unit setup free of browser bootstrap', () => {
  expect(typeof window).toBe('undefined')
  expect(typeof document).toBe('undefined')
  expect(i18n.isInitialized).not.toBe(true)
  expect(globalThis.localStorage).toBeUndefined()
  expect(globalThis.sessionStorage).toBeUndefined()
})
