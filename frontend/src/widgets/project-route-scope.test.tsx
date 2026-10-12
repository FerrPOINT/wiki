import { afterEach, expect, it, vi } from 'vitest'
import { cleanup, render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router'
import { useProjectRouteRef } from './namespace-context'
afterEach(() => {
  cleanup()
  vi.unstubAllEnvs()
})
it.each(['true', 'false'])('keeps the project resolver feature-gated: %s', (enabled) => {
  vi.stubEnv('VITE_NAMESPACE_ENABLED', enabled)
  const namespace = '22222222-2222-4222-8222-222222222222'
  function Scope() {
    const scope = useProjectRouteRef()
    return <span>{scope.ref?.namespace_id ?? 'legacy'}</span>
  }
  render(
    <MemoryRouter
      initialEntries={[
        `/tasks?space=LEGACY&registry_instance_id=11111111-1111-4111-8111-111111111111&namespace_id=${namespace}`,
      ]}
    >
      <Scope />
    </MemoryRouter>,
  )
  expect(screen.getByText(enabled === 'true' ? namespace : 'legacy')).toBeInTheDocument()
})
