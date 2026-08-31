import { expect, test, type Page, type Route } from '@playwright/test'

const baseURL = process.env.PLAYWRIGHT_BASE_URL ?? 'http://localhost:4173'

const now = '2026-08-31T10:00:00Z'
const user = {
  id: '00000000-0000-0000-0000-000000000001',
  email: 'demo@example.com',
  username: 'demo',
  display_name: 'Демо пользователь',
  role: 'admin',
  is_system_admin: true,
  active: true,
}
const evidence = {
  id: 'evidence-smoke',
  space_key: 'SDLC',
  document_id: 'product-requirements',
  task_key: 'SDLC-42',
  phase_key: 'implementation',
  title: 'Материал smoke-проверки фронта',
  evidence_type: 'external_url',
  url: 'https://ci.local/jobs/wiki-smoke',
  attachment_id: null,
  checksum: null,
  created_by: user.id,
  created_at: now,
}
const document = {
  id: 'product-requirements',
  space_key: 'SDLC',
  parent_id: null,
  slug: 'product-requirements',
  title: 'Требования к Wiki MVP',
  document_type: 'requirements',
  status: 'published',
  body_markdown:
    '# Требования к Wiki MVP\n\nБазовый документ для пространств, документов, связей с задачами и фазами, материалов, поиска и аудита.',
  draft_markdown:
    '# Требования к Wiki MVP\n\nБазовый документ для пространств, документов, связей с задачами и фазами, материалов, поиска и аудита.',
  current_revision: {
    id: 'revision-product-requirements-1',
    document_id: 'product-requirements',
    version: 1,
    title: 'Требования к Wiki MVP',
    body_markdown:
      '# Требования к Wiki MVP\n\nБазовый документ для пространств, документов, связей с задачами и фазами, материалов, поиска и аудита.',
    summary: 'Исходные требования MVP',
    author_id: user.id,
    published_at: now,
  },
  task_keys: ['SDLC-42'],
  phase_keys: ['implementation'],
  evidence: [evidence],
  created_by: user.id,
  updated_by: user.id,
  created_at: now,
  updated_at: now,
}
const documentSummary = {
  id: document.id,
  slug: document.slug,
  title: document.title,
  document_type: document.document_type,
  status: document.status,
  updated_at: document.updated_at,
}
const task = {
  space_key: 'SDLC',
  task_key: 'SDLC-42',
  title: document.title,
  document_count: 1,
  evidence_count: 1,
  documents: [documentSummary],
  evidence: [evidence],
}
const phase = {
  space_key: 'SDLC',
  phase_key: 'implementation',
  title: 'implementation',
  document_count: 1,
  evidence_count: 1,
  documents: [documentSummary],
  evidence: [evidence],
}

function routeJson(route: Route, body: unknown, status = 200) {
  return route.fulfill({
    status,
    contentType: 'application/json',
    body: JSON.stringify(body),
  })
}

async function installWikiApiMocks(page: Page) {
  const searchRequests: string[] = []

  await page.route('**/api/v1/**', (route) => {
    const request = route.request()
    const url = new URL(request.url())
    const path = url.pathname.replace('/api/v1', '')
    const method = request.method()

    if (method === 'POST' && path === '/auth/login') {
      return routeJson(route, {
        access_token: 'demo-token',
        refresh_token: 'demo-refresh',
        token_type: 'Bearer',
        user_id: user.id,
        email: user.email,
        username: user.username,
        display_name: user.display_name,
      })
    }
    if (method === 'POST' && path === '/auth/refresh') {
      return routeJson(route, { access_token: 'demo-token', refresh_token: 'demo-refresh' })
    }
    if (method === 'POST' && path === '/auth/logout') return route.fulfill({ status: 204 })
    if (method === 'GET' && path === '/users/me') return routeJson(route, user)
    if (method === 'GET' && path === '/users') return routeJson(route, { users: [user] })
    if (method === 'GET' && path === '/spaces') {
      return routeJson(route, {
        spaces: [
          {
            id: 'space-sdlc',
            key: 'SDLC',
            name: 'База знаний SDLC',
            description: 'Основное пространство Wiki для документов SDLC',
            owner_id: user.id,
            status: 'active',
            document_count: 1,
            member_count: 1,
            created_at: now,
            updated_at: now,
          },
        ],
      })
    }
    if (method === 'GET' && path === '/spaces/SDLC/tree') {
      return routeJson(route, {
        space_key: 'SDLC',
        documents: [
          {
            id: document.id,
            slug: document.slug,
            title: document.title,
            document_type: document.document_type,
            status: document.status,
            children: [],
          },
        ],
      })
    }
    if (method === 'GET' && path === '/documents/product-requirements') {
      return routeJson(route, document)
    }
    if (method === 'GET' && path === '/documents/product-requirements/revisions') {
      return routeJson(route, { revisions: [document.current_revision] })
    }
    if (method === 'GET' && path === '/spaces/SDLC/tasks')
      return routeJson(route, { tasks: [task] })
    if (method === 'GET' && path === '/spaces/SDLC/tasks/SDLC-42') return routeJson(route, task)
    if (method === 'GET' && path === '/spaces/SDLC/phases') {
      return routeJson(route, { phases: [phase] })
    }
    if (method === 'GET' && path === '/spaces/SDLC/phases/implementation') {
      return routeJson(route, phase)
    }
    if (method === 'GET' && path === '/evidence') return routeJson(route, { evidence: [evidence] })
    if (method === 'GET' && path === '/templates') {
      return routeJson(route, {
        templates: [
          {
            id: 'requirements',
            name: 'Требования',
            document_type: 'requirements',
            body_markdown: '# Требования\n\n## Контекст\n\n## Решения\n\n## Проверки\n',
          },
        ],
      })
    }
    if (method === 'GET' && path === '/audit-log') {
      return routeJson(route, {
        entries: [
          {
            id: 'audit-initial',
            actor_id: user.id,
            action: 'wiki.seeded',
            entity_type: 'space',
            entity_id: 'SDLC',
            created_at: now,
          },
        ],
      })
    }
    if (method === 'GET' && path === '/search') {
      searchRequests.push(url.search)
      return routeJson(route, {
        results: [
          {
            id: document.id,
            result_type: 'document',
            title: document.title,
            space_key: document.space_key,
            url: `/documents/${document.slug}`,
            snippet: document.body_markdown,
            updated_at: document.updated_at,
          },
          {
            id: evidence.id,
            result_type: 'evidence',
            title: evidence.title,
            space_key: evidence.space_key,
            url: `/evidence?id=${evidence.id}`,
            snippet: evidence.url,
            updated_at: evidence.created_at,
          },
        ],
      })
    }

    return routeJson(route, { error: `Unhandled mock route ${method} ${path}` }, 404)
  })

  return { searchRequests }
}

test.describe('wiki smoke', () => {
  test('login and navigate through wiki shell pages', async ({ page }) => {
    const apiMocks = await installWikiApiMocks(page)
    await page.goto(`${baseURL}/login`)
    await page.getByRole('textbox').nth(0).fill('demo@example.com')
    await page.getByRole('textbox').nth(1).fill('demo')
    await page.getByRole('button', { name: /войти/i }).click()

    await expect(page).toHaveURL(`${baseURL}/`, { timeout: 10_000 })
    await expect(page.getByRole('heading', { name: 'Wiki', exact: true })).toBeVisible()

    await page.goto(`${baseURL}/spaces`)
    await expect(page.getByRole('heading', { name: 'Пространства' })).toBeVisible()
    await expect(page.getByText('База знаний SDLC')).toBeVisible()

    await page.goto(`${baseURL}/documents/new`)
    await expect(page.getByRole('heading', { name: 'Новый документ' })).toBeVisible()

    await page.goto(`${baseURL}/tasks/SDLC-42`)
    await expect(page.getByRole('heading', { name: 'SDLC-42' })).toBeVisible()

    await page.goto(`${baseURL}/phases/implementation`)
    await expect(page.getByRole('heading', { name: 'implementation' })).toBeVisible()

    await page.goto(`${baseURL}/search`)
    await expect(page.getByRole('heading', { name: 'Поиск' })).toBeVisible()
    await page.getByRole('button', { name: 'Требования', exact: true }).click()
    await expect
      .poll(() =>
        apiMocks.searchRequests.some((query) => query.includes('document_type=requirements')),
      )
      .toBe(true)
  })
})
