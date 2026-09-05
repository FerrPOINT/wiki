import { StrictMode, useEffect, useState } from 'react'
import { createRoot } from 'react-dom/client'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { I18nextProvider } from 'react-i18next'
import { Toaster } from 'sonner'
import i18n from './shared/i18n/config'
import { RouterProvider } from 'react-router'
import { router } from './app/router'
import { ThemeProvider, PlatformProvider, PlatformServicesProvider } from '@sdlc/ui/lib'
import './index.css'

const queryClient = new QueryClient()

function Boot() {
  const [ready, setReady] = useState(i18n.isInitialized)
  useEffect(() => {
    if (i18n.isInitialized) {
      setReady(true)
      return
    }
    const handler = () => setReady(true)
    i18n.on('initialized', handler)
    return () => {
      i18n.off('initialized', handler)
    }
  }, [])
  if (!ready) return null
  return (
    <I18nextProvider i18n={i18n}>
      <QueryClientProvider client={queryClient}>
        <ThemeProvider>
          <PlatformProvider configUrl={import.meta.env.VITE_PLATFORM_BRANDING_URL ?? null}>
      <PlatformServicesProvider catalogUrl={import.meta.env.VITE_PLATFORM_SERVICES_URL ?? null}>
            <RouterProvider router={router} />
          </PlatformServicesProvider>
    </PlatformProvider>
          <Toaster theme="dark" />
        </ThemeProvider>
      </QueryClientProvider>
    </I18nextProvider>
  )
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <Boot />
  </StrictMode>,
)
