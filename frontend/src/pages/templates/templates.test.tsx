import { act, fireEvent, render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router'
import { afterEach, describe, expect, it, vi } from 'vitest'

import { TemplatesPage } from './'

const useCreateTemplate = vi.hoisted(() => vi.fn())
const useCurrentUser = vi.hoisted(() => vi.fn())
const useTemplates = vi.hoisted(() => vi.fn())

const createTemplateMutate = vi.hoisted(() => vi.fn())
const templatesRefetch = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({
  useCreateTemplate,
  useCurrentUser,
  useTemplates,
}))

function setupTemplates({
  createOverrides = {},
  currentUserOverrides = {},
  templatesOverrides = {},
}: {
  createOverrides?: Record<string, unknown>
  currentUserOverrides?: Record<string, unknown>
  templatesOverrides?: Record<string, unknown>
} = {}) {
  useCurrentUser.mockReturnValue({
    data: { is_system_admin: true },
    isLoading: false,
    ...currentUserOverrides,
  })
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
    vi.resetAllMocks()
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
    act(() =>
      createTemplateMutate.mock.calls[0]?.[1].onSuccess({
        id: 'release-plan',
        name: 'План релиза',
        document_type: 'release_note',
        body_markdown: '# Релиз',
      }),
    )
    expect(screen.queryByLabelText('Название шаблона')).not.toBeInTheDocument()
    expect(screen.getByRole('status')).toHaveTextContent('Шаблон «План релиза» создан')
    expect(screen.getByRole('link', { name: 'Использовать' })).toHaveAttribute(
      'href',
      '/documents/new?template=release-plan',
    )
  })

  it('keeps template use available but hides creation for a viewer', () => {
    setupTemplates({ currentUserOverrides: { data: { is_system_admin: false } } })

    expect(screen.queryByRole('button', { name: 'Новый шаблон' })).not.toBeInTheDocument()
    expect(screen.queryByRole('form', { name: 'Создание шаблона' })).not.toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'Использовать шаблон Требования' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'Создать документ' })).toBeInTheDocument()
  })

  it('confirms creation even when the current search excludes the new template', () => {
    const created = {
      id: 'new-template',
      name: 'Новый шаблон',
      document_type: 'test_plan',
      body_markdown: '# Новый',
    }
    createTemplateMutate.mockImplementation((_payload, options) => options.onSuccess(created))
    setupTemplates()
    fireEvent.change(screen.getByRole('searchbox', { name: 'Найти шаблон' }), {
      target: { value: 'Непохожее' },
    })
    fireEvent.click(screen.getByRole('button', { name: 'Новый шаблон' }))
    fireEvent.change(screen.getByLabelText('Название шаблона'), {
      target: { value: created.name },
    })
    fireEvent.change(screen.getByLabelText('Markdown шаблона'), {
      target: { value: created.body_markdown },
    })
    fireEvent.submit(screen.getByRole('form', { name: 'Создание шаблона' }))

    expect(screen.getByText('Шаблон «Новый шаблон» создан')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'Использовать' })).toHaveAttribute(
      'href',
      '/documents/new?template=new-template',
    )
    expect(screen.queryByRole('form', { name: 'Создание шаблона' })).not.toBeInTheDocument()
  })

  it('locks the create form while saving and keeps its draft on error', () => {
    setupTemplates()
    fireEvent.click(screen.getByRole('button', { name: 'Новый шаблон' }))
    fireEvent.change(screen.getByLabelText('Название шаблона'), {
      target: { value: 'Мой шаблон' },
    })
    fireEvent.change(screen.getByLabelText('Markdown шаблона'), {
      target: { value: '# Текст' },
    })
    useCreateTemplate.mockReturnValue({
      mutate: createTemplateMutate,
      isPending: true,
      isError: false,
      error: null,
    })
    fireEvent.change(screen.getByRole('searchbox', { name: 'Найти шаблон' }), {
      target: { value: 'refresh' },
    })
    expect(screen.getByRole('button', { name: 'Свернуть форму' })).toBeDisabled()
    expect(screen.getByRole('button', { name: 'Свернуть' })).toBeDisabled()
    expect(screen.getByLabelText('Название шаблона')).toBeDisabled()
    expect(screen.getByLabelText('Тип документа')).toBeDisabled()
    expect(screen.getByLabelText('Markdown шаблона')).toBeDisabled()
    expect(screen.getByRole('button', { name: 'Создаём...' })).toBeDisabled()
    fireEvent.submit(screen.getByRole('form', { name: 'Создание шаблона' }))
    expect(createTemplateMutate).not.toHaveBeenCalled()

    useCreateTemplate.mockReturnValue({
      mutate: createTemplateMutate,
      isPending: false,
      isError: true,
      error: { code: 'CONFLICT', message: 'Conflict' },
    })
    fireEvent.change(screen.getByRole('searchbox', { name: 'Найти шаблон' }), {
      target: { value: 'after error' },
    })
    expect(screen.getByLabelText('Название шаблона')).toHaveValue('Мой шаблон')
    expect(screen.getByLabelText('Markdown шаблона')).toHaveValue('# Текст')
    expect(screen.getByRole('alert')).toBeInTheDocument()
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
