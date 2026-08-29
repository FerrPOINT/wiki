// Evidence screenshots for Wiki README and docs.
// Usage: node scripts/shoot-evidence.mjs
import { mkdirSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { chromium } from '@playwright/test'

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..', '..')
const OUT = join(ROOT, 'docs', 'screenshots')
const BASE = process.env.PLAYWRIGHT_BASE_URL ?? 'http://127.0.0.1:4173'

mkdirSync(OUT, { recursive: true })

const authState = {
  state: {
    token: 'screenshot-token',
    userId: '00000000-0000-0000-0000-000000000001',
    email: 'demo@example.test',
    username: 'demo',
    displayName: 'Demo User',
  },
  version: 0,
}

const shots = [
  { name: '01-login.png', path: '/login', title: 'Login' },
  { name: '02-register.png', path: '/register', title: 'Register' },
  { name: '03-dashboard.png', path: '/', title: 'Dashboard' },
  { name: '04-spaces.png', path: '/spaces', title: 'Spaces' },
  { name: '05-document-compose.png', path: '/documents/new', title: 'Document compose' },
  { name: '06-document-view.png', path: '/documents/product-requirements', title: 'Document view' },
  { name: '07-task-dossiers.png', path: '/tasks', title: 'Task pages' },
  { name: '08-task-dossier-detail.png', path: '/tasks/SDLC-42', title: 'Task page detail' },
  { name: '09-phase-dossiers.png', path: '/phases', title: 'Phase pages' },
  {
    name: '10-phase-dossier-detail.png',
    path: '/phases/implementation',
    title: 'Phase page detail',
  },
  { name: '11-evidence.png', path: '/evidence', title: 'Evidence' },
  { name: '12-templates.png', path: '/templates', title: 'Templates' },
  { name: '13-audit-log.png', path: '/audit-log', title: 'Audit log' },
  { name: '14-users.png', path: '/users', title: 'Users' },
  { name: '15-settings.png', path: '/settings', title: 'Settings' },
  { name: '16-search.png', path: '/search', title: 'Search' },
  { name: '17-admin.png', path: '/admin', title: 'Administration' },
  { name: 'm-dashboard.png', path: '/', title: 'Dashboard mobile', mobile: true },
  { name: 'm-spaces.png', path: '/spaces', title: 'Spaces mobile', mobile: true },
  {
    name: 'm-document-view.png',
    path: '/documents/product-requirements',
    title: 'Document view mobile',
    mobile: true,
  },
  {
    name: 'm-task-dossier.png',
    path: '/tasks/SDLC-42',
    title: 'Task page mobile',
    mobile: true,
  },
  { name: 'm-search.png', path: '/search', title: 'Search mobile', mobile: true },
]

function routeJson(route, body, status = 200) {
  return route.fulfill({
    status,
    contentType: 'application/json',
    body: JSON.stringify(body),
  })
}

async function installApiMocks(page) {
  await page.route('**/api/v1/auth/login', (route) =>
    routeJson(route, {
      access_token: 'screenshot-token',
      refresh_token: 'screenshot-refresh',
      user_id: authState.state.userId,
      email: authState.state.email,
      username: authState.state.username,
      display_name: authState.state.displayName,
    }),
  )
  await page.route('**/api/v1/auth/register', (route) =>
    routeJson(route, {
      access_token: 'screenshot-token',
      refresh_token: 'screenshot-refresh',
      user_id: authState.state.userId,
      email: 'new@example.test',
      username: 'new-user',
      display_name: 'New User',
    }),
  )
  await page.route('**/api/v1/auth/refresh', (route) =>
    routeJson(route, { access_token: 'screenshot-token', refresh_token: 'screenshot-refresh' }),
  )
  await page.route('**/api/v1/auth/logout', (route) => route.fulfill({ status: 204 }))
  await page.route('**/api/v1/users/me', (route) =>
    routeJson(route, {
      id: authState.state.userId,
      email: authState.state.email,
      username: authState.state.username,
      display_name: authState.state.displayName,
      is_system_admin: true,
    }),
  )
}

const browser = await chromium.launch()

async function shoot(shot) {
  const context = await browser.newContext({
    viewport: shot.mobile ? { width: 375, height: 812 } : { width: 1920, height: 1080 },
    deviceScaleFactor: 1,
    locale: 'ru-RU',
  })
  const page = await context.newPage()
  await page.addInitScript((state) => {
    localStorage.setItem('theme', 'dark')
    localStorage.setItem('wiki-auth', JSON.stringify(state))
  }, authState)
  await installApiMocks(page)

  try {
    await page.goto(`${BASE}${shot.path}`, { waitUntil: 'networkidle', timeout: 30_000 })
    await page.waitForTimeout(300)
    await page.screenshot({ path: join(OUT, shot.name), fullPage: true })
    console.log(`shot ${shot.name} ${shot.path} ${shot.title}`)
  } finally {
    await context.close()
  }
}

for (const shot of shots) await shoot(shot)

await browser.close()
console.log(`done: ${shots.length}`)
