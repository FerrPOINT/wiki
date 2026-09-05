import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router'
import { afterEach, describe, expect, it, vi } from 'vitest'

import { DocumentComposePage } from './'

const navigate = vi.hoisted(() => vi.fn())
const useCreateDocument = vi.hoisted(() => vi.fn())
const useSpaces = vi.hoisted(() => vi.fn())
const useTemplates = vi.hoisted(() => vi.fn())

const createDocumentMutate = vi.hoisted(() => vi.fn())

vi.mock('react-router', async (importOriginal) => {
  const actual = await importOriginal<typeof import('react-router')>()
  return {
    ...actual,
    useNavigate: () => navigate,
  }
})

vi.mock('@/shared/api/hooks', () => ({
  defaultSpaceKey: 'BASE',
  useCreateDocument,
  useSpaces,
  useTemplates,
}))

function setupCompose(
  createOverrides: Record<string, unknown> = {},
  initialRoute = '/documents/new?space=eng',
) {
  useSpaces.mockReturnValue({
    data: {
      spaces: [
        {
          id: 'space-eng',
          key: 'ENG',
          name: 'Engineering',
          description: 'Engineering docs',
          owner_id: 'user-admin',
          status: 'active',
          document_count: 0,
          member_count: 1,
          created_at: '2026-08-31T10:00:00Z',
          updated_at: '2026-08-31T10:00:00Z',
        },
      ],
    },
    isLoading: false,
    isError: false,
  })
  useTemplates.mockReturnValue({
    data: {
      templates: [
        {
          id: 'tpl-test-plan',
          name: 'План проверки',
          document_type: 'test_plan',
          body_markdown: '# Проверка\n\n- smoke',
        },
      ],
    },
    isLoading: false,
    isError: false,
  })
  useCreateDocument.mockReturnValue({
    mutate: createDocumentMutate,
    isPending: false,
    isError: false,
    error: null,
    ...createOverrides,
  })

  render(
    <MemoryRouter initialEntries={[initialRoute]}>
      <DocumentComposePage />
    </MemoryRouter>,
  )
}

describe('DocumentComposePage', () => {
  afterEach(() => {
    vi.clearAllMocks()
  })

  it('creates a document through the shared API hook and navigates to the result', () => {
    setupCompose()

    fireEvent.change(screen.getByLabelText('Название'), {
      target: { value: '  Новый регламент  ' },
    })
    fireEvent.change(screen.getByLabelText('Пространство'), {
      target: { value: 'eng' },
    })
    fireEvent.change(screen.getByLabelText('Markdown документа'), {
      target: { value: '# Новый регламент' },
    })
    fireEvent.click(screen.getByRole('button', { name: 'Сохранить черновик' }))

    expect(createDocumentMutate).toHaveBeenCalledWith(
      {
        spaceKey: 'ENG',
        body: {
          content_markdown: '# Новый регламент',
          document_type: 'page',
          parent_id: null,
          phase_key: null,
          slug: null,
          task_key: null,
          title: 'Новый регламент',
        },
      },
      { onSuccess: expect.any(Function) },
    )

    createDocumentMutate.mock.calls[0]?.[1]?.onSuccess({ slug: 'new-policy' })
    expect(navigate).toHaveBeenCalledWith('/documents/new-policy')
  })

  it('applies the requested template from URL without filling demo task links', async () => {
    setupCompose({}, '/documents/new?space=eng&template=tpl-test-plan')

    await waitFor(() => {
      expect(screen.getByLabelText('Markdown документа')).toHaveValue('# Проверка\n\n- smoke')
    })
    expect(screen.getByLabelText('Тип')).toHaveValue('test_plan')

    fireEvent.change(screen.getByLabelText('Название'), {
      target: { value: 'План проверки релиза' },
    })
    fireEvent.click(screen.getByRole('button', { name: 'Сохранить черновик' }))

    expect(createDocumentMutate).toHaveBeenCalledWith(
      {
        spaceKey: 'ENG',
        body: {
          content_markdown: '# Проверка\n\n- smoke',
          document_type: 'test_plan',
          parent_id: null,
          phase_key: null,
          slug: null,
          task_key: null,
          title: 'План проверки релиза',
        },
      },
      { onSuccess: expect.any(Function) },
    )
  })

  it('renders validation errors without leaking request identifiers', () => {
    setupCompose({
      isError: true,
      error: {
        code: 'VALIDATION_ERROR',
        message: 'Validation failed; details=title: required; requestId=req-42',
        details: [{ field: 'title', message: 'required' }],
      },
    })

    const alert = screen.getByRole('alert')
    expect(alert).toHaveTextContent('Проверьте заполнение полей: Название: required')
    expect(alert).not.toHaveTextContent('requestId')
  })
})
