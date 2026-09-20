import { fireEvent, render, screen, within } from '@testing-library/react'
import { MemoryRouter } from 'react-router'
import { afterEach, describe, expect, it, vi } from 'vitest'

import { SpacesPage } from './'

const useArchiveSpace = vi.hoisted(() => vi.fn())
const useCreateSpace = vi.hoisted(() => vi.fn())
const useCurrentUser = vi.hoisted(() => vi.fn())
const useSpaceMembers = vi.hoisted(() => vi.fn())
const useSpaces = vi.hoisted(() => vi.fn())
const useSpaceTree = vi.hoisted(() => vi.fn())
const useUpdateSpace = vi.hoisted(() => vi.fn())
const useUsers = vi.hoisted(() => vi.fn())

const archiveMutate = vi.hoisted(() => vi.fn())
const archiveReset = vi.hoisted(() => vi.fn())
const createMutate = vi.hoisted(() => vi.fn())
const updateMutate = vi.hoisted(() => vi.fn())
const refetchMembers = vi.hoisted(() => vi.fn())
const refetchSpaces = vi.hoisted(() => vi.fn())
const refetchTree = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({
  defaultSpaceKey: 'BASE',
  useArchiveSpace,
  useCreateSpace,
  useCurrentUser,
  useSpaceMembers,
  useSpaces,
  useSpaceTree,
  useUpdateSpace,
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
      spaces: spaceList ?? [
        {
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
        },
      ],
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
    data: { members: [{ user_id: 'user-admin', role: 'admin' }] },
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

  it('renders spaces and the document tree without legacy membership controls', () => {
    setupSpaces()

    expect(screen.getByRole('heading', { name: 'Пространства' })).toBeInTheDocument()
    expect(screen.getByText('База знаний Base')).toBeInTheDocument()
    expect(useSpaceTree).not.toHaveBeenCalled()
    expect(useSpaceMembers).not.toHaveBeenCalled()
    fireEvent.click(screen.getByRole('button', { name: /База знаний Base/ }))
    expect(screen.getAllByText('Основные документы продукта')).toHaveLength(2)
    expect(screen.getByText('Дерево')).toBeInTheDocument()
    expect(screen.queryByText('Участники')).not.toBeInTheDocument()
    expect(screen.queryByText('2 участников')).not.toBeInTheDocument()
    expect(screen.queryByLabelText('Роль')).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Назначить' })).not.toBeInTheDocument()
    expect(screen.getByRole('link', { name: /Требования требования/ })).toHaveAttribute(
      'href',
      '/documents/product-requirements',
    )
    expect(screen.getByRole('link', { name: /План проверки план проверки/ })).toHaveAttribute(
      'href',
      '/documents/test-plan',
    )
    expect(useSpaceTree).toHaveBeenCalledWith('BASE')
    expect(useSpaceMembers).toHaveBeenCalledWith('BASE', false)
  })

  it('submits space create, update and archive mutations', () => {
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

    fireEvent.click(screen.getByRole('button', { name: 'Архивировать' }))
    const dialog = screen.getByRole('alertdialog')
    expect(archiveMutate).not.toHaveBeenCalled()
    fireEvent.click(within(dialog).getByRole('button', { name: 'Подтвердить' }))
    expect(archiveMutate).toHaveBeenCalledWith('BASE', { onSuccess: expect.any(Function) })
  })

  it('preserves legacy space-admin editing without exposing role assignment', () => {
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
    expect(useSpaceMembers).toHaveBeenCalledWith('BASE', true)
    expect(screen.getByLabelText('Название пространства')).toBeInTheDocument()
    expect(screen.queryByLabelText('Роль')).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Назначить' })).not.toBeInTheDocument()
  })

  it('keeps legacy viewers from editing', () => {
    setupSpaces({
      userOverrides: { data: { ...adminUser, is_system_admin: false } },
      membersOverrides: {
        data: { members: [{ user_id: 'user-admin', role: 'viewer' }] },
      },
    })
    fireEvent.click(screen.getByRole('button', { name: /База знаний Base/ }))
    expect(screen.queryByLabelText('Название пространства')).not.toBeInTheDocument()
  })

  it('reports legacy membership lookup errors with retry', () => {
    setupSpaces({
      userOverrides: { data: { ...adminUser, is_system_admin: false } },
      membersOverrides: {
        data: undefined,
        isLoading: false,
        isError: true,
        error: { code: 'FORBIDDEN', message: 'Forbidden' },
      },
    })
    fireEvent.click(screen.getByRole('button', { name: /База знаний Base/ }))
    expect(screen.getByRole('alert')).toHaveTextContent('Недостаточно прав для действия')
    fireEvent.click(screen.getByRole('button', { name: 'Повторить' }))
    expect(refetchMembers).toHaveBeenCalled()
  })

  it('keeps archived spaces read-only', () => {
    setupSpaces({
      spaceList: [
        {
          id: 'space-sdlc',
          key: 'BASE',
          name: 'База знаний Base',
          description: 'Основные документы продукта',
          owner_id: 'user-admin',
          status: 'archived',
          document_count: 2,
          member_count: 2,
          created_at: '2026-08-31T10:00:00Z',
          updated_at: '2026-08-31T11:00:00Z',
        },
      ],
    })

    fireEvent.click(screen.getByRole('button', { name: /База знаний Base/ }))
    expect(screen.queryByLabelText('Название пространства')).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Архивировать' })).not.toBeInTheDocument()
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
    expect(useSpaceMembers).toHaveBeenCalledWith('S13', false)

    fireEvent.change(screen.getByRole('searchbox', { name: 'Найти пространство' }), {
      target: { value: 'Space 25' },
    })
    expect(screen.getByText('Space 25')).toBeInTheDocument()
    expect(screen.queryByText('Space 13')).not.toBeInTheDocument()
  })
})
