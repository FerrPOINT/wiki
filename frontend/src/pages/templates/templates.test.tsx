import { act, fireEvent, render, screen } from '@testing-library/react'
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
    expect(screen.getByRole('link', { name: 'Использовать шаблон Требования' })).toHaveAttribute(
      'href',
      '/documents/new?template=requirements',
    )
    expect(screen.queryByLabelText('Название шаблона')).not.toBeInTheDocument()
    expect(screen.queryByText('# Требования')).not.toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Показать содержимое шаблона Требования' }))
    expect(screen.getByText(/# Требования/)).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Новый шаблон' }))
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
    act(() => createTemplateMutate.mock.calls[0]?.[1].onSuccess())
    expect(screen.queryByLabelText('Название шаблона')).not.toBeInTheDocument()
  })

  it('filters templates by text and type without rendering every Markdown body', () => {
    setupTemplates({
      templatesOverrides: {
        data: {
          templates: [
            {
              id: 'requirements',
              name: 'Требования',
              document_type: 'requirements',
              body_markdown: '# Требования',
            },
            {
              id: 'release',
              name: 'Релизная заметка',
              document_type: 'release_note',
              body_markdown: '# Релиз',
            },
            {
              id: 'plan',
              name: 'План проверки',
              document_type: 'test_plan',
              body_markdown: '# Проверка',
            },
          ],
        },
      },
    })

    expect(screen.queryByText('# Релиз')).not.toBeInTheDocument()
    fireEvent.change(screen.getByRole('searchbox', { name: 'Найти шаблон' }), {
      target: { value: 'релиз' },
    })
    expect(
      screen.getByRole('link', { name: 'Использовать шаблон Релизная заметка' }),
    ).toBeInTheDocument()
    expect(
      screen.queryByRole('link', { name: 'Использовать шаблон Требования' }),
    ).not.toBeInTheDocument()

    fireEvent.change(screen.getByRole('combobox', { name: 'Тип шаблона' }), {
      target: { value: 'requirements' },
    })
    expect(screen.getByRole('status')).toHaveTextContent('шаблоны не найдены')
  })

  it('paginates a growing template catalog', () => {
    setupTemplates({
      templatesOverrides: {
        data: {
          templates: Array.from({ length: 25 }, (_, index) => ({
            id: `template-${index + 1}`,
            name: `Шаблон ${String(index + 1).padStart(2, '0')}`,
            document_type: 'page',
            body_markdown: `# Шаблон ${index + 1}`,
          })),
        },
      },
    })

    expect(screen.getAllByRole('link', { name: /Использовать шаблон/ })).toHaveLength(12)
    fireEvent.click(screen.getByRole('button', { name: 'Далее' }))
    expect(screen.getAllByRole('link', { name: /Использовать шаблон/ })).toHaveLength(12)
    fireEvent.click(screen.getByRole('button', { name: 'Далее' }))
    expect(screen.getAllByRole('link', { name: /Использовать шаблон/ })).toHaveLength(1)
    expect(screen.getByText('3 / 3')).toBeInTheDocument()
    fireEvent.change(screen.getByRole('searchbox', { name: 'Найти шаблон' }), {
      target: { value: 'Шаблон 01' },
    })
    expect(screen.queryByRole('navigation', { name: 'Страницы шаблонов' })).not.toBeInTheDocument()
    expect(screen.getAllByRole('link', { name: /Использовать шаблон/ })).toHaveLength(1)
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
