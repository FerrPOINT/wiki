import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { SettingsPage } from './'

const useWikiSettings = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({
  useWikiSettings,
}))

describe('SettingsPage', () => {
  it('renders API-backed instance settings', () => {
    useWikiSettings.mockReturnValue({
      data: {
        instance_name: 'Wiki',
        api_base_path: '/api/v1',
        default_space_key: 'SDLC',
        default_language: 'ru',
        timezone: 'Europe/Moscow',
        registration_enabled: true,
        public_links_enabled: false,
        search_backend: 'PostgreSQL FTS',
        storage_backend: 'local',
        max_upload_bytes: 26214400,
        markdown_renderer: 'comrak',
        html_sanitizer: 'ammonia',
      },
      isLoading: false,
      isError: false,
      refetch: vi.fn(),
    })

    render(<SettingsPage />)

    expect(screen.getByRole('heading', { name: 'Настройки' })).toBeInTheDocument()
    expect(screen.getByDisplayValue('Wiki')).toBeInTheDocument()
    expect(screen.getByDisplayValue('/api/v1')).toBeInTheDocument()
    expect(screen.getByDisplayValue('PostgreSQL FTS')).toBeInTheDocument()
    expect(screen.getByDisplayValue('25 МБ')).toBeInTheDocument()
  })
})
