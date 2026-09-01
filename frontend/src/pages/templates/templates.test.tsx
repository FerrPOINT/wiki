import { fireEvent, render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router'
import { afterEach, describe, expect, it, vi } from 'vitest'

import { TemplatesPage } from './'

const useCreateTemplate = vi.hoisted(() => vi.fn())
const useTemplates = vi.hoisted(() => vi.fn())

const createTemplateMutate = vi.hoisted(() => vi.fn())
const templatesRefetch = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({
  useCreateTemplate,
  useTemplates,
}))

function setupTemplates({
  createOverrides = {},
  templatesOverrides = {},
}: {
  createOverrides?: Record<string, unknown>
  templatesOverrides?: Record<string, unknown>
} = {}) {
  useTemplates.mockReturnValue({
    data: {
      templates: [
        {
          id: 'requirements',
          name: 'Требования',
          document_type: 'requirements',
          body_markdown: '# Требования\n\n## Контекст\n',
        },
      ],
    },
    isLoading: false,
    isError: false,
    refetch: templatesRefetch,
    ...templatesOverrides,
  })
  useCreateTemplate.mockReturnValue({
    mutate: createTemplateMutate,
    isPending: false,
    isError: false,
    error: null,
    ...createOverrides,
  })

  render(
    <MemoryRouter>
      <TemplatesPage />
    </MemoryRouter>,
  )
}

describe('TemplatesPage', () => {
  afterEach(() => {
    vi.clearAllMocks()
  })

  it('renders templates and creates a new one through the shared API hook', () => {
    setupTemplates()

    expect(screen.getByRole('heading', { name: 'Шаблоны' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'Использовать' })).toHaveAttribute(
      'href',
      '/documents/new?template=requirements',
    )
    expect(screen.getByLabelText('Название шаблона')).toHaveValue('')
    expect(screen.getByLabelText('Markdown шаблона')).toHaveValue('')

    fireEvent.change(screen.getByLabelText('Название шаблона'), {
      target: { value: '  План релиза  ' },
    })
    fireEvent.change(screen.getByLabelText('Тип документа'), {
      target: { value: 'release_note' },
    })
    fireEvent.change(screen.getByLabelText('Markdown шаблона'), {
      target: { value: '# Релиз\n\n## Проверки\n' },
    })
    fireEvent.submit(screen.getByLabelText('Markdown шаблона').closest('form')!)

    expect(createTemplateMutate).toHaveBeenCalledWith(
      {
        name: 'План релиза',
        document_type: 'release_note',
        body_markdown: '# Релиз\n\n## Проверки',
      },
      { onSuccess: expect.any(Function) },
    )
  })

  it('renders template query errors with retry', () => {
    setupTemplates({
      templatesOverrides: {
        data: undefined,
        isError: true,
        error: { code: 'FORBIDDEN', message: 'Forbidden' },
      },
    })

    expect(screen.getByRole('alert')).toHaveTextContent('Недостаточно прав для действия')
    fireEvent.click(screen.getByRole('button', { name: 'Повторить' }))
    expect(templatesRefetch).toHaveBeenCalled()
  })
})
