import { fireEvent, render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router'
import { afterEach, describe, expect, it, vi } from 'vitest'

import { EvidencePage } from './'

const useCreateEvidence = vi.hoisted(() => vi.fn())
const useCreateFileEvidence = vi.hoisted(() => vi.fn())
const useEvidence = vi.hoisted(() => vi.fn())

const createFileMutate = vi.hoisted(() => vi.fn())
const createLinkMutate = vi.hoisted(() => vi.fn())
const refetchEvidence = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({
  defaultSpaceKey: 'SDLC',
  useCreateEvidence,
  useCreateFileEvidence,
  useEvidence,
}))

function setupEvidence() {
  useEvidence.mockReturnValue({
    data: {
      evidence: [
        {
          attachment_id: null,
          checksum: null,
          created_at: '2026-08-31T12:10:00Z',
          created_by: 'user-editor',
          document_id: 'product-requirements',
          evidence_type: 'external_url',
          id: 'evidence-link',
          phase_key: 'implementation',
          space_key: 'SDLC',
          task_key: 'SDLC-42',
          title: 'Сборка прошла',
          url: 'https://ci.local/jobs/wiki-smoke',
        },
        {
          attachment_id: 'attachment-1',
          checksum: 'sha256:abc123',
          created_at: '2026-08-31T12:15:00Z',
          created_by: 'user-editor',
          document_id: 'test-plan',
          evidence_type: 'uploaded_file',
          id: 'evidence-file',
          phase_key: 'testing',
          space_key: 'SDLC',
          task_key: 'SDLC-43',
          title: 'Лог сборки',
          url: null,
        },
      ],
    },
    isLoading: false,
    isError: false,
    refetch: refetchEvidence,
  })
  useCreateEvidence.mockReturnValue({
    mutate: createLinkMutate,
    isPending: false,
    error: null,
  })
  useCreateFileEvidence.mockReturnValue({
    mutate: createFileMutate,
    isPending: false,
    error: null,
  })

  render(
    <MemoryRouter>
      <EvidencePage />
    </MemoryRouter>,
  )
}

describe('EvidencePage', () => {
  afterEach(() => {
    vi.clearAllMocks()
  })

  it('renders evidence registry with document, task and phase links', () => {
    setupEvidence()

    expect(screen.getByRole('heading', { name: 'Материалы' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'Сборка прошла' })).toHaveAttribute(
      'href',
      'https://ci.local/jobs/wiki-smoke',
    )
    expect(screen.getByRole('link', { name: 'product-requirements' })).toHaveAttribute(
      'href',
      '/documents/product-requirements',
    )
    expect(screen.getByRole('link', { name: 'SDLC-42' })).toHaveAttribute('href', '/tasks/SDLC-42')
    expect(screen.getByRole('link', { name: 'implementation' })).toHaveAttribute(
      'href',
      '/phases/implementation',
    )
    expect(screen.getByText('ссылка')).toBeInTheDocument()
    expect(screen.getByText('файл')).toBeInTheDocument()
  })

  it('filters the visible registry without changing API-level filters', () => {
    setupEvidence()

    fireEvent.change(screen.getByLabelText('Поиск материалов'), {
      target: { value: 'лог' },
    })

    expect(screen.queryByText('Сборка прошла')).not.toBeInTheDocument()
    expect(screen.getByText('Лог сборки')).toBeInTheDocument()
    expect(useEvidence).toHaveBeenCalledWith({
      space: 'SDLC',
      document_id: undefined,
      task_key: undefined,
      phase_key: undefined,
    })
  })

  it('submits URL evidence through the shared API hook', () => {
    setupEvidence()

    fireEvent.submit(screen.getByRole('button', { name: 'Добавить материал' }).closest('form')!)
    expect(createLinkMutate).toHaveBeenCalledWith(
      {
        document_id: 'product-requirements',
        evidence_type: 'external_url',
        phase_key: 'implementation',
        space: 'SDLC',
        task_key: 'SDLC-42',
        title: 'Проверка сборки',
        url: 'https://ci.local/jobs/wiki-smoke',
      },
      { onSuccess: expect.any(Function) },
    )
  })

  it('submits file evidence through the shared API hook', async () => {
    const user = userEvent.setup()
    setupEvidence()

    await user.click(screen.getByRole('button', { name: 'Файл' }))
    const fileInput = screen.getByLabelText('Файл материала')
    const file = new File(['build ok'], 'build.log', { type: 'text/plain' })
    await user.upload(fileInput, file)
    fireEvent.submit(fileInput.closest('form')!)

    expect(createFileMutate).toHaveBeenCalledWith(
      {
        file,
        evidence: {
          document_id: 'product-requirements',
          phase_key: 'implementation',
          space: 'SDLC',
          task_key: 'SDLC-42',
          title: 'Проверка сборки',
        },
      },
      { onSuccess: expect.any(Function) },
    )
    expect(within(screen.getByRole('table')).getByText('Лог сборки')).toBeInTheDocument()
  })
})
