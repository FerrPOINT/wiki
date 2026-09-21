import { fireEvent, render, screen, within } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { SettingsPage } from './'

const useWikiSettings = vi.hoisted(() => vi.fn())

vi.mock('@/shared/api/hooks', () => ({
  useWikiSettings,
}))

describe('SettingsPage', () => {
  const settings = {
    instance_name: 'Wiki',
    api_base_path: '/api/v1',
    default_space_key: 'BASE',
    default_language: 'ru',
    timezone: 'Europe/Moscow',
    registration_enabled: true,
    public_links_enabled: false,
    search_backend: 'PostgreSQL FTS',
    storage_backend: 'local',
    max_upload_bytes: 26214400,
    markdown_renderer: 'comrak',
    html_sanitizer: 'ammonia',
  }

  it('renders API-backed settings as a read-only overview', () => {
    useWikiSettings.mockReturnValue({
      data: settings,
      isLoading: false,
      isError: false,
      refetch: vi.fn(),
    })

    render(<SettingsPage />)

    expect(screen.getByRole('heading', { name: 'Настройки' })).toBeInTheDocument()
    expect(screen.queryByRole('textbox')).not.toBeInTheDocument()
    expect(
      within(screen.getByRole('region', { name: 'Инстанс' })).getByText('Wiki'),
    ).toBeInTheDocument()
    expect(screen.getByText('/api/v1')).toBeInTheDocument()
    expect(screen.getByText('PostgreSQL FTS')).toBeInTheDocument()
    expect(screen.getByText('25 МБ')).toBeInTheDocument()
    const access = within(screen.getByRole('region', { name: 'Доступ' }))
    expect(access.getByText('включено')).toBeInTheDocument()
    expect(access.getByText('выключено')).toBeInTheDocument()
  })

  it('does not show stale values after a failed refresh and retries on request', () => {
    const refetch = vi.fn()
    useWikiSettings.mockReturnValue({
      data: settings,
      isLoading: false,
      isError: true,
      error: new Error('network error'),
      refetch,
    })

    render(<SettingsPage />)

    expect(screen.queryByRole('region', { name: 'Инстанс' })).not.toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Повторить' }))
    expect(refetch).toHaveBeenCalledOnce()
  })

  it('shows loading state without stale values', () => {
    useWikiSettings.mockReturnValue({
      data: settings,
      isLoading: true,
      isError: false,
      refetch: vi.fn(),
    })

    render(<SettingsPage />)

    expect(screen.getByText('Загружаем настройки')).toBeInTheDocument()
    expect(screen.queryByRole('region', { name: 'Инстанс' })).not.toBeInTheDocument()
  })
})
