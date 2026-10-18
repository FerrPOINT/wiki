import i18n from 'i18next'
import { expect, it } from 'vitest'

it('keeps DOM matchers, Russian translations and browser storage ready', () => {
  const button = document.createElement('button')
  button.textContent = i18n.t('auth.login')
  document.body.append(button)
  try {
    expect(i18n.language).toBe('ru')
    expect(button).toBeInTheDocument()
    expect(button).toHaveTextContent('Войти')
    for (const name of ['localStorage', 'sessionStorage'] as const) {
      const storage = globalThis[name]
      const key = '__wiki_setup_contract__'
      const previous = storage.getItem(key)
      try {
        storage.setItem(key, 'ready')
        expect(window[name].getItem(key)).toBe('ready')
      } finally {
        if (previous === null) storage.removeItem(key)
        else storage.setItem(key, previous)
      }
    }
  } finally {
    button.remove()
  }
})
