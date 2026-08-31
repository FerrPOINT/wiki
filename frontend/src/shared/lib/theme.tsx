import { createContext, useContext, useEffect, useState } from 'react'

import { getSafeBrowserStorage } from './browser-storage'

type Theme = 'dark' | 'gray' | 'light'

interface ThemeContextValue {
  theme: Theme
  setTheme: (theme: Theme) => void
}

const ThemeContext = createContext<ThemeContextValue | null>(null)

function readInitialTheme(): Theme {
  try {
    const stored = getSafeBrowserStorage().getItem('theme')
    if (stored === 'dark' || stored === 'gray' || stored === 'light') return stored
  } catch {
    // Theme can still render with the default when storage cannot be read.
  }
  return 'dark'
}

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [theme, setThemeState] = useState<Theme>(readInitialTheme)

  const setTheme = (value: Theme) => {
    setThemeState(value)
    try {
      getSafeBrowserStorage().setItem('theme', value)
    } catch {
      // Theme still updates in memory even when browser storage is unavailable.
    }
  }

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme)
  }, [theme])

  return <ThemeContext.Provider value={{ theme, setTheme }}>{children}</ThemeContext.Provider>
}

export function useTheme() {
  const ctx = useContext(ThemeContext)
  if (!ctx) throw new Error('useTheme must be used within ThemeProvider')
  return ctx
}
