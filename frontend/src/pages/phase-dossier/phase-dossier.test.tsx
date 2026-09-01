import { fireEvent, render, screen } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router'
import { afterEach, describe, expect, it, vi } from 'vitest'

import { PhaseDossierPage } from './'

const useLinkPhaseDocument = vi.hoisted(() => vi.fn())
const usePhase = vi.hoisted(() => vi.fn())
const usePhases = vi.hoisted(() => vi.fn())
const linkPhaseMutate = vi.hoisted(() => vi.fn())
const phaseRefetch = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({
  defaultSpaceKey: 'SDLC',
  useLinkPhaseDocument,
  usePhase,
  usePhases,
}))

const phasePage = {
  space_key: 'SDLC',
  phase_key: 'implementation',
  title: 'implementation',
  document_count: 0,
  evidence_count: 0,
  documents: [],
  evidence: [],
}

function renderPhasePage(linkState: Record<string, unknown> = {}) {
  usePhase.mockReturnValue({
    data: phasePage,
    isLoading: false,
    isError: false,
    refetch: phaseRefetch,
  })
  usePhases.mockReturnValue({ data: { phases: [] }, isLoading: false, isError: false })
  useLinkPhaseDocument.mockReturnValue({
    mutate: linkPhaseMutate,
    isPending: false,
    isError: false,
    error: null,
    ...linkState,
  })

  render(
    <MemoryRouter initialEntries={['/phases/implementation']}>
      <Routes>
        <Route path="/phases/:phaseId" element={<PhaseDossierPage />} />
      </Routes>
    </MemoryRouter>,
  )
}

describe('PhaseDossierPage', () => {
  afterEach(() => {
    vi.clearAllMocks()
  })

  it('links an existing document to the phase dossier', () => {
    linkPhaseMutate.mockImplementation((_variables, options) => options?.onSuccess?.())
    renderPhasePage()

    fireEvent.change(screen.getByLabelText('Документ для фазы'), {
      target: { value: ' product-requirements ' },
    })
    fireEvent.click(screen.getByRole('button', { name: 'Привязать' }))

    expect(linkPhaseMutate).toHaveBeenCalledWith(
      {
        spaceKey: 'SDLC',
        phaseKey: 'implementation',
        body: { document_id: 'product-requirements' },
      },
      expect.objectContaining({ onSuccess: expect.any(Function) }),
    )
    expect(screen.getByText('Документ привязан к фазе')).toBeInTheDocument()
    expect(screen.getByLabelText('Документ для фазы')).toHaveValue('')
  })

  it('renders phase link API errors', () => {
    renderPhasePage({
      isError: true,
      error: { code: 'FORBIDDEN', message: 'Forbidden' },
    })

    expect(screen.getByRole('alert')).toHaveTextContent('Недостаточно прав для действия')
  })
})
