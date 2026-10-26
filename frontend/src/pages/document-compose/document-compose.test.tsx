import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { ReactNode } from 'react'
import {
  act,
  fireEvent,
  render as renderView,
  screen,
  waitFor,
  within,
} from '@testing-library/react'
import { createMemoryRouter, RouterProvider } from 'react-router'
import { afterEach, describe, expect, it, vi } from 'vitest'

import { DocumentComposePage } from './'

function render(view: ReactNode) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return renderView(<QueryClientProvider client={client}>{view}</QueryClientProvider>)
}

const useCreateDocument = vi.hoisted(() => vi.fn())
const useSpaces = vi.hoisted(() => vi.fn())
const useTemplates = vi.hoisted(() => vi.fn())

const createDocumentMutate = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({
  defaultSpaceKey: 'BASE',
  useCreateDocument,
  useSpaces,
  useTemplates,
}))

function setupCompose(
  createOverrides: Record<string, unknown> = {},
  initialRoute = '/documents/new?space=eng',
  spaceOverrides: Record<string, unknown> = {},
  templateOverrides: Record<string, unknown> = {},
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
    ...spaceOverrides,
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
    ...templateOverrides,
  })
  useCreateDocument.mockReturnValue({
    mutate: createDocumentMutate,
    isPending: false,
    isError: false,
    error: null,
    ...createOverrides,
  })

  const router = createMemoryRouter(
    [
      { path: '/documents/new', element: <DocumentComposePage /> },
      { path: '*', element: <div>Destination</div> },
    ],
    { initialEntries: [initialRoute] },
  )
  render(<RouterProvider router={router} />)
  return router
}

describe('DocumentComposePage', () => {
  afterEach(() => {
    vi.clearAllMocks()
  })

  it('locks the submitted draft while document creation is pending', () => {
    setupCompose({ isPending: true })

    for (const label of [
      'Название',
      'Markdown документа',
      'Тип',
      'Адрес (slug)',
      'Задача',
      'Фаза',
    ]) {
      expect(screen.getByLabelText(label, { exact: true })).toBeDisabled()
    }
    expect(screen.getByRole('button', { name: 'План проверки' })).toBeDisabled()
  })

  it('does not submit another create request while the first is pending', () => {
    setupCompose({ isPending: true })

    const form = screen.getByLabelText('Название').closest('form')!
    fireEvent.submit(form)

    expect(createDocumentMutate).not.toHaveBeenCalled()
  })

  it('creates a document through the shared API hook and navigates to the result', async () => {
    const router = setupCompose()

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

    act(() =>
      createDocumentMutate.mock.calls[0]?.[1]?.onSuccess({
        id: '01a10563-d3e4-7e60-bc93-d273364c683f',
        slug: 'new-policy',
      }),
    )
    await waitFor(() =>
      expect(router.state.location.pathname).toBe(
        '/documents/01a10563-d3e4-7e60-bc93-d273364c683f',
      ),
    )
  })

  it('opens the preview from the page action', () => {
    setupCompose()
    fireEvent.change(screen.getByLabelText('Название'), { target: { value: 'Новый регламент' } })
    fireEvent.change(screen.getByLabelText('Markdown документа'), {
      target: { value: '# Содержание' },
    })

    fireEvent.click(screen.getByRole('button', { name: 'Предпросмотр' }))

    expect(screen.getByRole('tab', { name: 'Просмотр' })).toHaveAttribute('data-state', 'active')
    expect(screen.getByRole('heading', { name: 'Содержание', level: 1 })).toBeVisible()
    expect(createDocumentMutate).not.toHaveBeenCalled()
  })

  it('renders Markdown safely and preserves the source when returning to the editor', () => {
    setupCompose()
    const markdown =
      '# Проверка\n\n**Результат** и *курсив*\n\n- Первый\n- Второй\n\n[Документы](https://example.test/docs)\n\n```text\ncode <tag>\n```\n\n<script>alert(1)</script>\n\n[Опасно](javascript:alert%281%29)'
    fireEvent.change(screen.getByLabelText('Markdown документа'), { target: { value: markdown } })
    fireEvent.click(screen.getByRole('button', { name: 'Предпросмотр' }))
    const preview = screen.getByRole('tabpanel', { name: 'Просмотр' })
    expect(within(preview).getByText('Результат').tagName).toBe('STRONG')
    expect(within(preview).getByText('курсив').tagName).toBe('EM')
    expect(within(preview).getAllByRole('listitem')).toHaveLength(2)
    expect(within(preview).getByRole('link', { name: 'Документы' })).toHaveAttribute(
      'href',
      'https://example.test/docs',
    )
    expect(preview.querySelector('pre code')).toHaveTextContent('code <tag>')
    expect(preview.querySelector('pre')).toHaveAttribute('tabindex', '0')
    expect(preview.querySelector('script')).toBeNull()
    expect(preview.querySelector('[href^="javascript:"]')).toBeNull()
    fireEvent.mouseDown(screen.getByRole('tab', { name: 'Markdown' }), {
      button: 0,
      ctrlKey: false,
    })
    expect(screen.getByLabelText('Markdown документа')).toHaveValue(markdown)
    expect(createDocumentMutate).not.toHaveBeenCalled()
  })

  it('retains the code node and focus while other form data changes', () => {
    setupCompose()
    fireEvent.change(screen.getByLabelText('Markdown документа'), {
      target: { value: '```\nRetain preview focus\n```' },
    })
    fireEvent.click(screen.getByRole('button', { name: 'Предпросмотр' }))
    const code = screen.getByRole('region', { name: 'Код' })
    code.focus()
    fireEvent.change(screen.getByLabelText('Название'), { target: { value: 'Updated title' } })
    expect(screen.getByRole('region', { name: 'Код' })).toBe(code)
    expect(code).toHaveFocus()
    expect(createDocumentMutate).not.toHaveBeenCalled()
  })

  it('keeps an unsaved draft when navigation is cancelled and leaves after confirmation', async () => {
    const router = setupCompose()
    const cleanUnloadEvent = new Event('beforeunload', { cancelable: true })
    window.dispatchEvent(cleanUnloadEvent)
    expect(cleanUnloadEvent.defaultPrevented).toBe(false)

    fireEvent.change(screen.getByLabelText('Название'), { target: { value: 'Черновик' } })
    const dirtyUnloadEvent = new Event('beforeunload', { cancelable: true })
    window.dispatchEvent(dirtyUnloadEvent)
    expect(dirtyUnloadEvent.defaultPrevented).toBe(true)

    await act(async () => router.navigate('/spaces'))
    const dialog = screen.getByRole('alertdialog')
    expect(dialog).toHaveTextContent('Несохранённые изменения будут потеряны')
    expect(router.state.location.pathname).toBe('/documents/new')

    fireEvent.click(within(dialog).getByRole('button', { name: 'Остаться' }))
    expect(screen.getByLabelText('Название')).toHaveValue('Черновик')
    expect(router.state.location.pathname).toBe('/documents/new')

    await act(async () => router.navigate('/spaces'))
    fireEvent.click(
      within(screen.getByRole('alertdialog')).getByRole('button', { name: 'Покинуть' }),
    )
    await waitFor(() => expect(router.state.location.pathname).toBe('/spaces'))
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

  it('keeps an explicitly selected template instead of reapplying the URL template', () => {
    setupCompose(
      {},
      '/documents/new?space=eng&template=tpl-test-plan',
      {},
      {
        data: {
          templates: [
            {
              id: 'tpl-test-plan',
              name: 'План проверки',
              document_type: 'test_plan',
              body_markdown: '# Проверка',
            },
            {
              id: 'tpl-release',
              name: 'Выпуск',
              document_type: 'release_note',
              body_markdown: '# Выпуск',
            },
          ],
        },
      },
    )
    expect(screen.getByLabelText('Markdown документа')).toHaveValue('# Проверка')
    fireEvent.click(screen.getByRole('button', { name: 'Выпуск' }))
    expect(screen.getByLabelText('Markdown документа')).toHaveValue('# Выпуск')
    expect(screen.getByLabelText('Тип')).toHaveValue('release_note')
    fireEvent.change(screen.getByLabelText('Название'), { target: { value: 'Релиз' } })
    fireEvent.click(screen.getByRole('button', { name: 'Сохранить черновик' }))
    expect(createDocumentMutate).toHaveBeenCalledWith(
      expect.objectContaining({
        body: expect.objectContaining({
          content_markdown: '# Выпуск',
          document_type: 'release_note',
        }),
      }),
      expect.any(Object),
    )
  })

  it.each(['# Мой текст', ''])(
    'does not replace an edited body when the requested template arrives late: %j',
    (body) => {
      setupCompose(
        {},
        '/documents/new?space=eng&template=tpl-test-plan',
        {},
        {
          data: undefined,
          isLoading: true,
        },
      )
      fireEvent.change(screen.getByLabelText('Markdown документа'), {
        target: { value: 'Начатый текст' },
      })
      fireEvent.change(screen.getByLabelText('Markdown документа'), { target: { value: body } })
      useTemplates.mockReturnValue({
        data: {
          templates: [
            {
              id: 'tpl-test-plan',
              name: 'План проверки',
              document_type: 'test_plan',
              body_markdown: '# Проверка',
            },
          ],
        },
        isLoading: false,
        isError: false,
      })
      fireEvent.change(screen.getByLabelText('Название'), { target: { value: 'Мой документ' } })
      expect(screen.getByLabelText('Markdown документа')).toHaveValue(body)
      expect(screen.getByLabelText('Тип')).toHaveValue('page')
      expect(createDocumentMutate).not.toHaveBeenCalled()
      fireEvent.click(screen.getByRole('button', { name: 'План проверки' }))
      expect(screen.getByLabelText('Markdown документа')).toHaveValue('# Проверка')
      expect(screen.getByLabelText('Тип')).toHaveValue('test_plan')
    },
  )

  it('does not replace an explicitly edited document type when the requested template arrives late', () => {
    setupCompose(
      {},
      '/documents/new?space=eng&template=tpl-test-plan',
      {},
      {
        data: undefined,
        isLoading: true,
      },
    )
    fireEvent.change(screen.getByLabelText('Тип'), { target: { value: 'research_note' } })
    useTemplates.mockReturnValue({
      data: {
        templates: [
          {
            id: 'tpl-test-plan',
            name: 'План проверки',
            document_type: 'test_plan',
            body_markdown: '# Проверка',
          },
        ],
      },
      isLoading: false,
      isError: false,
    })
    fireEvent.change(screen.getByLabelText('Название'), { target: { value: 'Исследование' } })
    expect(screen.getByLabelText('Тип')).toHaveValue('research_note')
    expect(screen.getByLabelText('Markdown документа')).toHaveValue('')
    expect(createDocumentMutate).not.toHaveBeenCalled()
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
  it('blocks creation when the confirmed space catalogue is empty and keeps the entered draft', () => {
    setupCompose({}, '/documents/new', { data: { spaces: [] } })
    fireEvent.change(screen.getByLabelText('Название'), { target: { value: 'Keep my draft' } })
    expect(screen.getByRole('button', { name: 'Сохранить черновик' })).toBeDisabled()
    expect(screen.getByText(/Нет доступных пространств/)).toBeInTheDocument()
    expect(screen.getByLabelText('Название')).toHaveValue('Keep my draft')
    expect(createDocumentMutate).not.toHaveBeenCalled()
  })

  it('selects an existing non-BASE space for a blank URL without creating a dirty draft', async () => {
    setupCompose({}, '/documents/new')
    await screen.findByRole('button', { name: 'Пространство: Engineering (ENG)' })
    const event = new Event('beforeunload', { cancelable: true })
    window.dispatchEvent(event)
    expect(event.defaultPrevented).toBe(false)
    expect(screen.getByRole('button', { name: 'Сохранить черновик' })).toBeEnabled()
  })

  it('does not replace an unavailable explicit space with the first accessible one', () => {
    setupCompose({}, '/documents/new?space=MISSING')
    expect(screen.getByText(/Выбранное пространство недоступно/)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Сохранить черновик' })).toBeDisabled()
    expect(createDocumentMutate).not.toHaveBeenCalled()
  })

  it('does not move an automatically selected draft when BASE appears on catalogue refresh', async () => {
    setupCompose({}, '/documents/new')
    await screen.findByRole('button', { name: 'Пространство: Engineering (ENG)' })
    fireEvent.change(screen.getByLabelText('Название'), { target: { value: 'Engineering draft' } })
    const previous = useSpaces.mock.results.at(-1)!.value
    useSpaces.mockReturnValue({
      ...previous,
      data: {
        spaces: [
          { ...previous.data.spaces[0], id: 'space-base', key: 'BASE', name: 'Base' },
          ...previous.data.spaces,
        ],
      },
    })
    fireEvent.change(screen.getByLabelText('Markdown документа'), {
      target: { value: 'Private engineering content' },
    })
    expect(screen.getByRole('button', { name: 'Пространство: Engineering (ENG)' })).toBeVisible()
    fireEvent.click(screen.getByRole('button', { name: 'Сохранить черновик' }))
    expect(createDocumentMutate).toHaveBeenCalledWith(
      expect.objectContaining({ spaceKey: 'ENG' }),
      expect.any(Object),
    )
  })

  it('blocks saving when the automatically selected space disappears instead of choosing another', async () => {
    setupCompose({}, '/documents/new')
    await screen.findByRole('button', { name: 'Пространство: Engineering (ENG)' })
    const previous = useSpaces.mock.results.at(-1)!.value
    useSpaces.mockReturnValue({
      ...previous,
      data: {
        spaces: [{ ...previous.data.spaces[0], id: 'space-base', key: 'BASE', name: 'Base' }],
      },
    })
    fireEvent.change(screen.getByLabelText('Название'), {
      target: { value: 'Keep my engineering draft' },
    })
    expect(screen.getByRole('button', { name: 'Сохранить черновик' })).toBeDisabled()
    expect(screen.getByText(/Выбранное пространство недоступно/)).toBeVisible()
    expect(screen.getByLabelText('Название')).toHaveValue('Keep my engineering draft')
    expect(createDocumentMutate).not.toHaveBeenCalled()
  })

  it('retains input when the catalogue is unavailable rather than inventing BASE', () => {
    setupCompose({}, '/documents/new', { data: undefined, isError: true, refetch: vi.fn() })
    fireEvent.change(screen.getByLabelText('Markdown документа'), {
      target: { value: 'Unsaved body' },
    })
    expect(screen.getByRole('button', { name: 'Сохранить черновик' })).toBeDisabled()
    expect(screen.getByLabelText('Markdown документа')).toHaveValue('Unsaved body')
    expect(createDocumentMutate).not.toHaveBeenCalled()
  })
})
