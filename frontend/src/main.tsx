import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { I18nextProvider, useTranslation } from 'react-i18next'
import { Toaster } from '@pdlc/ui/ui'
import i18n from './shared/i18n/config'
import { RouterProvider } from 'react-router'
import { router } from './app/router'
import { AuthBoundary } from './app/auth-boundary'
import { ThemeProvider, PlatformProvider, PlatformServicesProvider } from '@pdlc/ui/lib'
import './index.css'
import { relaySsoCallback, ssoNamespace } from '@pdlc/ui/sso'
import { capturePlatformReturn, getPlatformProduct } from '@pdlc/ui/platform'

capturePlatformReturn(import.meta.env.VITE_PLATFORM_ORIGIN)

function Boot() {
  const { ready } = useTranslation(undefined, { i18n, useSuspense: false })
  if (!ready) return null
  return (
    <I18nextProvider i18n={i18n}>
      <AuthBoundary>
        <ThemeProvider>
          <PlatformProvider configUrl={import.meta.env.VITE_PLATFORM_BRANDING_URL ?? null}>
            <PlatformServicesProvider
              catalogUrl={import.meta.env.VITE_PLATFORM_SERVICES_URL ?? null}
            >
              <RouterProvider router={router} />
            </PlatformServicesProvider>
          </PlatformProvider>
          <Toaster />
        </ThemeProvider>
      </AuthBoundary>
    </I18nextProvider>
  )
}

if (!relaySsoCallback(ssoNamespace(import.meta.env.VITE_AUTH_NAMESPACE))) {
  const root = createRoot(document.getElementById('root')!)
  getPlatformProduct()?.onRetire(() => {
    root.unmount()
    router.dispose()
  })
  root.render(
    <StrictMode>
      <Boot />
    </StrictMode>,
  )
}
