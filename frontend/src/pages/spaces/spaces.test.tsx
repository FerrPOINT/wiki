import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router'
import { afterEach, describe, expect, it, vi } from 'vitest'

import { SpacesPage } from './'

const useSpaces = vi.hoisted(() => vi.fn())
const useSpaceTree = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({
  defaultSpaceKey: 'SDLC',
  useSpaces,
  useSpaceTree,
}))

describe('SpacesPage', () => {
  afterEach(() => {
    vi.clearAllMocks()
  })

  it('renders API-backed spaces with document tree preview links', () => {
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
            member_count: 3,
            created_at: '2026-08-31T10:00:00Z',
            updated_at: '2026-08-31T11:00:00Z',
          },
        ],
      },
      isLoading: false,
      isError: false,
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
    })

    render(
      <MemoryRouter>
        <SpacesPage />
      </MemoryRouter>,
    )

    expect(screen.getByRole('heading', { name: 'Пространства' })).toBeInTheDocument()
    expect(screen.getByText(/SDLC · База знаний SDLC/)).toBeInTheDocument()
    expect(screen.getByText('Основные документы продукта')).toBeInTheDocument()
    expect(screen.getByText('Дерево')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: /Требования требования/ })).toHaveAttribute(
      'href',
      '/documents/product-requirements',
    )
    expect(screen.getByRole('link', { name: /План проверки план проверки/ })).toHaveAttribute(
      'href',
      '/documents/test-plan',
    )
    expect(useSpaceTree).toHaveBeenCalledWith('SDLC')
  })

  it('shows the empty state action for a fresh Wiki instance', () => {
    useSpaces.mockReturnValue({
      data: { spaces: [] },
      isLoading: false,
      isError: false,
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
