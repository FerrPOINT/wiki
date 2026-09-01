import { fireEvent, render, screen } from '@testing-library/react'
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
const createMutate = vi.hoisted(() => vi.fn())
const deleteMemberMutate = vi.hoisted(() => vi.fn())
const updateMutate = vi.hoisted(() => vi.fn())
const upsertMemberMutate = vi.hoisted(() => vi.fn())
const refetchMembers = vi.hoisted(() => vi.fn())
const refetchSpaces = vi.hoisted(() => vi.fn())
const refetchTree = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({
  defaultSpaceKey: 'SDLC',
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

function setupSpaces({
  membersOverrides = {},
  userOverrides = {},
  usersOverrides = {},
}: {
  membersOverrides?: Record<string, unknown>
  userOverrides?: Record<string, unknown>
  usersOverrides?: Record<string, unknown>
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
      spaces: [
        {
          id: 'space-sdlc',
          key: 'SDLC',
          name: 'База знаний SDLC',
          description: 'Основные документы продукта',
          owner_id: 'user-admin',
          status: 'active',
          document_count: 2,
          member_count: 2,
          created_at: '2026-08-31T10:00:00Z',
          updated_at: '2026-08-31T11:00:00Z',
        },
      ],
    },
    isLoading: false,
    isError: false,
    refetch: refetchSpaces,
  })
  useSpaceTree.mockReturnValue({
    data: {
      space_key: 'SDLC',
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

  render(
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
    expect(screen.getByText(/SDLC · База знаний SDLC/)).toBeInTheDocument()
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
    expect(useSpaceTree).toHaveBeenCalledWith('SDLC')
    expect(useSpaceMembers).toHaveBeenCalledWith('SDLC')
  })

  it('submits space create, update, archive and member mutations', () => {
    setupSpaces()

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
      spaceKey: 'SDLC',
      body: {
        name: 'База знаний продукта',
        description: 'Основные документы продукта',
      },
    })

    fireEvent.click(screen.getByRole('button', { name: 'Архивировать' }))
    expect(archiveMutate).toHaveBeenCalledWith('SDLC')

    fireEvent.change(screen.getByLabelText('Пользователь'), {
      target: { value: 'user-editor' },
    })
    fireEvent.change(screen.getByLabelText('Роль'), {
      target: { value: 'viewer' },
    })
    fireEvent.submit(screen.getByLabelText('Пользователь').closest('form')!)
    expect(upsertMemberMutate).toHaveBeenCalledWith(
      {
        spaceKey: 'SDLC',
        userId: 'user-editor',
        body: { role: 'viewer' },
      },
      { onSuccess: expect.any(Function) },
    )

    const removeButtons = screen.getAllByRole('button', { name: 'Удалить' })
    expect(removeButtons).toHaveLength(2)
    fireEvent.click(removeButtons[1]!)
    expect(deleteMemberMutate).toHaveBeenCalledWith({
      spaceKey: 'SDLC',
      userId: 'user-editor',
    })
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

    fireEvent.change(screen.getByLabelText('Пользователь'), {
      target: { value: 'user-editor' },
    })
    fireEvent.submit(screen.getByLabelText('Пользователь').closest('form')!)
    expect(upsertMemberMutate).toHaveBeenCalledWith(
      {
        spaceKey: 'SDLC',
        userId: 'user-editor',
        body: { role: 'viewer' },
      },
      { onSuccess: expect.any(Function) },
    )
  })

  it('shows member permission errors with a retry action', () => {
    setupSpaces({
      membersOverrides: {
        data: undefined,
        isError: true,
        error: { code: 'FORBIDDEN', message: 'Forbidden' },
      },
    })

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
      '/documents/new?space=SDLC',
    )
    expect(useSpaceTree).not.toHaveBeenCalled()
  })
})
