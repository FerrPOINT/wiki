import { act, fireEvent, render, screen, within } from '@testing-library/react'
import { MemoryRouter } from 'react-router'
import { afterEach, describe, expect, it, vi } from 'vitest'

import { SpacesPage } from './'

const useArchiveSpace = vi.hoisted(() => vi.fn())
const useCreateSpace = vi.hoisted(() => vi.fn())
const useCurrentUser = vi.hoisted(() => vi.fn())
const useDeleteSpaceMember = vi.hoisted(() => vi.fn())
const useSpaceMembers = vi.hoisted(() => vi.fn())
const useSpaces = vi.hoisted(() => vi.fn())
const useSpaceTree = vi.hoisted(() => vi.fn())
const useUpdateSpace = vi.hoisted(() => vi.fn())
const useUpsertSpaceMember = vi.hoisted(() => vi.fn())
const useUsers = vi.hoisted(() => vi.fn())

const archiveMutate = vi.hoisted(() => vi.fn())
const archiveReset = vi.hoisted(() => vi.fn())
const createMutate = vi.hoisted(() => vi.fn())
const deleteMemberMutate = vi.hoisted(() => vi.fn())
const updateMutate = vi.hoisted(() => vi.fn())
const upsertMemberMutate = vi.hoisted(() => vi.fn())
const refetchMembers = vi.hoisted(() => vi.fn())
const refetchSpaces = vi.hoisted(() => vi.fn())
const refetchTree = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({
  defaultSpaceKey: 'BASE',
  useArchiveSpace,
  useCreateSpace,
  useCurrentUser,
  useDeleteSpaceMember,
  useSpaceMembers,
  useSpaces,
  useSpaceTree,
  useUpdateSpace,
  useUpsertSpaceMember,
  useUsers,
}))

const adminUser = {
  active: true,
  display_name: 'Администратор',
  email: 'admin@example.test',
  id: 'user-admin',
  is_system_admin: true,
  role: 'admin',
  username: 'admin',
}

const editorUser = {
  active: true,
  display_name: 'Редактор',
  email: 'editor@example.test',
  id: 'user-editor',
  is_system_admin: false,
  role: 'user',
  username: 'editor',
}

const baseSpace = {
  id: 'space-sdlc',
  key: 'BASE',
  name: 'База знаний Base',
  description: 'Основные документы продукта',
  owner_id: 'user-admin',
  status: 'active',
  document_count: 2,
  member_count: 2,
  created_at: '2026-08-31T10:00:00Z',
  updated_at: '2026-08-31T11:00:00Z',
}

function setupSpaces({
  membersOverrides = {},
  userOverrides = {},
  usersOverrides = {},
  spaceList,
}: {
  membersOverrides?: Record<string, unknown>
  userOverrides?: Record<string, unknown>
  usersOverrides?: Record<string, unknown>
  spaceList?: Array<Record<string, unknown>>
} = {}) {
  useCurrentUser.mockReturnValue({
    data: adminUser,
    isLoading: false,
    isError: false,
    ...userOverrides,
  })
  useUsers.mockReturnValue({
    data: { users: [adminUser, editorUser] },
    isLoading: false,
    isError: false,
    ...usersOverrides,
  })
  useSpaces.mockReturnValue({
    data: {
      spaces: spaceList ?? [baseSpace],
    },
    isLoading: false,
    isError: false,
    refetch: refetchSpaces,
  })
  useSpaceTree.mockReturnValue({
    data: {
      space_key: 'BASE',
      documents: [
        {
          id: 'doc-root',
          slug: 'product-requirements',
          title: 'Требования',
          document_type: 'requirements',
          status: 'published',
          children: [
            {
              id: 'doc-child',
              slug: 'test-plan',
              title: 'План проверки',
              document_type: 'test_plan',
              status: 'draft',
              children: [],
            },
          ],
        },
      ],
    },
    isLoading: false,
    isError: false,
    refetch: refetchTree,
  })
  useSpaceMembers.mockReturnValue({
    data: {
      members: [
        {
          display_name: 'Администратор',
          email: 'admin@example.test',
          joined_at: '2026-08-31T10:00:00Z',
          role: 'admin',
          user_id: 'user-admin',
        },
        {
          display_name: 'Редактор',
          email: 'editor@example.test',
          joined_at: '2026-08-31T10:00:00Z',
          role: 'editor',
          user_id: 'user-editor',
        },
      ],
    },
    isLoading: false,
    isError: false,
    refetch: refetchMembers,
    ...membersOverrides,
  })
  useCreateSpace.mockReturnValue({
    mutate: createMutate,
    isPending: false,
    isError: false,
    error: null,
  })
  useUpdateSpace.mockReturnValue({
    mutate: updateMutate,
    isPending: false,
    isError: false,
    error: null,
  })
  useArchiveSpace.mockReturnValue({
    mutate: archiveMutate,
    reset: archiveReset,
    isPending: false,
    isError: false,
    error: null,
  })
  useUpsertSpaceMember.mockReturnValue({
    mutate: upsertMemberMutate,
    isPending: false,
    isError: false,
    error: null,
  })
  useDeleteSpaceMember.mockReturnValue({
    mutate: deleteMemberMutate,
    isPending: false,
    isError: false,
    error: null,
  })

  return render(
    <MemoryRouter>
      <SpacesPage />
    </MemoryRouter>,
  )
}

describe('SpacesPage', () => {
  afterEach(() => {
    vi.clearAllMocks()
  })

  it('renders API-backed spaces with document tree and members', () => {
    setupSpaces()

    expect(screen.getByRole('heading', { name: 'Пространства' })).toBeInTheDocument()
    expect(screen.getByText('База знаний Base')).toBeInTheDocument()
    expect(useSpaceTree).not.toHaveBeenCalled()
    expect(useSpaceMembers).not.toHaveBeenCalled()
    fireEvent.click(screen.getByRole('button', { name: /База знаний Base/ }))
    expect(screen.getAllByText('Основные документы продукта')).toHaveLength(2)
    expect(screen.getByText('Дерево')).toBeInTheDocument()
    expect(screen.getByText('Участники')).toBeInTheDocument()
    expect(screen.getByText('Редактор')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: /Требования требования/ })).toHaveAttribute(
      'href',
      '/documents/product-requirements',
    )
    expect(screen.getByRole('link', { name: /План проверки план проверки/ })).toHaveAttribute(
      'href',
      '/documents/test-plan',
    )
    expect(useSpaceTree).toHaveBeenCalledWith('BASE')
    expect(useSpaceMembers).toHaveBeenCalledWith('BASE')
  })

  it('submits space create, update and member mutations', () => {
    setupSpaces()
    fireEvent.click(screen.getByRole('button', { name: 'Создать пространство' }))
    fireEvent.click(screen.getByRole('button', { name: /База знаний Base/ }))

    fireEvent.change(screen.getByLabelText('Ключ'), {
      target: { value: 'TEAM' },
    })
    fireEvent.change(screen.getByLabelText('Название'), {
      target: { value: 'Командная Wiki' },
    })
    fireEvent.change(screen.getAllByLabelText('Описание')[0]!, {
      target: { value: 'Документы команды и решений по задачам' },
    })
    fireEvent.submit(screen.getByLabelText('Ключ').closest('form')!)
    expect(createMutate).toHaveBeenCalledWith(
      {
        key: 'TEAM',
        name: 'Командная Wiki',
        description: 'Документы команды и решений по задачам',
      },
      { onSuccess: expect.any(Function) },
    )

    fireEvent.change(screen.getByLabelText('Название пространства'), {
      target: { value: 'База знаний продукта' },
    })
    fireEvent.submit(screen.getByLabelText('Название пространства').closest('form')!)
    expect(updateMutate).toHaveBeenCalledWith({
      spaceKey: 'BASE',
      body: {
        name: 'База знаний продукта',
        description: 'Основные документы продукта',
      },
    })

    fireEvent.change(screen.getByLabelText('Пользователь'), {
      target: { value: 'user-editor' },
    })
    fireEvent.change(screen.getByLabelText('Роль'), {
      target: { value: 'viewer' },
    })
    fireEvent.submit(screen.getByLabelText('Пользователь').closest('form')!)
    expect(upsertMemberMutate).toHaveBeenCalledWith(
      {
        spaceKey: 'BASE',
        userId: 'user-editor',
        body: { role: 'viewer' },
      },
      { onSuccess: expect.any(Function) },
    )

    const removeButtons = screen.getAllByRole('button', { name: 'Удалить' })
    expect(removeButtons).toHaveLength(2)
    fireEvent.click(removeButtons[1]!)
    expect(deleteMemberMutate).toHaveBeenCalledWith({
      spaceKey: 'BASE',
      userId: 'user-editor',
    })
  })

  it('confirms archive, allows cancellation, and keeps the archived space visible', () => {
    const otherSpaces = Array.from({ length: 24 }, (_, index) => ({
      ...baseSpace,
      id: `space-${index + 1}`,
      key: `S${index + 1}`,
      name: `Space ${index + 1}`,
      updated_at: '2026-09-01T11:00:00Z',
    }))
    const view = setupSpaces({ spaceList: [...otherSpaces, baseSpace] })
    fireEvent.change(screen.getByRole('combobox', { name: 'Статус пространства' }), {
      target: { value: 'active' },
    })
    fireEvent.click(screen.getByRole('button', { name: 'Далее' }))
    fireEvent.click(screen.getByRole('button', { name: 'Далее' }))
    fireEvent.click(screen.getByRole('button', { name: /База знаний Base/ }))
    fireEvent.click(screen.getByRole('button', { name: 'Архивировать' }))

    let dialog = screen.getByRole('alertdialog')
    expect(dialog).toHaveTextContent('База знаний Base')
    expect(dialog).toHaveTextContent('Восстановление из интерфейса пока недоступно')
    expect(archiveMutate).not.toHaveBeenCalled()
    fireEvent.click(within(dialog).getByRole('button', { name: 'Отмена' }))
    expect(archiveMutate).not.toHaveBeenCalled()

    fireEvent.click(screen.getByRole('button', { name: 'Архивировать' }))
    dialog = screen.getByRole('alertdialog')
    expect(archiveReset).toHaveBeenCalledTimes(2)
    fireEvent.click(within(dialog).getByRole('button', { name: 'Подтвердить' }))
    expect(archiveMutate).toHaveBeenCalledWith('BASE', { onSuccess: expect.any(Function) })
    expect(dialog).toBeVisible()

    const onSuccess = archiveMutate.mock.calls[0]![1].onSuccess as (space: typeof baseSpace) => void
    act(() =>
      onSuccess({ ...baseSpace, status: 'archived', updated_at: '2026-09-20T11:00:00Z' }),
    )
    expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument()
    expect(screen.getByRole('status')).toHaveTextContent(
      'Пространство «База знаний Base» архивировано',
    )
    expect(screen.getByRole('combobox', { name: 'Статус пространства' })).toHaveValue('all')
    expect(screen.getByRole('button', { name: /База знаний Base/ })).toHaveTextContent(
      'архивировано',
    )
    expect(screen.getByRole('navigation', { name: 'Страницы пространств' })).toHaveTextContent(
      '1 / 3',
    )
    expect(screen.queryByRole('button', { name: 'Архивировать' })).not.toBeInTheDocument()

    useSpaces.mockReturnValue({
      data: {
        spaces: [
          ...otherSpaces,
          { ...baseSpace, status: 'archived', updated_at: '2026-09-20T11:00:00Z' },
        ],
      },
      isLoading: false,
      isError: false,
      refetch: refetchSpaces,
    })
    view.rerender(
      <MemoryRouter>
        <SpacesPage />
      </MemoryRouter>,
    )
    expect(screen.getByRole('button', { name: /База знаний Base/ })).toHaveTextContent(
      'архивировано',
    )
  })

  it('keeps archive confirmation open during pending and error states', () => {
    const view = setupSpaces()
    fireEvent.click(screen.getByRole('button', { name: /База знаний Base/ }))
    fireEvent.click(screen.getByRole('button', { name: 'Архивировать' }))
    fireEvent.click(
      within(screen.getByRole('alertdialog')).getByRole('button', { name: 'Подтвердить' }),
    )

    useArchiveSpace.mockReturnValue({
      mutate: archiveMutate,
      reset: archiveReset,
      isPending: true,
      isError: false,
      error: null,
    })
    view.rerender(
      <MemoryRouter>
        <SpacesPage />
      </MemoryRouter>,
    )
    expect(
      within(screen.getByRole('alertdialog')).getByRole('button', { name: 'Отмена' }),
    ).toBeDisabled()
    expect(
      within(screen.getByRole('alertdialog')).getByRole('button', { name: 'Загрузка...' }),
    ).toBeDisabled()

    useArchiveSpace.mockReturnValue({
      mutate: archiveMutate,
      reset: archiveReset,
      isPending: false,
      isError: true,
      error: { code: 'FORBIDDEN' },
    })
    view.rerender(
      <MemoryRouter>
        <SpacesPage />
      </MemoryRouter>,
    )
    expect(screen.getByRole('alertdialog')).toHaveTextContent('Недостаточно прав для действия')
    expect(archiveMutate).toHaveBeenCalledOnce()
    fireEvent.click(
      within(screen.getByRole('alertdialog')).getByRole('button', { name: 'Подтвердить' }),
    )
    expect(archiveMutate).toHaveBeenCalledTimes(2)
  })

  it('keeps member assignment available for space admins without the global user list', () => {
    setupSpaces({
      userOverrides: {
        data: { ...adminUser, is_system_admin: false },
      },
      usersOverrides: {
        data: undefined,
      },
    })

    expect(screen.queryByRole('button', { name: 'Создать пространство' })).not.toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: /База знаний Base/ }))

    fireEvent.change(screen.getByLabelText('Пользователь'), {
      target: { value: 'user-editor' },
    })
    fireEvent.submit(screen.getByLabelText('Пользователь').closest('form')!)
    expect(upsertMemberMutate).toHaveBeenCalledWith(
      {
        spaceKey: 'BASE',
        userId: 'user-editor',
        body: { role: 'viewer' },
      },
      { onSuccess: expect.any(Function) },
    )
  })

  it('does not offer archive or member changes to a viewer', () => {
    setupSpaces({ userOverrides: { data: editorUser } })
    fireEvent.click(screen.getByRole('button', { name: /База знаний Base/ }))

    expect(screen.queryByRole('button', { name: 'Архивировать' })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Назначить' })).not.toBeInTheDocument()
    expect(archiveMutate).not.toHaveBeenCalled()
  })

  it('shows member permission errors with a retry action', () => {
    setupSpaces({
      membersOverrides: {
        data: undefined,
        isError: true,
        error: { code: 'FORBIDDEN', message: 'Forbidden' },
      },
    })

    fireEvent.click(screen.getByRole('button', { name: /База знаний Base/ }))
    expect(screen.getByRole('alert')).toHaveTextContent('Недостаточно прав для действия')
    fireEvent.click(screen.getByRole('button', { name: 'Повторить' }))
    expect(refetchMembers).toHaveBeenCalled()
  })

  it('shows the empty state action for a fresh Wiki instance', () => {
    useCurrentUser.mockReturnValue({
      data: adminUser,
      isLoading: false,
      isError: false,
    })
    useUsers.mockReturnValue({
      data: { users: [adminUser] },
      isLoading: false,
      isError: false,
    })
    useSpaces.mockReturnValue({
      data: { spaces: [] },
      isLoading: false,
      isError: false,
      refetch: refetchSpaces,
    })
    useCreateSpace.mockReturnValue({
      mutate: createMutate,
      isPending: false,
      isError: false,
      error: null,
    })

    render(
      <MemoryRouter>
        <SpacesPage />
      </MemoryRouter>,
    )

    expect(screen.getByText('Пространства ещё не созданы')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'Создать первый документ' })).toHaveAttribute(
      'href',
      '/documents/new?space=BASE',
    )
    expect(useSpaceTree).not.toHaveBeenCalled()
  })

  it('paginates and searches spaces without eager detail requests', () => {
    const spaceList = Array.from({ length: 25 }, (_, index) => ({
      id: `space-${index + 1}`,
      key: `S${index + 1}`,
      name: `Space ${index + 1}`,
      description: `Description ${index + 1}`,
      owner_id: 'user-admin',
      status: 'active',
      document_count: 1,
      member_count: 1,
      created_at: '2026-08-31T10:00:00Z',
      updated_at: '2026-08-31T11:00:00Z',
    }))
    setupSpaces({ spaceList })

    expect(screen.getByText('Space 1')).toBeInTheDocument()
    expect(screen.queryByText('Space 13')).not.toBeInTheDocument()
    expect(useSpaceTree).not.toHaveBeenCalled()
    expect(useSpaceMembers).not.toHaveBeenCalled()

    fireEvent.click(screen.getByRole('button', { name: 'Далее' }))
    expect(screen.getByText('Space 13')).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: /Space 13/ }))
    expect(useSpaceTree).toHaveBeenCalledTimes(1)
    expect(useSpaceMembers).toHaveBeenCalledTimes(1)

    fireEvent.change(screen.getByRole('searchbox', { name: 'Найти пространство' }), {
      target: { value: 'Space 25' },
    })
    expect(screen.getByText('Space 25')).toBeInTheDocument()
    expect(screen.queryByText('Space 13')).not.toBeInTheDocument()
  })
})
